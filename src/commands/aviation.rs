use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{CommandResult, macros::command, Args};
use serenity::utils::Colour;
use urlencoding::encode as url_encode;
use std::env;
use crate::models::avwx::{AvwxIcao, AvwxIcaoRunway};
use std::time::Duration;
use serenity::futures::StreamExt;
use serenity::builder::CreateEmbed;
use crate::helpers::global_data::ReqwestContainer;

#[command]
#[num_args(1)]
#[aliases("ic")]
#[usage = "ident"]
async fn icao(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Parse the arguments
    let icao = match args.single::<String>() {
        Ok(icao) => icao,
        Err(_) => { msg.channel_id.say(ctx, ":no_entry_sign: Couldn't parse the ICAO ident, Are you sure it's valid?").await?; return Ok(()); }
    };

    // Get the reqwest client
    let reqwest_client  = ctx.data.read().await.get::<ReqwestContainer>().cloned().unwrap();
    // Fetch the data
    let avwx_response = reqwest_client.get(&format!("https://avwx.rest/api/station/{}?format=json", url_encode(&icao)))
        .header(reqwest::header::AUTHORIZATION, &format!("Token {}", env::var("AVWX_TOKEN").unwrap()))
        .send().await?;

    // Deserialize the data
    let avwx_data = match avwx_response.json::<AvwxIcao>().await {
        Ok(avwx_data) => avwx_data,
        Err(why) => {
            if why.is_decode() {
                msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
                    e.title("An error occurred decoding AVWX's response.")
                        .description(format!("```{}```", why))
                        .color(Colour::RED)
                })).await?;
            } else {
                msg.channel_id.say(&ctx.http, ":no_entry_sign: An error occurred fetching from AVWX's API.").await?;
            }

            return Ok(())
        }
    };

    // Position in the menu
    let mut current_pos = 0;

    // Send the embed
    let mut sent_message = msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
        e.0 = create_icao_embed(current_pos as i32, &avwx_data).0;
        e
    })).await?;

    // React the menu
    sent_message.react(&ctx.http, ReactionType::Unicode(String::from("⬅"))).await?;
    sent_message.react(&ctx.http, ReactionType::Unicode(String::from("➡"))).await?;

    // Check if the bot can manage messages, And if so enable the bot to remove reactions.
    let mut can_manage_messages = false;
    if let Some(guild) = msg.guild(&ctx).await {
        can_manage_messages = guild
            .user_permissions_in(guild.channels.get(&msg.channel_id).unwrap(), guild.members.get(&ctx.http.get_current_user().await?.id).unwrap())
            .unwrap().contains(Permissions::MANAGE_MESSAGES);
    }

    // Create a reaction collector and wait for someone to react to the message
    let mut reactions_collector = sent_message.await_reactions(&ctx).timeout(Duration::from_secs(5 * 60))
        .author_id(msg.author.id).added(true).removed(!can_manage_messages).await;
    while let Some(reaction) = reactions_collector.next().await {
        // Delete the reaction
        if can_manage_messages {
            reaction.as_inner_ref().delete(&ctx.http).await?;
        }

        let emoji = &reaction.as_inner_ref().emoji.to_string();
        if emoji == "⬅" && current_pos != 0 {
            current_pos -= 1;
        } else if emoji == "➡" && &current_pos != &avwx_data.runways.len() {
            current_pos += 1;
        }

        // Edit the original message
        &sent_message.edit(&ctx.http, |f| f.embed(|embed| {
            if current_pos == 0 {
                embed.0 = create_icao_embed(current_pos as i32, &avwx_data).0;
            } else {
                embed.0 = create_icao_runway_embed(current_pos as i32, &avwx_data.runways.get(current_pos - 1).unwrap()).0;
            }
            embed
        })).await;
    }

    // Delete all reactions once done
    if can_manage_messages {
        &sent_message.delete_reactions(&ctx.http).await?;
    }

    Ok(())
}

fn create_icao_embed(position: i32, avwx_icao: &AvwxIcao) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed.title(format!("{} - {}", avwx_icao.icao, &avwx_icao.name))
        .url(&avwx_icao.wiki)
        .field("Location", format!("Country: {}\nCity: {}\nLatitude: {}\nLongitude: {}",
                                   &avwx_icao.country, &avwx_icao.city, &avwx_icao.latitude, &avwx_icao.longitude), true)
        .field("Elevation", format!("In feet: {}\nIn meters: {}",
                                    &avwx_icao.elevation_ft, &avwx_icao.elevation_m), true)
        .field("Other", format!("Iata: {}\nType: {}\nReporting: {}", &avwx_icao.iata, &avwx_icao.airport_type, &avwx_icao.reporting), true)
        .footer(|f| f.text(format!("Menu position: {}", position)))
        .color(Colour::BLITZ_BLUE);

    let mut extra_text = String::new();
    if let Some(website) = &avwx_icao.website { extra_text += &format!("Website: {}\n", website) }
    if let Some(note) = &avwx_icao.note { extra_text += &format!("Note: {}\n", note) }
    if !extra_text.is_empty() { embed.field("Extra", extra_text, true); }

    embed
}

fn create_icao_runway_embed(position: i32, avwx_icao_runway: &AvwxIcaoRunway) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed.title(format!("Runway {}-{}", &avwx_icao_runway.ident1, &avwx_icao_runway.ident2))
        .field("Bearings", format!("One: {}\nTwo: {}", &avwx_icao_runway.bearing1, &avwx_icao_runway.bearing2), true)
        //.field("Idents", format!("One: {}\nTwo: {}", &avwx_icao_runway.ident1, &avwx_icao_runway.ident2), true)
        .field("Size", format!("Width: {}\nLenght: {}", &avwx_icao_runway.width_ft, &avwx_icao_runway.width_ft), true)
        .field("Other", format!("Surface: {}\nLights: {}", &avwx_icao_runway.surface, &avwx_icao_runway.lights), true)
        .footer(|f| f.text(format!("Menu position: {}", position)))
        .color(Colour::BLITZ_BLUE);

    embed
}