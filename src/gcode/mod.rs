use serde::Deserialize;

mod de;
mod iter;

use de::{Error, from_str};

pub use iter::GcodeIter;

#[derive(Debug, Deserialize)]
pub enum Gcode {
    /// Rapid linear move. Commands the tool to move in a straight line to the
    /// specified position at the maximum possible feed rate. No material is
    /// extruded during a G0 move.
    ///
    /// # Parameters
    /// - `x` – Target X position (mm). Omit to leave unchanged.
    /// - `y` – Target Y position (mm). Omit to leave unchanged.
    /// - `z` – Target Z position (mm). Omit to leave unchanged.
    /// - `f` – Feed rate (mm/min). Persists for subsequent moves.
    ///
    /// # Example
    /// ```gcode
    /// G0 X100 Y50 F9000
    /// ```
    G0(G0),

    /// Linear move at specified feed rate. Commands the tool to move in a
    /// straight line to the specified position. Material is extruded at the
    /// commanded feed rate and E value.
    ///
    /// # Parameters
    /// - `x` – Target X position (mm).
    /// - `y` – Target Y position (mm).
    /// - `z` – Target Z position (mm).
    /// - `e` – Extrude this amount of filament (mm). Positive = extrude,
    ///   negative = retract (unless M83 is active).
    /// - `f` – Feed rate (mm/min).
    ///
    /// # Example
    /// ```gcode
    /// G1 X100.5 Y200.3 E12.7 F1800
    /// ```
    G1(G1),

    /// Clockwise arc. Moves the tool along a circular arc in the currently
    /// selected plane (default XY, set via G17/G18/G19).
    ///
    /// # Parameters
    /// - `x` / `y` / `z` – Endpoint of the arc.
    /// - `i` / `j` / `k` – Distance from the current position to the arc
    ///   center, along each axis. At least two of these are required.
    /// - `r` – Arc radius (mm). Alternative to I/J/K.
    /// - `e` – Extrude this amount (mm).
    /// - `f` – Feed rate (mm/min).
    ///
    /// # Example
    /// ```gcode
    /// G2 X100 Y50 I25 J0 E5 F1200
    /// ```
    G2(G2),

    /// Counter-clockwise arc. Same parameters as G2 but the arc is traced in
    /// the opposite direction.
    ///
    /// # Example
    /// ```gcode
    /// G3 X100 Y50 I25 J0 E5 F1200
    /// ```
    G3(G3),

    /// Dwell. Pauses the machine for a specified duration. All active moves
    /// must complete before the dwell begins.
    ///
    /// # Parameters
    /// - `p` – Dwell time in milliseconds.
    ///
    /// # Example
    /// ```gcode
    /// G4 P1000
    /// ```
    G4(G4),

    /// Set tool offset. In RepRap firmware this is overloaded to also handle
    /// workspace coordinate offsets and firmware retraction settings.
    ///
    /// # Parameters
    /// - `s` – Firmware retract: retract length (mm). Positive value sets
    ///   the retract distance; a negative value is ignored.
    /// - `p` – Firmware retract: retract extra (extra distance added to the
    ///   retract move).
    /// - `f` – Firmware retract: retract feedrate (mm/min).
    ///
    /// # Example (firmware retract)
    /// ```gcode
    /// G10 S1.0 P0.5 F1800
    /// ```
    G10(G10),

    /// Firmware unretract. Reverses the last G10 retract move by the same
    /// distance, feedrate, and extra values that were active when G10 was
    /// issued. Takes no parameters.
    G11,

    /// Clean nozzle. Firmware-specific routine that wipes the nozzle. In
    /// Klipper this is typically a macro, not a native command.
    G12,

    /// Select XY plane (default). Sets the active arc plane to the XY plane.
    /// G2/G3 arcs will be interpolated in this plane. The Z axis is the
    /// normal to the plane.
    G17,

    /// Select XZ plane. Sets the active arc plane to the XZ plane. The Y
    /// axis is the normal to the plane.
    G18,

