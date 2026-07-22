use serde::Deserialize;

use crate::config::pin::Pin;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Axis {
    #[serde(rename = "tmc2209")]
    Tmc2209(Tmc2209Config),
    #[serde(rename = "a4988")]
    A4988(A4988Config),
    #[serde(rename = "dummy")]
    Dummy,
}

#[derive(Debug, Deserialize)]
pub struct AxisConfig {
    step_pin: Pin,
    dir_pin: Pin,
}

#[derive(Debug, Deserialize)]
pub struct Tmc2209Config {
    uart_pin: Pin,
    run_current: f32,

    #[serde(flatten)]
    config: AxisConfig,
}

#[derive(Debug, Deserialize)]
pub struct A4988Config {
    other_config: f32,

    #[serde(flatten)]
    config: AxisConfig,
}
