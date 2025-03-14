use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub garage_door: GarageDoorConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GarageDoorConfig {
    pub close_limit_pin: u8,
    pub open_limit_pin: u8,
    pub coupler_pin: u8,
    pub poll_interval_ms: u64,
    pub expected_shut_time_sec: u64,
    pub shut_time_buffer_sec: u64,
    pub coupler_duration_intervals: u64,
    pub server_address: String,
    pub api_key: String,
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize::<AppConfig>()
}