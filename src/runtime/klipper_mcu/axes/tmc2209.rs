use core::cell::RefCell;

use alloc::{boxed::Box, rc::Rc};

use crate::{
    config::Tmc2209Config,
    runtime::klipper_mcu::{
        KlipperMCURuntime,
        protocol::{RecvCommand, SendCommand},
    },
    traits::Axis,
    utils::{
        error::IOError,
        tmc,
        units::{Length, Velocity, length::htmm, velocity::htmm_per_millisecond},
    },
};

pub struct KTMC2209 {
    klipper: Rc<RefCell<KlipperMCURuntime>>,
    motor_oid: u8,
    clock_freq: u32,
    step_dd: Length,
    dir_pin_invert: u8,
}

impl KTMC2209 {
    pub fn new(
        config: Tmc2209Config,
        oid: u8,
        klipper: Rc<RefCell<KlipperMCURuntime>>,
    ) -> Result<Box<dyn Axis>, IOError> {
        let motor_oid = oid;
        let tmc_uart_oid = oid + 1;

        let one_wire = config.uart_pin.num == config.tx_pin.num;

        let invert_step = if config.step_pin.invert > 0 && config.dir_pin.invert > 0 {
            2
        } else {
            config.step_pin.invert
        };

        klipper
            .borrow_mut()
            .send_command_expect_ack(SendCommand::config_stepper {
                oid: motor_oid,
                step_pin: config.step_pin.num,
                dir_pin: config.dir_pin.num,
                invert_step,
                step_pulse_ticks: 0,
            })?;

        let rx_pin = config.uart_pin.num as u32;
        let tx_pin = if one_wire {
            rx_pin
        } else {
            config.tx_pin.num as u32
        };
        let clock_freq = klipper.borrow().identity.config.clock_freq;
        let bit_time = clock_freq / 40_000;

        klipper
            .borrow_mut()
            .send_command_expect_ack(SendCommand::config_tmcuart {
                oid: tmc_uart_oid,
                rx_pin,
                pull_up: 0,
                tx_pin,
                bit_time,
            })?;

        let addr = config.uart_address;

        let gconf = (1 << 6) | (1 << 7) | (1 << 8) | (1 << 2);
        tmc_send_write(&klipper, tmc_uart_oid, addr, 0x00, gconf)?;

        let (vsense, irun) = tmc::calc_current_bits(config.run_current, config.sense_resistor);
        let ihold = tmc::calc_hold_current_bits(
            config.hold_current.min(config.run_current),
            config.sense_resistor,
            vsense,
        );

        let mres = 4u32;
        let chopconf = (3 << 0)
            | (5 << 4)
            | (0 << 7)
            | (2 << 15)
            | (mres << 24)
            | ((config.interpolate as u32) << 28)
            | ((vsense as u32) << 17);
        tmc_send_write(&klipper, tmc_uart_oid, addr, 0x6C, chopconf)?;

        let ihold_irun = (ihold as u32) | ((irun as u32) << 8) | (8 << 16);
        tmc_send_write(&klipper, tmc_uart_oid, addr, 0x10, ihold_irun)?;

        tmc_send_write(&klipper, tmc_uart_oid, addr, 0x11, 20)?;

        tmc_send_write(
            &klipper,
            tmc_uart_oid,
            addr,
            0x40,
            config.driver_sgthrs as u32,
        )?;

        let coolconf = (config.driver_sgthrs as u32) << 16;
        tmc_send_write(&klipper, tmc_uart_oid, addr, 0x42, coolconf)?;

        let pwmconf = (36 << 0) | (14 << 8) | (1 << 16) | (1 << 18) | (1 << 19);
        tmc_send_write(&klipper, tmc_uart_oid, addr, 0x70, pwmconf)?;

        tmc_send_write(&klipper, tmc_uart_oid, addr, 0x14, 0xFFFFF)?;

        tmc_send_write(&klipper, tmc_uart_oid, addr, 0x13, 0)?;

        let total_steps = config.full_steps_per_rotation as i32 * config.microsteps as i32;
        let step_dd = config.rotation_distance / total_steps;

        Ok(Box::new(Self {
            klipper,
            motor_oid,
            clock_freq,
            step_dd,
            dir_pin_invert: config.dir_pin.invert,
        }) as Box<dyn Axis>)
    }
}

impl Axis for KTMC2209 {
    // Simple move: fires one step at the computed interval.
    //
    // Derivation:
    //   clock ticks per step = clock_freq / (steps_per_ms × 1000)
    //
    // Substituting steps_per_ms = raw / step_dd:
    //   interval = clock_freq / ((raw / step) × 1000)
    //            = clock_freq × step / (raw × 1000)
    //            = (clock_freq / 1000) × step / raw
    //
    // The naive approach computes steps/ms first via raw / step, which
    // truncates to zero for any velocity below 1 step/ms (e.g. 1 mm/s
    // with step_dd = 124 htmm → 10/124 = 0 steps/ms → zero steps queued).
    // The direct form (mul before div) avoids this intermediate truncation
    // while staying in pure u32 — ticks_per_ms × step fits comfortably
    // for all realistic motor parameters.
    fn simple_move(&mut self, velocity: Velocity)
    where
        Self: Sized,
    {
        let raw = velocity.get::<htmm_per_millisecond>();

        let dir = if raw > 0 { 0 } else { 1 };
        let dir = dir ^ (self.dir_pin_invert & 1);

        let _ = self
            .klipper
            .borrow_mut()
            .send_command_expect_ack(SendCommand::set_next_step_dir {
                oid: self.motor_oid,
                dir,
            });

        if raw == 0 {
            return;
        }

        let step = self.step_dd.get::<htmm>() as u32;
        if step == 0 {
            return;
        }

        let ticks_per_ms = self.clock_freq / 1000;
        let interval = ticks_per_ms * step / raw.unsigned_abs();

        let _ = self
            .klipper
            .borrow_mut()
            .send_command_expect_ack(SendCommand::queue_step {
                oid: self.motor_oid,
                interval,
                count: 1,
                add: 0,
            });
    }
}

fn tmc_send_write(
    klipper: &RefCell<KlipperMCURuntime>,
    tmc_uart_oid: u8,
    addr: u8,
    reg: u8,
    val: u32,
) -> Result<(), IOError> {
    let datagram = tmc::build_write_datagram(addr, reg, val);
    let serial_data = tmc::add_serial_bits(&datagram);
    let read_len = serial_data.len() as u8;

    match klipper
        .borrow_mut()
        .send_command_expect_reponse(SendCommand::tmcuart_send {
            oid: tmc_uart_oid,
            write: serial_data,
            read: read_len,
        })? {
        RecvCommand::tmcuart_response { oid: _, read } => {
            let raw = tmc::remove_serial_bits(&read, 8);
            if raw.len() < 8 {
                return Err(IOError::TmcUartVerificationFailed);
            }
            let crc = tmc::tmc_crc8(&raw[..7]);
            if crc != raw[7] {
                return Err(IOError::TmcUartVerificationFailed);
            }
            Ok(())
        }
        _ => Err(IOError::UnexpectedCommand),
    }
}
