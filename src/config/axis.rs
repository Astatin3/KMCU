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
    Dummy(DummyAxisConfig),
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

fn dummy_step_amount_mm() -> f32 {
    0.1
}
fn dummy_limits_mm() -> (f32, f32) {
    (0., 256.)
}

#[derive(Debug, Deserialize)]
pub struct DummyAxisConfig {
    #[serde(default = "dummy_step_amount_mm")]
    pub step_amount_mm: f32,

    #[serde(default = "dummy_limits_mm")]
    pub limits_mm: (f32, f32),
}
