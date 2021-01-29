use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AvwxError {
    pub error: String,
    pub param: String,
    pub help: String,
    pub timestamp: String,
}

#[derive(Deserialize, Debug)]
pub struct AvwxIcao {
    pub city: String,
    pub country: String,
    pub elevation_ft: i32,
    pub elevation_m: i32,
    pub iata: String,
    pub icao: String,
    pub latitude: f32,
    pub longitude: f32,
    pub name: String,
    pub note: Option<String>,
    pub reporting: bool,
    pub state: String,
    #[serde(alias = "type")]
    pub airport_type: String,
    pub website: Option<String>,
    pub wiki: String,
    pub runways: Vec<AvwxIcaoRunway>,
}

#[derive(Deserialize, Debug)]
pub struct AvwxIcaoRunway {
    pub length_ft: i32,
    pub width_ft: i32,
    pub surface: String,
    pub lights: bool,
    pub ident1: String,
    pub ident2: String,
    pub bearing1: f32,
    pub bearing2: f32,
}