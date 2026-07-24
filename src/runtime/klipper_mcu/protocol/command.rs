use crate::{runtime::klipper_mcu::protocol::dictionary::Dictionary, traits::binary::Binary};

macro_rules! command {
    (
        $(#[$enum_attr:meta])*
        $vis:vis enum $ename:ident {
            $(
                $(#[$vattr:meta])*
                $vname:ident $( { $($fname:ident : $fty:ty),* $(,)? } )? = $id:expr
            ),* $(,)?
        }
    ) => {
        $(#[$enum_attr])*
        $vis enum $ename {
            $(
                $(#[$vattr])*
                $vname $( { $($fname : $fty),* } )?
            ),*
        }

        impl Binary for $ename {
            type EncodeArg = Dictionary;
            type DecodeArg = Dictionary;

            fn encode(&self, writer: &mut dyn std::io::Write, dict: &Dictionary) -> anyhow::Result<()> {
                match self {
                    $(
                        $ename::$vname $( { $($fname),* } )? => {
                            let dynamic_id = dict.get_dynamic_id($id)
                                .ok_or_else(|| anyhow::anyhow!("unregistered command '{}'", stringify!($ename)))?;

                            <i16 as Binary>::encode(&dynamic_id, writer, &())?;

                            // Encode each argument
                            $(
                                $( <$fty as Binary>::encode($fname, writer, &())?; )*
                            )?

                            Ok(())
                        }
                    )*
                }
            }

            fn decode(reader: &mut dyn std::io::Read, dict: &Dictionary) -> anyhow::Result<Self> {
                // Read the id
                let id = <i16 as Binary>::decode(reader, &())?;

                // Convert this back to an actual static id
                let static_id = dict.get_static_id(id)
                    .ok_or_else(|| anyhow::anyhow!("unregistered command id '{}'", id))?;

                $(
                    if static_id == ($id) {

                        // Decode each argument
                        $(
                            $( let $fname = <$fty as Binary>::decode(reader, &())?; )*
                        )?

                        // Reconstruct self
                        return Ok($ename::$vname $( { $($fname),* } )?);
                    }
                )*

                Err(anyhow::anyhow!(
                    "unknown variant id {} for enum {}",
                    id,
                    stringify!($ename)
                ))
            }
        }

        impl $ename {
            /// Look up the wire id for a variant by its Rust name.
            pub fn id_for_name(name: &str) -> u16 {
                $(
                    if name == stringify!($vname) {
                        return $id;
                    }
                )*
                u16::MAX
            }
        }
    };
}

command! {
    #[derive(Clone, Debug)]
    #[allow(non_camel_case_types)]
    pub enum SendCommand {

        // ── Identification ────────────────────────────────────────────────

        /// Requests a chunk of the MCU's data dictionary (compressed JSON containing
        /// command definitions, constants, and enumerations). The host sends multiple
        /// `identify` commands with increasing offsets to download the full dictionary.
        /// This is the very first command sent after connecting.
        ///
        /// Response: [`RecvCommand::identify_response`]
        /// Source: `src/basecmd.c`
        identify { offset: u32, count: u8 } = 0,

        /// Informs the MCU of the maximum number of object IDs (oids) the host will
        /// require. Must be issued exactly once before any `config_` commands. An oid
        /// is an integer identifier assigned to each stepper, endstop, SPI device, I2C
        /// device, and other MCU objects. The MCU uses this to allocate its internal
        /// oid-to-object mapping table.
        ///
        /// Must precede all `config_` commands; followed by [`Self::finalize_config`].
        /// Source: `src/basecmd.c`
        allocate_oids { count: u8 } = 1,

        // ── Core / System ─────────────────────────────────────────────────

        /// Requests the MCU's uptime. The MCU responds with the upper 32 bits of its
        /// 64-bit uptime counter and the current 32-bit clock value. Used to track full
        /// 64-bit uptime and detect MCU resets. Can be invoked during shutdown.
        ///
        /// Response: [`RecvCommand::uptime`]
        /// Source: `src/basecmd.c`
        get_uptime = 2,

        /// Requests the current MCU clock value. The host sends this once per second
        /// to estimate clock drift between host and MCU for time synchronization.
        /// Can be invoked during shutdown.
        ///
        /// Response: [`RecvCommand::clock`]
        /// Source: `src/basecmd.c`
        get_clock = 3,

        /// Queries the MCU's current configuration state: whether it is configured,
        /// its stored CRC, whether it is in shutdown, and the number of available move
        /// queue entries. The host uses this to determine if (re)configuration is needed.
        /// Can be invoked during shutdown.
        ///
        /// Response: [`RecvCommand::config`]
        /// Source: `src/basecmd.c`
        get_config = 4,

        /// Transitions the MCU from unconfigured to configured state. Must be the last
        /// configuration command sent. The CRC is stored in the MCU and returned in
        /// subsequent `get_config` responses; the host compares it to determine if
        /// reconfiguration is needed on reconnection. Also allocates the move queue.
        ///
        /// Source: `src/basecmd.c`
        finalize_config { crc: u32 } = 5,

        /// Clears the MCU's shutdown state. When the MCU enters shutdown it refuses
        /// most commands; this command clears the shutdown flag to resume normal
        /// operation. Can be invoked during shutdown.
        ///
        /// Source: `src/basecmd.c`
        clear_shutdown = 6,

        /// Immediately forces the MCU into shutdown. All configured output pins revert
        /// to their default values, the move queue is cleared, and active timers are
        /// cancelled. Can be invoked during shutdown.
        ///
        /// Source: `src/basecmd.c`
        emergency_stop = 7,

        /// Performs a full MCU hardware reset. The MCU reboots, losing all
        /// configuration. After reset the host must re-download the data dictionary
        /// and reconfigure from scratch.
        ///
        /// Source: MCU platform-specific code
        reset = 8,

        // ── Shutdown Pins / POR ───────────────────────────────────────────

        /// Creates an internal object for managing a group of shutdown pins. These
        /// pins are set to specific values when the MCU enters shutdown, providing
        /// a hardware-level safety mechanism independent of the main GPIO config.
        ///
        /// See also: [`Self::shutdown_pins_add`], [`Self::shutdown_pins_set`], [`Self::config_por`]
        config_shutdown_pins { oid: u8 } = 9,

        /// Adds a pin to a previously created shutdown pins group. When the MCU shuts
        /// down, this pin will be driven to `value` (0=low, 1=high).
        ///
        /// See also: [`Self::config_shutdown_pins`], [`Self::shutdown_pins_set`]
        shutdown_pins_add { oid: u8, pin: u8, value: u8 } = 10,

        /// Activates the shutdown pin group, causing all pins added via
        /// [`Self::shutdown_pins_add`] to immediately be set to their configured values.
        ///
        /// See also: [`Self::config_shutdown_pins`], [`Self::shutdown_pins_add`]
        shutdown_pins_set { oid: u8 } = 11,

        /// Configures a Power-On Reset (POR) monitoring object. Monitors a pin during
        /// MCU startup to detect a proper power-on reset condition. Works with a
        /// shutdown pins group to implement power monitoring.
        ///
        /// See also: [`Self::config_shutdown_pins`], [`Self::shutdown_pins_add`]
        config_por { oid: u8, shutdown_pins_oid: u8, pin: u8, pull_up: u8, pin_value: u8, sample_ticks: u32, sample_count: u8, rest_ticks: u32 } = 12,

        // ── Stepper / Motion ──────────────────────────────────────────────

        /// Creates an internal stepper object. Configures step and direction pins as
        /// digital outputs. `invert_step` controls step edge: 0=rising, 1=falling,
        /// -1=both edges (if MCU supports `STEPPER_BOTH_EDGE`). `step_pulse_ticks`
        /// sets the minimum step pulse duration. Initializes the stepper's move queue.
        ///
        /// See also: [`Self::queue_step`], [`Self::set_next_step_dir`],
        /// [`Self::reset_step_clock`], [`Self::stepper_get_position`],
        /// [`Self::stepper_stop_on_trigger`]
        /// Source: `src/stepper.c`
        config_stepper { oid: u8, step_pin: u8, dir_pin: u8, invert_step: u8, step_pulse_ticks: u32 } = 13,

        /// Schedules a sequence of steps for a stepper motor. `interval` is MCU clock
        /// ticks between each step. `count` is the number of steps. `add` is a signed
        /// value added to the interval after each step, enabling acceleration/deceleration
        /// trapezoids. Steps are appended to a per-stepper FIFO queue.
        ///
        /// See also: [`Self::config_stepper`], [`Self::set_next_step_dir`]
        /// Source: `src/stepper.c`
        queue_step { oid: u8, interval: u32, count: u16, add: i16 } = 14,

        /// Sets the direction of the next queued step sequence. When the next
        /// [`Self::queue_step`] is processed, if the direction differs, the MCU
        /// toggles the direction pin before issuing steps.
        ///
        /// See also: [`Self::queue_step`], [`Self::config_stepper`]
        /// Source: `src/stepper.c`
        set_next_step_dir { oid: u8, dir: u8 } = 15,

        /// Resets the stepper's time reference so that the next step is relative to
        /// the specified absolute `clock` time instead of relative to the last scheduled
        /// step. The stepper must not be actively stepping when this is issued.
        ///
        /// See also: [`Self::queue_step`], [`Self::config_stepper`]
        /// Source: `src/stepper.c`
        reset_step_clock { oid: u8, clock: u32 } = 16,

        /// Requests the current position of the stepper: total forward steps minus
        /// total reverse steps, accounting for any in-progress step sequences.
        ///
        /// Response: [`RecvCommand::stepper_position`]
        /// See also: [`Self::config_stepper`], [`Self::queue_step`]
        /// Source: `src/stepper.c`
        stepper_get_position { oid: u8 } = 17,

        /// Registers a stepper to be stopped when a specific trsync event fires.
        /// Used during homing: when the trsync fires (e.g. endstop hit), the stepper's
        /// timer is cancelled, pending steps are cleared, and the direction pin is reset.
        ///
        /// See also: [`Self::config_trsync`], [`Self::trsync_start`], [`Self::endstop_home`]
        /// Source: `src/stepper.c`
        stepper_stop_on_trigger { oid: u8, trsync_oid: u8 } = 18,

        /// Creates a Trigger Synchronization (trsync) object. Coordinates multiple
        /// steppers and sensors to respond to a trigger event simultaneously. Supports
        /// periodic status reporting and timeout-based expiration. Used during homing.
        ///
        /// See also: [`Self::trsync_start`], [`Self::trsync_set_timeout`],
        /// [`Self::trsync_trigger`], [`Self::stepper_stop_on_trigger`]
        /// Source: `src/trsync.c`
        config_trsync { oid: u8 } = 19,

        /// Activates a trsync object. `report_clock`/`report_ticks` configure periodic
        /// [`RecvCommand::trsync_state`] reports (0 disables). `expire_reason` is the
        /// reason code used if the timeout expires.
        ///
        /// See also: [`Self::config_trsync`], [`Self::trsync_set_timeout`], [`Self::trsync_trigger`]
        /// Source: `src/trsync.c`
        trsync_start { oid: u8, report_clock: u32, report_ticks: u32, expire_reason: u8 } = 20,

        /// Sets an expiration timeout for an active trsync. If not triggered by `clock`,
        /// it fires automatically with the `expire_reason` from [`Self::trsync_start`].
        /// Used during homing to prevent indefinite waits.
        ///
        /// See also: [`Self::trsync_start`], [`Self::trsync_trigger`]
        /// Source: `src/trsync.c`
        trsync_set_timeout { oid: u8, clock: u32 } = 21,

        /// Manually fires a trsync trigger with the specified reason code. Immediately
        /// triggers all registered signal handlers (stopping linked steppers), cancels
        /// report/expiry timers, and sends a [`RecvCommand::trsync_state`] response.
        ///
        /// See also: [`Self::trsync_start`], [`Self::stepper_stop_on_trigger`]
        /// Source: `src/trsync.c`
        trsync_trigger { oid: u8, reason: u8 } = 22,

        // ── Digital Output ────────────────────────────────────────────────

        /// Creates a digital output object for a GPIO pin. The pin is set to `value`
        /// immediately. `default_value` is applied on shutdown. `max_duration` is a
        /// safety timeout: if non-zero, the host must schedule a new update within
        /// `max_duration` ticks or the MCU shuts down (prevents runaway heaters).
        ///
        /// See also: [`Self::update_digital_out`], [`Self::queue_digital_out`],
        /// [`Self::set_digital_out_pwm_cycle`], [`Self::set_digital_out`]
        /// Source: `src/gpiocmds.c`
        config_digital_out { oid: u8, pin: u32, value: u8, default_value: u8, max_duration: u32 } = 23,

        /// Immediately configures a pin as a digital output and sets it to `value`.
        /// Runs immediately without requiring prior configuration. Useful for setting
        /// initial pin states before the MCU is fully configured.
        ///
        /// See also: [`Self::config_digital_out`]
        /// Source: `src/gpiocmds.c`
        set_digital_out { pin: u32, value: u8 } = 24,

        /// Immediately updates a previously configured digital output pin to `value`,
        /// cancelling any pending queued updates. If `max_duration` was configured and
        /// the pin is set to a non-default value, a timeout timer is started.
        ///
        /// See also: [`Self::config_digital_out`], [`Self::queue_digital_out`]
        /// Source: `src/gpiocmds.c`
        update_digital_out { oid: u8, value: u8 } = 25,

        /// Configures software PWM for a digital output pin. `cycle_ticks` sets the
        /// PWM period in MCU clock ticks. Should be >= 10ms due to software PWM
        /// implementation. Once set, [`Self::queue_digital_out`] uses `on_ticks` to
        /// specify the on-duration within each cycle.
        ///
        /// See also: [`Self::queue_digital_out`], [`Self::config_digital_out`]
        /// Source: `src/gpiocmds.c`
        set_digital_out_pwm_cycle { oid: u8, cycle_ticks: u32 } = 26,

        /// Schedules a change to a digital output pin at a specific clock time. If
        /// [`Self::set_digital_out_pwm_cycle`] was called, `on_ticks` specifies the
        /// on-duration within each PWM cycle. Multiple updates can be queued for
        /// time-precise GPIO sequencing.
        ///
        /// See also: [`Self::config_digital_out`], [`Self::set_digital_out_pwm_cycle`],
        /// [`Self::update_digital_out`]
        /// Source: `src/gpiocmds.c`
        queue_digital_out { oid: u8, clock: u32, on_ticks: u32 } = 27,

        // ── Endstop ───────────────────────────────────────────────────────

        /// Creates an endstop object and configures the pin as a digital input.
        /// `pull_up` enables the MCU's internal pull-up resistor. Used during
        /// homing operations via [`Self::endstop_home`].
        ///
        /// See also: [`Self::endstop_home`], [`Self::endstop_query_state`]
        /// Source: `src/endstop.c`
        config_endstop { oid: u8, pin: u8, pull_up: u8 } = 28,

        /// Initiates endstop sampling for a homing operation. The MCU periodically
        /// samples the pin every `rest_ticks` and checks if it matches `pin_value`.
        /// On match, it verifies with `sample_count` additional samples spaced
        /// `sample_ticks` apart (debouncing). On confirmed trigger, it fires the
        /// linked trsync to stop associated steppers. `sample_count=0` disables
        /// endstop checking.
        ///
        /// See also: [`Self::config_endstop`], [`Self::stepper_stop_on_trigger`],
        /// [`Self::trsync_start`]
        /// Source: `src/endstop.c`
        endstop_home { oid: u8, clock: u32, sample_ticks: u32, sample_count: u8, rest_ticks: u32, pin_value: u8, trsync_oid: u8, trigger_reason: u8 } = 29,

        /// Queries the current state of an endstop: whether it is in homing mode,
        /// the next expected clock time, and the current pin value.
        ///
        /// Response: [`RecvCommand::endstop_state`]
        /// See also: [`Self::endstop_home`], [`Self::config_endstop`]
        /// Source: `src/endstop.c`
        endstop_query_state { oid: u8 } = 30,

        // ── Buttons ───────────────────────────────────────────────────────

        /// Creates a buttons monitoring object. `button_count` specifies the maximum
        /// number of buttons (up to 8). Buttons are individually added with
        /// [`Self::buttons_add`]. The MCU periodically reads all button pins,
        /// performs debouncing, and reports state changes.
        ///
        /// See also: [`Self::buttons_add`], [`Self::buttons_query`], [`Self::buttons_ack`]
        /// Source: `src/buttons.c`
        config_buttons { oid: u8, button_count: u8 } = 31,

        /// Adds a button pin to a buttons object at the specified position (0-indexed).
        /// Must be called after [`Self::config_buttons`] and before [`Self::buttons_query`].
        ///
        /// See also: [`Self::config_buttons`], [`Self::buttons_query`]
        /// Source: `src/buttons.c`
        buttons_add { oid: u8, pos: u8, pin: u32, pull_up: u8 } = 32,

        /// Acknowledges receipt of button state reports. `count` indicates how many
        /// reports the host has received. Clears those from the MCU's buffer and
        /// resets the retransmit timer.
        ///
        /// See also: [`Self::buttons_query`]
        /// Source: `src/buttons.c`
        buttons_ack { oid: u8, count: u8 } = 33,

        /// Starts periodic button polling. MCU reads all configured button pins every
        /// `rest_ticks` starting at `clock`. State changes (after debouncing) are
        /// reported via [`RecvCommand::buttons_state`]. `retransmit_count` controls
        /// how many intervals before retransmitting unacknowledged state. `invert`
        /// sets the initial baseline for all buttons. `rest_ticks=0` stops polling.
        ///
        /// See also: [`Self::config_buttons`], [`Self::buttons_add`], [`Self::buttons_ack`]
        /// Source: `src/buttons.c`
        buttons_query { oid: u8, clock: u32, rest_ticks: u32, retransmit_count: u8, invert: u8 } = 34,

        // ── Analog / Counter ──────────────────────────────────────────────

        /// Configures a GPIO pin for analog input (ADC) sampling. Creates an internal
        /// analog input object that can be periodically sampled with [`Self::query_analog_in`].
        ///
        /// See also: [`Self::query_analog_in`]
        /// Source: `src/adccmds.c`
        config_analog_in { oid: u8, pin: u32 } = 35,

        /// Sets up recurring analog input sampling starting at `clock`. The MCU
        /// over-samples `sample_count` times with `sample_ticks` delay, then waits
        /// `rest_ticks` before the next batch. `min_value`/`max_value` define a
        /// safety range; if exceeded more than `range_check_count` consecutive times
        /// the MCU shuts down. `sample_count=0` stops sampling.
        ///
        /// Response: [`RecvCommand::analog_in_state`]
        /// See also: [`Self::config_analog_in`]
        /// Source: `src/adccmds.c`
        query_analog_in { oid: u8, clock: u32, sample_ticks: u32, sample_count: u8, rest_ticks: u32, min_value: u16, max_value: u16, range_check_count: u8 } = 36,

        /// Configures a GPIO pin as an edge/pulse counter. The MCU polls the pin and
        /// counts rising and falling edges. `pull_up` enables the internal pull-up.
        ///
        /// See also: [`Self::query_counter`]
        /// Source: `src/pulse_counter.c`
        config_counter { oid: u8, pin: u32, pull_up: u8 } = 37,

        /// Starts periodic edge counting. MCU polls the pin every `poll_ticks` and
        /// sends a [`RecvCommand::counter_state`] report every `sample_ticks`.
        ///
        /// See also: [`Self::config_counter`]
        /// Source: `src/pulse_counter.c`
        query_counter { oid: u8, clock: u32, poll_ticks: u32, sample_ticks: u32 } = 38,

        // ── SPI ───────────────────────────────────────────────────────────

        /// Creates an SPI device object with a chip select (CS) pin. CS polarity
        /// is controlled by `cs_active_high`. Must call [`Self::spi_set_bus`] or
        /// [`Self::spi_set_software_bus`] after this to complete configuration.
        ///
        /// See also: [`Self::spi_set_bus`], [`Self::spi_set_software_bus`],
        /// [`Self::spi_transfer`], [`Self::spi_send`]
        /// Source: `src/spicmds.c`
        config_spi { oid: u8, pin: u32, cs_active_high: u8 } = 39,

        /// Creates an SPI device object without a chip select pin. For devices that
        /// have no CS line or where CS is managed externally.
        ///
        /// See also: [`Self::spi_set_bus`], [`Self::spi_set_software_bus`]
        /// Source: `src/spicmds.c`
        config_spi_without_cs { oid: u8 } = 40,

        /// Configures an SPI device to use a hardware SPI bus. `spi_bus` identifies
        /// the SPI peripheral (MCU-specific). `mode` is the SPI mode (0-3). `rate`
        /// is the clock rate in Hz. Must be called after [`Self::config_spi`] or
        /// [`Self::config_spi_without_cs`].
        ///
        /// See also: [`Self::spi_set_software_bus`]
        /// Source: `src/spicmds.c`
        spi_set_bus { oid: u8, spi_bus: u32, mode: u32, rate: u32 } = 41,

        /// Configures an SPI device to use bit-banged (software) SPI. Specifies
        /// MISO, MOSI, and SCLK GPIO pins, the SPI mode (0-3), and target rate.
        ///
        /// See also: [`Self::spi_set_bus`]
        /// Source: `src/spi_software.c`
        spi_set_software_bus { oid: u8, miso_pin: u32, mosi_pin: u32, sclk_pin: u32, mode: u32, rate: u32 } = 42,

        /// Sends data over SPI without reading the response. CS is asserted during
        /// transfer and deasserted afterward. Data on MISO is discarded.
        ///
        /// See also: [`Self::spi_transfer`], [`Self::config_spi`]
        /// Source: `src/spicmds.c`
        spi_send { oid: u8, data: Vec<u8> } = 43,

        /// Performs a full-duplex SPI transfer. Data is simultaneously sent on MOSI
        /// and received on MISO. The `data` buffer is overwritten with received data.
        ///
        /// Response: [`RecvCommand::spi_transfer_response`]
        /// See also: [`Self::spi_send`], [`Self::config_spi`]
        /// Source: `src/spicmds.c`
        spi_transfer { oid: u8, data: Vec<u8> } = 44,

        /// Registers an SPI message to be automatically sent when the MCU enters
        /// shutdown. Used for safety-critical devices (e.g. disabling a motor driver).
        ///
        /// See also: [`Self::config_spi`], [`Self::spi_send`]
        /// Source: `src/spicmds.c`
        config_spi_shutdown { oid: u8, spi_oid: u8, shutdown_msg: Vec<u8> } = 45,

        /// Creates an SPI angle sensor object. `spi_oid` references a configured SPI
        /// device. `spi_angle_type` selects the chip type (A1333, AS5047D, TLE5012B,
        /// MT6816, MT6826S). The SPI device must have a CS pin.
        ///
        /// See also: [`Self::query_spi_angle`], [`Self::spi_angle_transfer`]
        /// Source: `src/sensor_angle.c`
        config_spi_angle { oid: u8, spi_oid: u8, spi_angle_type: u8 } = 46,

        /// Starts or stops periodic angle sensor readings. When `rest_ticks > 0`,
        /// begins sampling at `clock` with the given interval. `time_shift` controls
        /// timestamp bit-shifting for bulk data. `rest_ticks=0` stops measurements.
        /// Data is sent as [`RecvCommand::sensor_bulk_data`].
        ///
        /// See also: [`Self::config_spi_angle`], [`Self::spi_angle_transfer`]
        /// Source: `src/sensor_angle.c`
        query_spi_angle { oid: u8, clock: u32, rest_ticks: u32, time_shift: u8 } = 47,

        /// Performs a single SPI transfer for angle sensor data acquisition. Behavior
        /// varies by chip type (CS toggling, end-of-transfer timing, etc.). Returns
        /// the raw SPI response and precise MCU timestamp.
        ///
        /// Response: [`RecvCommand::spi_angle_transfer_response`]
        /// See also: [`Self::config_spi_angle`], [`Self::query_spi_angle`]
        /// Source: `src/sensor_angle.c`
        spi_angle_transfer { oid: u8, data: Vec<u8> } = 48,

        // ── I2C ───────────────────────────────────────────────────────────

        /// Creates an I2C device object. Must call [`Self::i2c_set_bus`] or
        /// [`Self::i2c_set_software_bus`] after this to complete configuration.
        ///
        /// See also: [`Self::i2c_set_bus`], [`Self::i2c_set_software_bus`]
        /// Source: `src/i2ccmds.c`
        config_i2c { oid: u8 } = 49,

        /// Configures an I2C device to use a hardware I2C bus. `i2c_bus` identifies
        /// the I2C peripheral (MCU-specific). `rate` is the clock rate in Hz.
        /// `address` is the 7-bit device address (bit 7 is masked off).
        ///
        /// See also: [`Self::i2c_set_software_bus`]
        /// Source: `src/i2ccmds.c`
        i2c_set_bus { oid: u8, i2c_bus: u32, rate: u32, address: u32 } = 50,

        /// Configures an I2C device to use bit-banged (software) I2C. Specifies
        /// SCL and SDA GPIO pins, target clock rate, and device address.
        ///
        /// See also: [`Self::i2c_set_bus`]
        /// Source: `src/i2ccmds.c`
        i2c_set_software_bus { oid: u8, scl_pin: u32, sda_pin: u32, rate: u32, address: u32 } = 51,

        /// Writes data to the I2C device. `data` includes the register address
        /// (if applicable) followed by data bytes.
        ///
        /// See also: [`Self::i2c_read`], [`Self::i2c_modify_bits`]
        /// Source: `src/i2ccmds.c`
        i2c_write { oid: u8, data: Vec<u8> } = 52,

        /// Reads data from the I2C device. `reg` specifies the register address(es)
        /// to read from, and `read_len` is the number of bytes to read. Performs a
        /// write-then-read I2C transaction.
        ///
        /// Response: [`RecvCommand::i2c_read_response`]
        /// See also: [`Self::i2c_write`], [`Self::i2c_modify_bits`]
        /// Source: `src/i2ccmds.c`
        i2c_read { oid: u8, reg: Vec<u8>, read_len: u32 } = 53,

        /// Performs a read-modify-write operation on an I2C register. `reg` specifies
        /// the register. `clear_set_bits` is a byte string where odd-indexed bytes are
        /// clear masks and even-indexed bytes are set masks.
        ///
        /// See also: [`Self::i2c_write`], [`Self::i2c_read`]
        /// Source: `src/i2ccmds.c`
        i2c_modify_bits { oid: u8, reg: Vec<u8>, clear_set_bits: Vec<u8> } = 54,

        // ── NeoPixel ──────────────────────────────────────────────────────

        /// Creates a NeoPixel (WS2812-type) LED strip object. `pin` is the data
        /// output GPIO. `data_size` is total data bytes (3 per RGB LED, 4 per RGBW).
        /// `bit_max_ticks` detects hardware IRQ glitches. `reset_min_ticks` is the
        /// minimum reset pulse duration between transmissions (~50us).
        ///
        /// See also: [`Self::neopixel_update`], [`Self::neopixel_send`]
        /// Source: `src/neopixel.c`
        config_neopixel { oid: u8, pin: u32, data_size: u16, bit_max_ticks: u32, reset_min_ticks: u32 } = 55,

        /// Copies LED color data into the MCU's internal buffer at the specified byte
        /// offset. Does not transmit to the LEDs. Multiple calls can be made before
        /// [`Self::neopixel_send`].
        ///
        /// See also: [`Self::neopixel_send`], [`Self::config_neopixel`]
        /// Source: `src/neopixel.c`
        neopixel_update { oid: u8, pos: u16, data: Vec<u8> } = 56,

        /// Transmits the current buffer to NeoPixel LEDs using WS2812 bit-banging.
        /// Waits for the reset period, then sends each bit as a precisely timed
        /// GPIO pulse. Reports failure if a hardware IRQ disrupts timing.
        ///
        /// Response: [`RecvCommand::neopixel_result`]
        /// See also: [`Self::neopixel_update`], [`Self::config_neopixel`]
        /// Source: `src/neopixel.c`
        neopixel_send { oid: u8 } = 57,

        // ── TMC UART ─────────────────────────────────────────────────────

        /// Creates a TMC stepper driver UART object for TMC2208/TMC2209
        /// communication. If `rx_pin == tx_pin`, operates in single-wire mode.
        /// `bit_time` is the UART bit duration in MCU clock cycles. Uses enhanced
        /// baud detection via sync nibble timing.
        ///
        /// See also: [`Self::tmcuart_send`]
        /// Source: `src/tmcuart.c`
        config_tmcuart { oid: u8, rx_pin: u32, pull_up: u8, tx_pin: u32, bit_time: u32 } = 58,

        /// Sends a UART message to a TMC stepper driver. `write` is the data to
        /// transmit. If `read > 0`, switches to receive mode after transmission and
        /// captures the response.
        ///
        /// Response: [`RecvCommand::tmcuart_response`]
        /// See also: [`Self::config_tmcuart`]
        /// Source: `src/tmcuart.c`
        tmcuart_send { oid: u8, write: Vec<u8>, read: u8 } = 59,

        // ── Debug ─────────────────────────────────────────────────────────

        /// A no-operation command. Does absolutely nothing. Used for timing
        /// measurements and communication verification. Can be invoked during shutdown.
        ///
        /// See also: [`Self::debug_ping`]
        /// Source: `src/debugcmds.c`
        debug_nop = 60,

        /// Echoes the provided data back as a [`RecvCommand::pong`] response. Used
        /// for round-trip communication testing and latency measurement. Can be
        /// invoked during shutdown.
        ///
        /// Response: [`RecvCommand::pong`]
        /// Source: `src/debugcmds.c`
        debug_ping { data: Vec<u8> } = 61,

        /// Writes a value to an arbitrary memory address. `order` specifies access
        /// width: 0=byte, 1=16-bit, 2=32-bit. Extremely dangerous -- low-level
        /// hardware debugging only. Can be invoked during shutdown.
        ///
        /// See also: [`Self::debug_read`]
        /// Source: `src/debugcmds.c`
        debug_write { order: u8, addr: u32, val: u32 } = 62,

        /// Reads a value from an arbitrary memory address. `order` specifies access
        /// width: 0=byte, 1=16-bit, 2=32-bit. Low-level hardware debugging only.
        /// Can be invoked during shutdown.
        ///
        /// Response: [`RecvCommand::debug_result`]
        /// See also: [`Self::debug_write`]
        /// Source: `src/debugcmds.c`
        debug_read { order: u8, addr: u32 } = 63,

        // ── Displays ──────────────────────────────────────────────────────

        /// Configures an HD44780-compatible character LCD using 4-bit parallel
        /// interface. Sets up RS, E, and D4-D7 pins as digital outputs.
        /// `delay_ticks` specifies the minimum inter-command delay.
        ///
        /// See also: [`Self::hd44780_send_cmds`], [`Self::hd44780_send_data`]
        /// Source: `src/lcd_hd44780.c`
        config_hd44780 { oid: u8, rs_pin: u32, e_pin: u32, d4_pin: u32, d5_pin: u32, d6_pin: u32, d7_pin: u32, delay_ticks: u32 } = 64,

        /// Sends command bytes to the HD44780 LCD. RS pin is set low (command mode).
        /// Commands include display initialization, cursor positioning, and control.
        ///
        /// See also: [`Self::hd44780_send_data`], [`Self::config_hd44780`]
        /// Source: `src/lcd_hd44780.c`
        hd44780_send_cmds { oid: u8, cmds: Vec<u8> } = 65,

        /// Sends data bytes (character content) to the HD44780 LCD. RS pin is set
        /// high (data mode). Writes characters at the current cursor position.
        ///
        /// See also: [`Self::hd44780_send_cmds`], [`Self::config_hd44780`]
        /// Source: `src/lcd_hd44780.c`
        hd44780_send_data { oid: u8, data: Vec<u8> } = 66,

        /// Configures an ST7920 graphical LCD using serial (SPI-like) interface.
        /// `cs_pin` is chip select (active high), `sclk_pin` is serial clock,
        /// `sid_pin` is serial data. Delay values are calibrated for strict-timing
        /// MCUs.
        ///
        /// See also: [`Self::st7920_send_cmds`], [`Self::st7920_send_data`]
        /// Source: `src/lcd_st7920.c`
        config_st7920 { oid: u8, cs_pin: u32, sclk_pin: u32, sid_pin: u32, sync_delay_ticks: u32, cmd_delay_ticks: u32 } = 67,

        /// Sends command bytes to the ST7920 LCD. A sync byte (0xF8) is sent first,
        /// followed by command data transmitted serially.
        ///
        /// See also: [`Self::st7920_send_data`], [`Self::config_st7920`]
        /// Source: `src/lcd_st7920.c`
        st7920_send_cmds { oid: u8, cmds: Vec<u8> } = 68,

        /// Sends data bytes to the ST7920 LCD. A sync byte (0xFA) is sent first,
        /// followed by data payload. Writes pixel/character data to display RAM.
        ///
        /// See also: [`Self::st7920_send_cmds`], [`Self::config_st7920`]
        /// Source: `src/lcd_st7920.c`
        st7920_send_data { oid: u8, data: Vec<u8> } = 69,

        // ── Load Cell / Strain Gauge ──────────────────────────────────────

        /// Creates a load cell endstop object. Combines probe endstop functionality
        /// with analog sensor triggering. References an SOS filter object for digital
        /// signal processing of load cell readings. Detects when a sudden force
        /// indicates a collision (e.g. nozzle touching the bed).
        ///
        /// See also: [`Self::load_cell_endstop_home`], [`Self::config_sos_filter`],
        /// [`Self::set_range_load_cell_endstop`]
        config_load_cell_endstop { oid: u8, sos_filter_oid: u8 } = 70,

        /// Creates a load cell endstop object with an additional GPIO pin for
        /// auxiliary signaling or additional endstop functionality.
        ///
        /// See also: [`Self::config_load_cell_endstop`], [`Self::load_cell_endstop_home`]
        config_load_cell_endstop_with_pin { oid: u8, sos_filter_oid: u8, pin: u32 } = 71,

        /// Initiates a homing/probing operation using the load cell. Links to a
        /// trsync object for stopping steppers on trigger or error. Monitors filtered
        /// load cell readings and triggers when the configured threshold is exceeded.
        ///
        /// Response: [`RecvCommand::load_cell_endstop_state`]
        /// See also: [`Self::config_load_cell_endstop`], [`Self::trsync_start`],
        /// [`Self::set_range_load_cell_endstop`]
        load_cell_endstop_home { oid: u8, trsync_oid: u8, trigger_reason: u8, error_reason: u8, clock: u32, sample_count: u8, rest_ticks: u32, timeout: u32 } = 72,

        /// Queries the current state of a load cell endstop: homing status, trigger
        /// detection, timing information, and current raw sample value.
        ///
        /// Response: [`RecvCommand::load_cell_endstop_state`]
        /// See also: [`Self::load_cell_endstop_home`], [`Self::config_load_cell_endstop`]
        load_cell_endstop_query_state { oid: u8 } = 73,

        /// Sets measurement range and trigger parameters for the load cell endstop.
        /// `safety_counts_min/max` define ADC limits (outside = shutdown). `tare_counts`
        /// is the zero-offset. `trigger_grams` is the force threshold. `grams_per_count`
        /// converts raw ADC counts to grams.
        ///
        /// See also: [`Self::load_cell_endstop_home`], [`Self::config_load_cell_endstop`]
        set_range_load_cell_endstop { oid: u8, safety_counts_min: i32, safety_counts_max: i32, tare_counts: i32, trigger_grams: u32, grams_per_count: i32 } = 74,

        /// Attaches an HX71x (HX711/HX717) sensor to a load cell endstop object.
        /// Routes the HX71x readings to the load cell endstop for force-based
        /// triggering.
        ///
        /// See also: [`Self::config_hx71x`], [`Self::config_load_cell_endstop`]
        attach_endstop_hx71x { oid: u8, load_cell_endstop_oid: u8 } = 75,

        /// Attaches a CS123x sensor to a load cell endstop object. Routes the
        /// CS123x readings to the load cell endstop for force-based triggering.
        ///
        /// See also: [`Self::config_cs123x`], [`Self::config_load_cell_endstop`]
        attach_endstop_cs123x { oid: u8, load_cell_endstop_oid: u8 } = 76,

        /// Creates a Second Order Sections (SOS) digital filter object for real-time
        /// signal conditioning of sensor data (high-pass, low-pass, notch filtering).
        /// Uses fixed-point arithmetic for efficient MCU operation.
        ///
        /// See also: [`Self::config_sos_filter_section`]
        /// Source: `src/sos_filter.c`
        config_sos_filter { oid: u8 } = 77,

        /// Configures one section of an SOS filter. `section_idx` is the 0-indexed
        /// section number. `n_sections` is the total section count. `sos0`-`sos4`
        /// are the five transfer function coefficients (b0, b1, b2, a1, a2) in
        /// fixed-point Q-format. Each section implements a second-order IIR filter.
        ///
        /// See also: [`Self::config_sos_filter`]
        /// Source: `src/sos_filter.c`
        config_sos_filter_section { oid: u8, n_sections: u8, section_idx: u8, sos0: i32, sos1: i32, sos2: i32, sos3: i32, sos4: i32 } = 78,

        // ── Sensor: HX71x ────────────────────────────────────────────────

        /// Creates an HX711 or HX717 ADC object for reading load cells. `gain_channel`
        /// selects gain and channel: 1=ChA 128x (HX711 default), 2=ChA 64x,
        /// 3=ChB 32x, 4=ChA 128x high-rate (HX717). Communication is bit-banged.
        ///
        /// See also: [`Self::query_hx71x`], [`Self::query_hx71x_status`]
        /// Source: `src/sensor_hx71x.c`
        config_hx71x { oid: u8, gain_channel: u8, dout_pin: u32, sclk_pin: u32 } = 79,

        /// Starts or stops periodic HX71x data capture. When `rest_ticks > 0`,
        /// polls DOUT and bit-bangs reads of 24+gain bits. Results are sent as
        /// [`RecvCommand::sensor_bulk_data`]. `rest_ticks=0` puts HX71x into
        /// power-down mode.
        ///
        /// See also: [`Self::config_hx71x`], [`Self::query_hx71x_status`]
        /// Source: `src/sensor_hx71x.c`
        query_hx71x { oid: u8, rest_ticks: u32 } = 80,

        /// Queries HX71x sensor status without reading new data. Reports timing,
        /// buffered bytes, and whether new data is ready.
        ///
        /// Response: [`RecvCommand::sensor_bulk_status`]
        /// See also: [`Self::query_hx71x`], [`Self::config_hx71x`]
        /// Source: `src/sensor_hx71x.c`
        query_hx71x_status { oid: u8 } = 81,

        // ── Sensor: CS123x ───────────────────────────────────────────────

        /// Creates a CS123x ADC object (CS1232/CS1237/CS1238) for load cell reading.
        /// Similar to HX71x but with different protocol and gain/resolution options.
        ///
        /// See also: [`Self::query_cs123x`], [`Self::query_cs123x_status`]
        config_cs123x { oid: u8, dout_pin: u32, sclk_pin: u32 } = 82,

        /// Performs a single read from the CS123x ADC. Generates clock pulses to
        /// shift out the conversion result.
        ///
        /// Response: [`RecvCommand::cs123x_read_response`]
        /// See also: [`Self::config_cs123x`], [`Self::cs123x_write`]
        cs123x_read { oid: u8 } = 83,

        /// Writes configuration to the CS123x chip. Accepts configuration during a
        /// specific time window after power-up. The `config` byte sets gain and
        /// reference settings.
        ///
        /// See also: [`Self::config_cs123x`], [`Self::cs123x_read`]
        cs123x_write { oid: u8, config: u8 } = 84,

        /// Starts or stops periodic CS123x data capture. When `rest_ticks > 0`,
        /// polls and reads at the specified interval. Results sent as
        /// [`RecvCommand::sensor_bulk_data`]. `rest_ticks=0` stops.
        ///
        /// See also: [`Self::config_cs123x`], [`Self::query_cs123x_status`]
        query_cs123x { oid: u8, rest_ticks: u32 } = 85,

        /// Queries CS123x sensor status: timing, buffered data count, and overflow.
        ///
        /// Response: [`RecvCommand::sensor_bulk_status`]
        /// See also: [`Self::query_cs123x`], [`Self::config_cs123x`]
        query_cs123x_status { oid: u8 } = 86,

        // ── Sensor: ADS1220 ──────────────────────────────────────────────

        /// Creates an ADS1220 24-bit delta-sigma ADC object. `spi_oid` references
        /// a configured SPI device. `data_ready_pin` signals when a conversion is
        /// complete (active low). The MCU polls this pin for new data.
        ///
        /// See also: [`Self::query_ads1220`], [`Self::query_ads1220_status`]
        /// Source: `src/sensor_ads1220.c`
        config_ads1220 { oid: u8, spi_oid: u8, data_ready_pin: u32 } = 87,

        /// Starts or stops periodic ADS1220 data capture. When `rest_ticks > 0`,
        /// polls data_ready and reads 24-bit results via SPI. Results sent as
        /// [`RecvCommand::sensor_bulk_data`]. `rest_ticks=0` stops.
        ///
        /// See also: [`Self::config_ads1220`], [`Self::query_ads1220_status`]
        /// Source: `src/sensor_ads1220.c`
        query_ads1220 { oid: u8, rest_ticks: u32 } = 88,

        /// Queries ADS1220 status: whether data_ready is asserted, timing, and
        /// buffer status.
        ///
        /// Response: [`RecvCommand::sensor_bulk_status`]
        /// See also: [`Self::query_ads1220`], [`Self::config_ads1220`]
        /// Source: `src/sensor_ads1220.c`
        query_ads1220_status { oid: u8 } = 89,

        // ── Sensor: Thermocouple ─────────────────────────────────────────

        /// Creates a thermocouple reader object. `spi_oid` references a configured
        /// SPI device. `thermocouple_type` selects the chip: MAX31855, MAX31856,
        /// MAX31865, or MAX6675. Each has different SPI protocols and data formats.
        ///
        /// See also: [`Self::query_thermocouple`]
        /// Source: `src/thermocouple.c`
        config_thermocouple { oid: u8, spi_oid: u8, thermocouple_type: u8 } = 90,

        /// Starts or stops periodic thermocouple readings. When `rest_ticks > 0`,
        /// reads at the given interval starting at `clock`. `min_value`/`max_value`
        /// define valid range; after `max_invalid_count` consecutive invalid readings
        /// the MCU shuts down. `rest_ticks=0` stops.
        ///
        /// Response: [`RecvCommand::thermocouple_result`]
        /// See also: [`Self::config_thermocouple`]
        /// Source: `src/thermocouple.c`
        query_thermocouple { oid: u8, clock: u32, rest_ticks: u32, min_value: u32, max_value: u32, max_invalid_count: u8 } = 91,

        // ── Sensor: LDC1612 ──────────────────────────────────────────────

        /// Creates an LDC1612 inductive/eddy current sensor object. `i2c_oid`
        /// references a configured I2C device. Measures coil resonant frequency
        /// changes for contactless bed probing and filament monitoring.
        ///
        /// See also: [`Self::query_ldc1612`], [`Self::query_status_ldc1612`]
        /// Source: `src/sensor_ldc1612.c`
        config_ldc1612 { oid: u8, i2c_oid: u8 } = 92,

        /// Creates an LDC1612 object with an interrupt output pin (INTB). Uses the
        /// INTB pin to detect when new data is available rather than polling over I2C.
        ///
        /// See also: [`Self::config_ldc1612`], [`Self::query_ldc1612`]
        /// Source: `src/sensor_ldc1612.c`
        config_ldc1612_with_intb { oid: u8, i2c_oid: u8, intb_pin: u8 } = 93,

        /// Configures the LDC1612 for use as a homing/probing trigger. Sets the
        /// threshold value that, when crossed, fires the linked trsync to stop
        /// steppers. Error conditions (I2C failures, out-of-range data) trigger
        /// with `error_reason`.
        ///
        /// See also: [`Self::query_ldc1612`], [`Self::config_ldc1612`], [`Self::trsync_start`]
        ldc1612_setup_home { oid: u8, clock: u32, threshold: u32, trsync_oid: u8, trigger_reason: u8, error_reason: u8 } = 94,

        /// Starts or stops periodic LDC1612 data capture. When `rest_ticks > 0`,
        /// polls at the given interval (or waits for INTB if configured). Reads
        /// 28-bit conversion results as [`RecvCommand::sensor_bulk_data`].
        /// `rest_ticks=0` stops.
        ///
        /// See also: [`Self::config_ldc1612`], [`Self::query_status_ldc1612`]
        /// Source: `src/sensor_ldc1612.c`
        query_ldc1612 { oid: u8, rest_ticks: u32 } = 95,

        /// Queries whether the LDC1612 is in homing mode and the clock of the
        /// last trigger event.
        ///
        /// Response: [`RecvCommand::ldc1612_home_state`]
        /// See also: [`Self::ldc1612_setup_home`], [`Self::query_ldc1612`]
        query_ldc1612_home_state { oid: u8 } = 96,

        /// Queries LDC1612 sensor status: checks INTB pin state or reads the
        /// status register over I2C for new data availability.
        ///
        /// Response: [`RecvCommand::sensor_bulk_status`]
        /// See also: [`Self::query_ldc1612`], [`Self::config_ldc1612`]
        /// Source: `src/sensor_ldc1612.c`
        query_status_ldc1612 { oid: u8 } = 97,

        // ── Sensor: Accelerometer / Gyro ──────────────────────────────────

        /// Creates an ADXL345 accelerometer object. `spi_oid` references a
        /// configured SPI device. Used for Klipper's input shaper resonance
        /// measurement.
        ///
        /// See also: [`Self::query_adxl345`], [`Self::query_adxl345_status`]
        /// Source: `src/sensor_adxl345.c`
        config_adxl345 { oid: u8, spi_oid: u8 } = 98,

        /// Starts or stops periodic ADXL345 data capture. Reads accelerometer FIFO
        /// via SPI at the specified interval. Each reading includes X/Y/Z values
        /// packed into 5 bytes. Data sent as [`RecvCommand::sensor_bulk_data`].
        /// `rest_ticks=0` stops.
        ///
        /// See also: [`Self::config_adxl345`], [`Self::query_adxl345_status`]
        /// Source: `src/sensor_adxl345.c`
        query_adxl345 { oid: u8, rest_ticks: u32 } = 99,

        /// Queries ADXL345 FIFO status: entry count, timing, buffered data, and
        /// overflow detection.
        ///
        /// Response: [`RecvCommand::sensor_bulk_status`]
        /// See also: [`Self::query_adxl345`], [`Self::config_adxl345`]
        /// Source: `src/sensor_adxl345.c`
        query_adxl345_status { oid: u8 } = 100,

        /// Creates a LIS2DW or LIS3DH accelerometer object. `spi_oid` references a
        /// configured SPI device. Used for input shaper resonance measurement.
        ///
        /// See also: [`Self::query_lis2dw`], [`Self::query_lis2dw_status`]
        /// Source: `src/sensor_lis2dw.c`
        config_lis2dw { oid: u8, spi_oid: u8 } = 101,

        /// Starts or stops periodic LIS2DW/LIS3DH data capture. Reads 8 samples at
        /// a time from the FIFO (48 bytes). Data sent as
        /// [`RecvCommand::sensor_bulk_data`]. `rest_ticks=0` stops.
        ///
        /// See also: [`Self::config_lis2dw`], [`Self::query_lis2dw_status`]
        /// Source: `src/sensor_lis2dw.c`
        query_lis2dw { oid: u8, rest_ticks: u32 } = 102,

        /// Queries LIS2DW/LIS3DH FIFO status: sample count, timing, buffered data,
        /// and overflow detection.
        ///
        /// Response: [`RecvCommand::sensor_bulk_status`]
        /// See also: [`Self::query_lis2dw`], [`Self::config_lis2dw`]
        /// Source: `src/sensor_lis2dw.c`
        query_lis2dw_status { oid: u8 } = 103,

        /// Creates an MPU9250 accelerometer/gyroscope object. `i2c_oid` references
        /// a configured I2C device. Primarily used for accelerometer-based input
        /// shaper calibration.
        ///
        /// See also: [`Self::query_mpu9250`], [`Self::query_mpu9250_status`]
        /// Source: `src/sensor_mpu9250.c`
        config_mpu9250 { oid: u8, i2c_oid: u8 } = 104,

        /// Starts or stops periodic MPU9250 data capture. Reads the sensor's FIFO
        /// in 48-byte blocks (8 samples x 6 bytes: X/Y/Z). Data sent as
        /// [`RecvCommand::sensor_bulk_data`]. `rest_ticks=0` stops.
        ///
        /// See also: [`Self::config_mpu9250`], [`Self::query_mpu9250_status`]
        /// Source: `src/sensor_mpu9250.c`
        query_mpu9250 { oid: u8, rest_ticks: u32 } = 105,

        /// Queries MPU9250 FIFO status: overflow flag, FIFO byte count, timing,
        /// and buffer status.
        ///
        /// Response: [`RecvCommand::sensor_bulk_status`]
        /// See also: [`Self::query_mpu9250`], [`Self::config_mpu9250`]
        /// Source: `src/sensor_mpu9250.c`
        query_mpu9250_status { oid: u8 } = 106,
    }
}

command! {
    #[derive(Clone, Debug)]
    #[allow(non_camel_case_types)]
    pub enum RecvCommand {

        // ── Identification ────────────────────────────────────────────────

        /// Response to [`SendCommand::identify`]. Contains a chunk of the MCU's data
        /// dictionary (compressed JSON) starting at `offset`. The host collects all
        /// chunks, decompresses, and parses the JSON to obtain command/response
        /// definitions, enumerations, constants, and firmware version.
        ///
        /// Source: `src/basecmd.c`
        identify_response { offset: u32, data: Vec<u8> } = 0,

        // ── Core / System ─────────────────────────────────────────────────

        /// Response to [`SendCommand::get_clock`]. Contains the current 32-bit MCU
        /// clock value. Used by the host's clock synchronization system to track MCU
        /// time and estimate clock drift.
        ///
        /// Source: `src/basecmd.c`
        clock { clock: u32 } = 1,

        /// Response to [`SendCommand::get_uptime`]. `high` is the upper 32 bits of
        /// the 64-bit uptime counter (increments when the low 32 bits wrap). `clock`
        /// is the current low 32-bit clock value. Together they form a 64-bit uptime
        /// measurement used to detect MCU resets.
        ///
        /// Source: `src/basecmd.c`
        uptime { high: u32, clock: u32 } = 2,

        /// Response to [`SendCommand::get_config`]. `is_config` indicates whether the
        /// MCU is configured. `crc` is the stored CRC (for configuration matching).
        /// `is_shutdown` indicates shutdown state. `move_count` is the number of
        /// available move queue entries.
        ///
        /// Source: `src/basecmd.c`
        config { is_config: u8, crc: u32, is_shutdown: u8, move_count: u16 } = 3,

        /// Response to [`SendCommand::debug_ping`]. Echoes back the data from the
        /// ping command. Used for round-trip latency measurement.
        ///
        /// Source: `src/debugcmds.c`
        pong { data: Vec<u8> } = 4,

        /// Sent by the MCU once during initialization, just before it begins
        /// processing commands. Indicates firmware has booted and is ready. The host
        /// uses this to detect MCU restarts.
        ///
        /// Source: `src/sched.c`
        starting = 5,

        // ── Shutdown ──────────────────────────────────────────────────────

        /// Sent when the MCU enters or is already in shutdown state. `static_string_id`
        /// identifies the shutdown reason string from the MCU's compiled-in string
        /// table. Sent whenever the host queries the MCU while in shutdown.
        ///
        /// Source: `src/sched.c`
        is_shutdown { static_string_id: u16 } = 6,

        /// Sent at the moment the MCU enters shutdown. `clock` is the MCU time when
        /// the shutdown occurred. `static_string_id` identifies the shutdown reason.
        /// All configured output pins revert to their default values immediately.
        ///
        /// Source: `src/sched.c`
        shutdown { clock: u32, static_string_id: u16 } = 7,

        // ── Stepper / Motion ──────────────────────────────────────────────

        /// Response to [`SendCommand::stepper_get_position`]. Reports the signed step
        /// position (forward steps minus reverse steps). Used for position verification
        /// and endstop offset calibration.
        ///
        /// Source: `src/stepper.c`
        stepper_position { oid: u8, pos: i32 } = 8,

        /// Periodic status report (or final trigger report) from a trsync object.
        /// `can_trigger` indicates whether the trsync is still armed. `trigger_reason`
        /// contains the reason code if triggered. Sent periodically and once on
        /// trigger/expiration.
        ///
        /// Source: `src/trsync.c`
        trsync_state { oid: u8, can_trigger: u8, trigger_reason: u8, clock: u32 } = 9,

        // ── Endstop ───────────────────────────────────────────────────────

        /// Response to [`SendCommand::endstop_query_state`]. `homing` indicates
        /// whether the endstop is actively sampling. `next_clock` is the next
        /// scheduled sample time. `pin_value` is the current logic level.
        ///
        /// Source: `src/endstop.c`
        endstop_state { oid: u8, homing: u8, next_clock: u32, pin_value: u8 } = 10,

        // ── Buttons ───────────────────────────────────────────────────────

        /// Reports debounced button state changes. `ack_count` is the number of
        /// reports the MCU knows the host has received. `state` is a byte string
        /// where each byte represents a snapshot of all button states (bit 0 = button 0)
        /// at each detected change. Multiple snapshots may be included if the host
        /// hasn't acknowledged previous reports.
        ///
        /// Source: `src/buttons.c`
        buttons_state { oid: u8, ack_count: u8, state: Vec<u8> } = 11,

        // ── Analog / Counter ──────────────────────────────────────────────

        /// Reports accumulated analog input samples. `next_clock` is the time of the
        /// next sample batch. `value` contains the raw 16-bit sample value.
        ///
        /// Source: `src/adccmds.c`
        analog_in_state { oid: u8, next_clock: u32, value: u16 } = 12,

        /// Reports edge counter status. `count` is the total edge count since
        /// counting started. `count_clock` is the timestamp of the most recent
        /// detected edge. `next_clock` is when the next sample will be reported.
        ///
        /// Source: `src/pulse_counter.c`
        counter_state { oid: u8, next_clock: u32, count: u32, count_clock: u32 } = 13,

        // ── SPI ───────────────────────────────────────────────────────────

        /// Response to [`SendCommand::spi_transfer`]. Contains the data received on
        /// MISO during the SPI transfer (the sent buffer overwritten with received data).
        ///
        /// Source: `src/spicmds.c`
        spi_transfer_response { oid: u8, response: Vec<u8> } = 14,

        /// Response to [`SendCommand::spi_angle_transfer`]. `clock` is the precise
        /// MCU timestamp of the measurement (critical for angle-based step generation).
        /// `response` contains the raw SPI data from the angle sensor.
        ///
        /// Source: `src/sensor_angle.c`
        spi_angle_transfer_response { oid: u8, clock: u32, response: Vec<u8> } = 15,

        // ── I2C ───────────────────────────────────────────────────────────

        /// Response to [`SendCommand::i2c_read`]. Contains the bytes read from the
        /// I2C device.
        ///
        /// Source: `src/i2ccmds.c`
        i2c_read_response { oid: u8, response: Vec<u8> } = 16,

        // ── TMC UART ─────────────────────────────────────────────────────

        /// Response to [`SendCommand::tmcuart_send`] when `read > 0`. Contains the
        /// data received from the TMC stepper driver's UART response.
        ///
        /// Source: `src/tmcuart.c`
        tmcuart_response { oid: u8, read: Vec<u8> } = 17,

        // ── NeoPixel ──────────────────────────────────────────────────────

        /// Response to [`SendCommand::neopixel_send`]. `success` is 1 if the LED
        /// data was transmitted successfully, or 0 if a hardware IRQ disrupted the
        /// timing-critical bit-banging.
        ///
        /// Source: `src/neopixel.c`
        neopixel_result { oid: u8, success: u8 } = 18,

        // ── Debug ─────────────────────────────────────────────────────────

        /// Response to [`SendCommand::debug_read`]. Contains the 32-bit value read
        /// from the specified memory address.
        ///
        /// Source: `src/debugcmds.c`
        debug_result { val: u32 } = 19,

        // ── Stats ─────────────────────────────────────────────────────────

        /// Periodic (every 5 seconds) MCU load statistics. `count` is the number of
        /// scheduling events, `sum` is total processing time, `sumsq` is sum of
        /// squared processing times (divided by 256). Used to estimate MCU load.
        ///
        /// Source: `src/basecmd.c`
        stats { count: u32, sum: u32, sumsq: u32 } = 20,

        // ── Sensor: Load Cell / Strain Gauge ──────────────────────────────

        /// Response to [`SendCommand::load_cell_endstop_query_state`]. Comprehensive
        /// status: homing state, trigger detection, precise MCU clock tick of trigger,
        /// filtered and raw sample values, current sample, and error conditions.
        load_cell_endstop_state { oid: u8, homing: u8, homing_triggered: u8, is_triggered: u8, trigger_ticks: u32, trigger_emit_ticks: u32, trigger_sample: i32, trigger_emit_sample: i32, sample: i32, sample_ticks: u32, error: u8 } = 21,

        /// Bulk sensor data message sent by all bulk sensors (accelerometers, load
        /// cells, angle sensors, etc.). `sequence` is a monotonically increasing
        /// sequence number for dropped-packet detection. `data` contains raw sensor
        /// readings in the sensor's native format.
        ///
        /// Source: `src/sensor_bulk.c`
        sensor_bulk_data { oid: u8, sequence: u16, data: Vec<u8> } = 22,

        /// Status report for bulk sensors. `clock` is MCU time, `query_ticks` is
        /// sensor read time, `next_sequence` is the next data packet sequence number,
        /// `buffered` is bytes waiting to be sent, `possible_overflows` counts
        /// detected FIFO overflows.
        ///
        /// Source: `src/sensor_bulk.c`
        sensor_bulk_status { oid: u8, clock: u32, query_ticks: u32, next_sequence: u16, buffered: u32, possible_overflows: u16 } = 23,

        // ── Sensor: CS123x ───────────────────────────────────────────────

        /// Response to [`SendCommand::cs123x_read`]. Contains the raw ADC count and
        /// the chip's configuration register value.
        cs123x_read_response { oid: u8, config: u8 } = 24,

        // ── Sensor: LDC1612 ──────────────────────────────────────────────

        /// Response to [`SendCommand::query_ldc1612_home_state`]. Reports whether
        /// the LDC1612 is in homing mode and the MCU clock time when the trigger
        /// fired.
        ldc1612_home_state { oid: u8, homing: u8, trigger_clock: u32 } = 25,

        // ── Sensor: Thermocouple ─────────────────────────────────────────

        /// Response to [`SendCommand::query_thermocouple`]. `value` is the raw ADC
        /// reading (format depends on chip type). `fault` contains fault flags (open
        /// circuit, short to GND/VCC). `next_clock` is when the next reading occurs.
        ///
        /// Source: `src/thermocouple.c`
        thermocouple_result { oid: u8, next_clock: u32, value: u32, fault: u8 } = 26,
    }
}
