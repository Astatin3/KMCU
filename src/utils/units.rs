// This project needs a custom unit system so it can support high resolution units
// on a u32 type

//               Precision       Max value (+/-)
// Mass          0.1 g           429.5 tonnes
//
// ShortTime     1 microsecond   71.6 min
// LongTime      1 millisecond   49.7 days
//
// Length        1 / 100_000 mm  429.5 m
// Velocity      0.1 mm/s        429.5 km/s
// Acceleration  10 mm/s^2       42,950 m/s^2

pub type Mass = i32::Mass;
pub type ShortTime = i32::ShortTime;
pub type LongTime = i32::LongTime;
pub type Length = i32::Length;
pub type Velocity = i32::Velocity;
pub type Acceleration = i32::Acceleration;

pub use acceleration::{
    centimeter_per_second_squared, htmm_per_millisecond_squared, meter_per_second_squared,
    millimeter_per_second_squared,
};
pub use length::{centimeter, htmm, meter, millimeter};
pub use long_time::{
    long_day, long_hour, long_microsecond, long_millisecond, long_minute, long_second,
};
pub use mass::{gram, kilogram};
pub use short_time::{short_microsecond, short_millisecond, short_minute, short_second};
use uom::Conversion;
pub use velocity::{
    centimeter_per_second, htmm_per_millisecond, meter_per_second, millimeter_per_second,
};

system! {
    quantities: Q {
        length: htmm, L;
        mass: kilogram, M;
        long_time: long_millisecond, T;
    }

    units: U {
        mod length::Length,
        mod mass::Mass,
        mod short_time::ShortTime,
        mod long_time::LongTime,
        mod velocity::Velocity,
        mod acceleration::Acceleration,
    }
}

/// `LongTimeKind` is a `Kind` for separating long-range, coarse-precision
/// time spans from the fine-grained `Time` quantity, even though they share
/// the same dimension (T^1). Because `LongTimeKind: uom::Kind`, `dyn
/// LongTimeKind` automatically satisfies `uom::Kind` (and therefore all of
/// its `marker::Add`/`Sub`/etc. supertraits) via trait-object supertrait
/// coercion -- no concrete type or manual marker impls needed.
pub trait LongTimeKind: uom::Kind {}

/// Widen a fine-grained `ShortTime` into the coarser, wider-range `LongTime`.
/// Sub-millisecond precision is truncated -- explicit and visible at the
/// call site, unlike a silent operator overload would be.
impl From<ShortTime> for LongTime {
    fn from(t: ShortTime) -> Self {
        LongTime::new::<long_time::long_millisecond>(t.get::<short_time::short_millisecond>())
    }
}

/// Error returned when a `LongTime` value doesn't fit back into
/// `ShortTime`'s u32-microsecond range (~71.5 minutes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TryFromLongTimeError;

impl core::fmt::Display for TryFromLongTimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "LongTime value out of range for ShortTime (u32 microseconds)"
        )
    }
}

/// Narrow a `LongTime` back down into `ShortTime`. Fails if the value is
/// too large to fit in `ShortTime`'s u32-microsecond range (~71.5 minutes).
impl core::convert::TryFrom<LongTime> for ShortTime {
    type Error = TryFromLongTimeError;
    fn try_from(lt: LongTime) -> Result<Self, Self::Error> {
        lt.get::<long_time::long_millisecond>()
            .checked_mul(1_000)
            .map(ShortTime::new::<short_time::short_microsecond>)
            .ok_or(TryFromLongTimeError)
    }
}

mod i32 {
    Q!(super, i32);
}

pub mod mass {
    uom::quantity! {
        /// Mass (base unit kilogram, kg).
        quantity: Mass; "mass";

        /// Mass dimension.
        dimension: Q<
            Z0,  // length
            P1,  // mass
            Z0>; // time

        units {
            @gram: 0.1; "g", "gram", "grams";
            @kilogram: 1.0E3; "kg", "kilogram", "kilograms";
        }
    }
}

