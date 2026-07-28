use alloc::string::String;
use serde::Deserialize;

use crate::utils::{pin::Pin, units::Length};

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
pub struct GeneralAxisConfig {}

fn default_hold_current() -> f32 {
    2.0
}

fn default_sense_resistor() -> f32 {
    0.110
}

fn default_true() -> bool {
    true
}

fn default_full_steps() -> u16 {
    200
}

fn default_microsteps() -> u16 {
    16
}

#[derive(Debug, Deserialize)]
pub struct Tmc2209Config {
    pub step_pin: Pin<4>,
    pub dir_pin: Pin<4>,
    pub uart_pin: Pin<4>,
    pub tx_pin: Pin<4>,

    #[serde(default)]
    pub uart_address: u8,

    pub run_current: f32,

    #[serde(default = "default_hold_current")]
    pub hold_current: f32,

    #[serde(default = "default_sense_resistor")]
    pub sense_resistor: f32,

    #[serde(default)]
    pub driver_sgthrs: u8,

    #[serde(default = "default_true")]
    pub interpolate: bool,

    #[serde(default)]
    pub diag_pin: Option<Pin<4>>,

    /// mm per full rotation of the stepper motor shaft
    pub rotation_distance: Length,

    /// full steps per rotation of the motor (typically 200)
    #[serde(default = "default_full_steps")]
    pub full_steps_per_rotation: u16,

    /// microsteps configured on the driver
    #[serde(default = "default_microsteps")]
    pub microsteps: u16,

    #[serde(flatten)]
    pub config: GeneralAxisConfig,
}

#[derive(Debug, Deserialize)]
pub struct A4988Config {
    pub step_pin: String,
    pub dir_pin: String,
    pub enable_pin: String,

    #[serde(flatten)]
    pub config: GeneralAxisConfig,
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
