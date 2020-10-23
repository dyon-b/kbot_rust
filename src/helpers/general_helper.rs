// Function from Robo Arc
pub fn seconds_to_days(seconds: u64) -> String {
    let days = seconds / 60 / 60 / 24;
    let hours = seconds / 3600 % 24;
    let minutes = seconds % 3600 / 60;
    let sec = seconds % 3600 % 60;

    if days == 0 {
        format!("{}:{:02}:{:02}", hours, minutes, sec)
    } else {
        format!("{} days {}:{:02}:{:02}", days, hours, minutes, sec)
    }
}
