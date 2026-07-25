#[allow(unused)]
pub mod klipper_mcu;

pub mod core_xy;
pub mod printer_runtime;

mod dummy {
    mod axis;
    mod mcu;

    pub use axis::DummyAxis;
    pub use mcu::SimMCURuntime;
}