    /// Select YZ plane. Sets the active arc plane to the YZ plane. The X
    /// axis is the normal to the plane.
    G19,

    /// Set units to inches. All subsequent coordinates and feed rates are
    /// interpreted in inches.
    G20,

    /// Set units to millimeters (default). All subsequent coordinates and
    /// feed rates are interpreted in millimeters.
    G21,

    /// Home the specified axes. Moves each requested axis to its endstop
    /// and sets the machine position to the configured home offset. If no
    /// axes are specified, all axes are homed.
    ///
    /// # Parameters
    /// - `x` – Home X axis (flag, no value needed).
    /// - `y` – Home Y axis.
    /// - `z` – Home Z axis.
    ///
    /// # Example
    /// ```gcode
    /// G28       ; home all axes
    /// G28 X Y   ; home only X and Y
    /// ```
    G28(G28),

    /// Probe the bed surface. The behaviour is highly firmware-dependent.
    /// In Klipper this is typically a macro. Sends the Z probe downward
    /// until it triggers, records the position, and repeats for a mesh or
    /// single-point calibration.
    G29,

    /// Single Z probe at the current position. Sends the probe down until
    /// triggered and reports the distance.
    G30,

    /// Probe toward the workpiece (touch probe). Moves the tool toward the
    /// workpiece along the specified axis until the probe triggers. If the
    /// probe does not trigger, the move is aborted with an error.
    ///
    /// # Parameters
    /// - `x` / `y` / `z` – Travel limit for the probe move (mm).
    /// - `f` – Feed rate for the probing move (mm/min).
    ///
    /// # Example
    /// ```gcode
    /// G38.2 Z5 F300
    /// ```
    #[serde(rename = "G38.2")]
    G38_2(G38_2),

    /// Cancel the current motion mode. Any in-progress G2/G3 arc or
    /// buffered G1 moves are flushed and the machine stops at the current
    /// position.
    G80,

    /// Set to absolute positioning mode. All subsequent coordinates are
    /// interpreted as absolute positions relative to the machine origin.
    G90,

    /// Set to relative positioning mode. All subsequent coordinates are
    /// interpreted as offsets from the current position.
    G91,

    /// Set position. Overrides the current machine coordinate for one or
    /// more axes without moving. Useful for setting the extruder position
    /// or redefining the origin.
    ///
    /// # Parameters
    /// - `x` – Set X position to this value (mm).
    /// - `y` – Set Y position to this value (mm).
    /// - `z` – Set Z position to this value (mm).
    /// - `e` – Set E position to this value (mm).
    ///
    /// # Example
    /// ```gcode
    /// G92 E0       ; reset extruder position to 0
    /// G92 X0 Y0   ; redefine current position as origin
    /// ```
    G92(G92),

    // =====================================================================
    //  M-codes
    // =====================================================================
    /// Program end / stop. Immediately stops the program. On Klipper this
    /// turns off the heaters and disables the steppers.
    M0,

    /// Sleep. Identical to M0 on most 3D printer firmware.
    M1,

    /// Spindle on (clockwise) or laser on. In laser mode the S parameter
    /// sets the laser power.
    ///
    /// # Parameters
    /// - `s` – Laser power (0–255 or 0.0–1.0 depending on firmware).
    /// - `p` – Spindle speed in RPM (for CNC spindles).
    ///
    /// # Example
    /// ```gcode
    /// M3 S128
    /// ```
    M3(M3),

    /// Spindle on (counter-clockwise) or laser on with M4-style power
    /// scaling. Behaves the same as M3 on most 3D printer firmware.
    ///
    /// # Parameters
    /// - `s` – Laser power.
    /// - `p` – Spindle speed in RPM.
    M4(M4),

    /// Spindle off / laser off. Turns off the spindle or laser.
    M5,

    /// Mist coolant on.
    M7,

    /// Flood coolant on.
    M8,

    /// All coolant off.
    M9,