pub mod short_time {
    uom::quantity! {
        /// Time (base unit microsecond, us).
        quantity: ShortTime; "short time";

        /// Time dimension.
        dimension: Q<
            Z0,  // length
            Z0,  // mass
            P1>; // time

        units {
            @short_microsecond: 1.0;   "us", "microsecond", "microseconds";
            @short_millisecond: 1.0_E3; "ms", "millisecond", "milliseconds";
            @short_second:      1.0_E6; "s",  "second",      "seconds";
            @short_minute:      6.0_E7; "min","minute",      "minutes";
        }
    }
}

pub mod long_time {
    uom::quantity! {
        /// LongTime (base unit millisecond, ms). Same dimension as `Time`
        /// but a distinct `Kind`, so it can span days in a u32 without
        /// colliding with Time's nanosecond/microsecond-scale arithmetic.
        quantity: LongTime; "long time";

        /// LongTime dimension.
        dimension: Q<
            Z0,  // length
            Z0,  // mass
            P1>; // time

        kind: dyn super::LongTimeKind;

        units {
            @long_microsecond: 1.0_E-3; "µs",  "microsecond", "microsecond";
            @long_millisecond: 1.0;     "ms",  "millisecond", "milliseconds";
            @long_second:      1.0_E3;  "s",   "second",      "seconds";
            @long_minute:      6.0_E4;  "min", "minute",      "minutes";
            @long_hour:        3.6_E6;  "h",   "hour",        "hours";
            @long_day:         8.64_E7; "d",   "day",         "days";
        }
    }
}

pub mod length {
    uom::quantity! {
        /// Length (base unit 0.00001 mm).
        quantity: Length; "length";

        /// Length dimension.
        dimension: Q<
            P1,  // length
            Z0,  // mass
            Z0>; // time

        units {
            @htmm:       1.0;    "htmm", "hundred thousandth of a millimeter", "hundred thousandths of a millimeter";
            @millimeter: 1.0_E4;  "mm",     "millimeter",  "millimeters";
            @centimeter: 1.0_E5;  "cm",     "centimeter",  "centimeters";
            @meter:      1.0_E7;  "m",      "meter",       "meters";
        }
    }
}

pub mod velocity {
    uom::quantity! {
        quantity: Velocity; "velocity";

        /// Velocity dimension, LT⁻¹.
        /// Base unit is htmm/ms (system length base / system time base).
        dimension: Q<
            P1,     // length
            Z0,     // mass
            N1>;    // time

        units {
            @htmm_per_millisecond:  1.0;     "htmm/ms", "hundred thousandth of a millimeter per millisecond", "hundred thousandths of a millimeter per millisecond";
            @millimeter_per_second: 1.0_E1;  "mm/s",    "millimeter per second",                              "millimeters per second";
            @centimeter_per_second: 1.0_E2;  "cm/s",    "centimeter per second",                              "centimeters per second";
            @meter_per_second:      1.0_E4;  "m/s",     "meter per second",                                   "meters per second";
        }
    }
}

pub mod acceleration {
    uom::quantity! {
        quantity: Acceleration; "acceleration";

        /// Acceleration dimension, LT⁻².
        /// Base unit is htmm/ms² (system length base / system time base²).
        dimension: Q<
            P1,     // length
            Z0,     // mass
            N2>;    // time

        units {
            @htmm_per_millisecond_squared:  1.0;     "htmm/ms²", "hundred thousandth of a millimeter per millisecond squared", "hundred thousandths of a millimeter per millisecond squared";
            @millimeter_per_second_squared: 1.0_E-2; "mm/s²",    "millimeter per second squared",                              "millimeters per second squared";
            @centimeter_per_second_squared: 1.0_E-1; "cm/s²",    "centimeter per second squared",                              "centimeters per second squared";
            @meter_per_second_squared:      1.0_E1;  "m/s²",     "meter per second squared",                                   "meters per second squared";
        }
    }
}
