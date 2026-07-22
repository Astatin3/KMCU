use serde::Deserialize;

use crate::config::pin::Pin;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AxisConfig {
    #[serde(rename = "tmc2209")]
    Tmc2209(Tmc2209Config),
    #[serde(rename = "a4988")]
    A4988(A4988Config),
    #[serde(rename = "dummy")]
    Dummy,
}

#[derive(Debug, Deserialize)]
pub struct GeneralAxisConfig {
    step_pin: String,
    dir_pin: String,
}

#[derive(Debug, Deserialize)]
pub struct Tmc2209Config {
    uart_pin: String,
    run_current: f32,

    #[serde(flatten)]
    config: GeneralAxisConfig,
}

#[derive(Debug, Deserialize)]
pub struct A4988Config {
    other_config: f32,

    #[serde(flatten)]
    config: GeneralAxisConfig,
}