    /// Enable (power on) stepper motors. Re-enables the drivers so they
    /// hold position.
    ///
    /// # Parameters
    /// - `x` / `y` / `z` / `e` – Enable only the specified axes. If none
    ///   are given, all are enabled.
    ///
    /// # Example
    /// ```gcode
    /// M17 X Y Z
    /// ```
    M17,

    /// Disable stepper motors. The motors lose holding torque and can be
    /// moved by hand.
    ///
    /// # Parameters
    /// - `x` / `y` / `z` / `e` – Disable only the specified axes. If none
    ///   are given, all are disabled.
    ///
    /// # Example
    /// ```gcode
    /// M18      ; disable all
    /// M18 E    ; disable only the extruder
    /// ```
    M18,

    /// Set extruder to absolute mode. Positive E values extrude; negative
    /// values retract. This is the default.
    M82,

    /// Set extruder to relative mode. Each E value is an amount to extrude
    /// relative to the current position, regardless of what the absolute E
    /// position is.
    M83,

    /// Set extruder target temperature (non-blocking). The heater begins
    /// ramping to the requested temperature but the command returns
    /// immediately without waiting.
    ///
    /// # Parameters
    /// - `s` – Target temperature in °C.
    ///
    /// # Example
    /// ```gcode
    /// M104 S210
    /// ```
    M104(M104),

    /// Report current temperatures. Queries all temperature sensors and
    /// reports the current and target values. Takes no parameters.
    M105,

    /// Set fan speed. Turns on the part cooling fan at the specified speed.
    ///
    /// # Parameters
    /// - `s` – Fan speed. 0 = off; 255 = full speed (or 0.0–1.0 in some
    ///   firmware).
    ///
    /// # Example
    /// ```gcode
    /// M106 S255
    /// ```
    M106(M106),

    /// Fan off. Turns off the part cooling fan. Takes no parameters.
    M107,

    /// Wait for extruder temperature (blocking). Pauses the program until
    /// the extruder has reached the specified temperature and is stable.
    ///
    /// # Parameters
    /// - `s` – Target temperature in °C.
    ///
    /// # Example
    /// ```gcode
    /// M109 S210
    /// ```
    M109(M109),

    /// Emergency stop. Immediately halts the machine, turns off heaters,
    /// and disables motors. Requires a firmware reset to recover.
    M112,

    /// Set bed temperature (non-blocking). Begins heating the bed but does
    /// not wait.
    ///
    /// # Parameters
    /// - `s` – Target temperature in °C.
    ///
    /// # Example
    /// ```gcode
    /// M140 S60
    /// ```
    M140(M140),

    /// Auto-report temperatures. Enables or disables automatic periodic
    /// temperature reports.
    ///
    /// # Parameters
    /// - `s` – Report interval in seconds. 0 = disable.
    ///
    /// # Example
    /// ```gcode
    /// M154 S5
    /// ```
    M154(M154),

    /// Wait for bed temperature (blocking). Pauses until the bed has
    /// reached the specified temperature.
    ///
    /// # Parameters
    /// - `s` – Target temperature in °C.
    ///
    /// # Example
    /// ```gcode
    /// M190 S60
    /// ```
    M190(M190),

    /// Set maximum feedrate for one or more axes.
    ///
    /// # Parameters
    /// - `x` / `y` / `z` / `e` – Maximum feedrate for that axis (mm/s).
    ///
    /// # Example
    /// ```gcode
    /// M203 X300 Y300 Z5 E40
    /// ```
    M203(M203),

    /// Set default acceleration.
    ///
    /// # Parameters
    /// - `s` – Print move acceleration (mm/s²).
    /// - `t` – Retract move acceleration (mm/s²).
    /// - `r` – Travel move acceleration (mm/s²).
    ///
    /// # Example
    /// ```gcode
    /// M204 S3000 T3000 R6000
    /// ```
    M204(M204),

