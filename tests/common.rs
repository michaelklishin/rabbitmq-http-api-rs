use std::env;
use std::time::Duration;

pub const ENDPOINT: &str = "http://localhost:15672";
pub const USERNAME: &str = "guest";
pub const PASSWORD: &str = "guest";

pub fn endpoint() -> String {
    ENDPOINT.to_owned()
}

#[allow(dead_code)]
pub fn await_metric_emission(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

#[allow(dead_code)]
pub fn await_queue_metric_emission() {
    let delay = env::var("TEST_STATS_DELAY").unwrap_or("500".to_owned());
    await_metric_emission(delay.parse::<u64>().unwrap());
}
