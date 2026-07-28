#[allow(unused)]
pub mod klipper_mcu;

pub mod core_xy;
mod device_map;
pub mod printer_runtime;

mod connection;
mod elegoo_0xA55A;

mod dummy {
    mod axis;
    mod mcu;

    pub use axis::DummyAxis;
    pub use mcu::SimMCURuntime;
}