    /// Advanced settings. Firmware-specific tuning parameters.
    ///
    /// # Parameters
    /// - `e` – E jerk / junction deviation (mm/s).
    /// - `j` – Junction deviation (mm).
    /// - `s` – Minimum feed rate (mm/s).
    /// - `t` – Minimum travel feed rate (mm/s).
    /// - `b` – Minimum segment time (µs).
    /// - `x` – X jerk (mm/s).
    /// - `y` – Y jerk (mm/s).
    /// - `z` – Z jerk (mm/s).
    ///
    /// # Example
    /// ```gcode
    /// M205 X10 Y10 Z0.4 E5
    /// ```
    M205(M205),

    /// Set speed factor override percentage. Multiplies the feed rate of
    /// all subsequent moves by this factor.
    ///
    /// # Parameters
    /// - `s` – Speed factor percentage (100 = normal speed).
    ///
    /// # Example
    /// ```gcode
    /// M220 S50    ; print at half speed
    /// M220 S100   ; restore normal speed
    /// ```
    M220(M220),

    /// Set extrude factor override percentage. Multiplies the amount of
    /// filament extruded by this factor.
    ///
    /// # Parameters
    /// - `s` – Flow factor percentage (100 = normal flow).
    ///
    /// # Example
    /// ```gcode
    /// M221 S120   ; extrude 20% more material
    /// ```
    M221(M221),

    /// Allow cold extrusion. Overrides the minimum extrusion temperature
    /// check so that filament can be extruded without heating the nozzle.
    /// Useful for filament loading/unloading.
    ///
    /// # Parameters
    /// - `s` – Cold extrusion speed (mm/s). Omit for firmware default.
    ///
    /// # Example
    /// ```gcode
    /// M302 S0     ; allow extrusion at any temperature
    /// ```
    M302(M302),

    /// Wait for all buffered moves to finish. The program pauses until the
    /// motion planner has executed all queued moves. Takes no parameters.
    M400,

    /// Save all non-volatile parameters (mesh, offsets, PID, etc.) to
    /// EEPROM / SD card. Takes no parameters.
    M500,

    /// Load saved parameters from EEPROM. Takes no parameters.
    M501,

    /// Reset all parameters to firmware defaults. Takes no parameters.
    M502,

    /// Report current firmware settings to the host. Takes no parameters.
    M503,

    /// Firmware-specific catch-all. Not part of the standard. Commands that
    /// don't match any known variant are routed here with the raw command
    /// name preserved.
    #[serde(skip)]
    Macro(String),
}

// =============================================================================
//  Struct definitions for parameterized variants
// =============================================================================

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G0 {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub f: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G1 {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub e: Option<f32>,
    pub f: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G2 {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub i: Option<f32>,
    pub j: Option<f32>,
    pub k: Option<f32>,
    pub r: Option<f32>,
    pub e: Option<f32>,
    pub f: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G3 {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub i: Option<f32>,
    pub j: Option<f32>,
    pub k: Option<f32>,
    pub r: Option<f32>,
    pub e: Option<f32>,
    pub f: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G4 {
    pub p: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G10 {
    pub s: Option<f32>,
    pub p: Option<f32>,
    pub r: Option<f32>,
    pub l: Option<u32>,
    pub i: Option<f32>,
    pub j: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G28 {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G38_2 {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub f: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct G92 {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub e: Option<f32>,
}

// -- M-code structs --

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M3 {
    pub s: Option<f32>,
    pub p: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M4 {
    pub s: Option<f32>,
    pub p: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M17 {
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub e: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M18 {
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub e: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M104 {
    pub s: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M106 {
    pub s: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M109 {
    pub s: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M140 {
    pub s: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M154 {
    pub s: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M190 {
    pub s: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M203 {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub e: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M204 {
    pub s: Option<f32>,
    pub t: Option<f32>,
    pub r: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M205 {
    pub e: Option<f32>,
    pub j: Option<f32>,
    pub s: Option<f32>,
    pub t: Option<f32>,
    pub b: Option<u32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M220 {
    pub s: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M221 {
    pub s: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct M302 {
    pub s: Option<f32>,
}
