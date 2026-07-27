// This uses a few allocations and also f32s which aren't necessarily supported
// on microprocessors, however it only runs once so it should be fine.

use serde::{Deserialize, de::Error};

use crate::utils::units::{self, Mass, gram, kilogram};

/// Deserialize some string like "125 mm/s" or "125mm/s^2"
fn deserialize_unit<'de, D>(deserializer: D) -> Result<(f32, &'de str), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = <&'de str>::deserialize(deserializer)?;

    // Find split point
    let split_idx = s
        .chars()
        .position(|c| {
            !c.is_numeric()
                && c != '.'
                && c != '_'
                && c != ','
                && c != '-'
                && c != '+'
                && c != 'e'
                && c != 'E'
        })
        .unwrap_or(s.len());

    let num_str = &s[..split_idx].trim();

    // Skip whitespace to find unit start
    let unit_part = &s[split_idx..];
    let unit_str = unit_part.trim();

    // Validate Number
    if num_str.is_empty() {
        return Err(Error::custom("Input must start with a valid number"));
    }

    // Validate Unit Presence
    if unit_str.is_empty() {
        return Err(Error::custom(
            "Missing unit: value must include both a number and a unit (e.g., '500 mm/s')",
        ));
    }

    let num_str: alloc::string::String =
        num_str.chars().filter(|c| *c != '_' && *c != ',').collect();

    let num: f32 = num_str.parse().map_err(|_| Error::custom("Not a number"))?;

    Ok((num, unit_str))
}

impl<'de> Deserialize<'de> for units::Mass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (n, unit_str) = deserialize_unit(deserializer)?;

        match unit_str {
            "g" | "gram" | "grams" => Ok(Mass::new::<gram>(n as i32)),
            "kg" | "kilogram" | "kilograms" => {
                let c = Mass::new::<kilogram>(1).get::<gram>() as f32;
                Ok(Mass::new::<gram>((c * n) as i32))
            }
            _ => Err(Error::unknown_variant(
                unit_str,
                &["g", "kg", "gram", "kilogram", "grams", "kilograms"],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for units::ShortTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (n, unit_str) = deserialize_unit(deserializer)?;

        match unit_str {
            "us" | "microsecond" | "microseconds" => {
                Ok(units::ShortTime::new::<units::short_microsecond>(n as i32))
            }
            "ms" | "millisecond" | "milliseconds" => {
                let c = units::ShortTime::new::<units::short_millisecond>(1)
                    .get::<units::short_microsecond>() as f32;
                Ok(units::ShortTime::new::<units::short_microsecond>(
                    (c * n) as i32,
                ))
            }
            "s" | "second" | "seconds" => {
                let c = units::ShortTime::new::<units::short_second>(1)
                    .get::<units::short_microsecond>() as f32;
                Ok(units::ShortTime::new::<units::short_microsecond>(
                    (c * n) as i32,
                ))
            }
            "min" | "minute" | "minutes" => {
                let c = units::ShortTime::new::<units::short_minute>(1)
                    .get::<units::short_microsecond>() as f32;
                Ok(units::ShortTime::new::<units::short_microsecond>(
                    (c * n) as i32,
                ))
            }
            _ => Err(Error::unknown_variant(
                unit_str,
                &[
                    "us",
                    "microsecond",
                    "microseconds",
                    "ms",
                    "millisecond",
                    "milliseconds",
                    "s",
                    "second",
                    "seconds",
                    "min",
                    "minute",
                    "minutes",
                ],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for units::LongTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (n, unit_str) = deserialize_unit(deserializer)?;

        match unit_str {
            "us" | "µs" | "microsecond" | "microseconds" => {
                let c = units::LongTime::new::<units::long_microsecond>(1)
                    .get::<units::long_millisecond>() as f32;
                Ok(units::LongTime::new::<units::long_millisecond>(
                    (c * n) as i32,
                ))
            }
            "ms" | "millisecond" | "milliseconds" => {
                Ok(units::LongTime::new::<units::long_millisecond>(n as i32))
            }
            "s" | "second" | "seconds" => {
                let c = units::LongTime::new::<units::long_second>(1)
                    .get::<units::long_millisecond>() as f32;
                Ok(units::LongTime::new::<units::long_millisecond>(
                    (c * n) as i32,
                ))
            }
            "min" | "minute" | "minutes" => {
                let c = units::LongTime::new::<units::long_minute>(1)
                    .get::<units::long_millisecond>() as f32;
                Ok(units::LongTime::new::<units::long_millisecond>(
                    (c * n) as i32,
                ))
            }
            "h" | "hour" | "hours" => {
                let c = units::LongTime::new::<units::long_hour>(1).get::<units::long_millisecond>()
                    as f32;
                Ok(units::LongTime::new::<units::long_millisecond>(
                    (c * n) as i32,
                ))
            }
            "d" | "day" | "days" => {
                let c = units::LongTime::new::<units::long_day>(1).get::<units::long_millisecond>()
                    as f32;
                Ok(units::LongTime::new::<units::long_millisecond>(
                    (c * n) as i32,
                ))
            }
            _ => Err(Error::unknown_variant(
                unit_str,
                &[
                    "us",
                    "µs",
                    "microsecond",
                    "microseconds",
                    "ms",
                    "millisecond",
                    "milliseconds",
                    "s",
                    "second",
                    "seconds",
                    "min",
                    "minute",
                    "minutes",
                    "h",
                    "hour",
                    "hours",
                    "d",
                    "day",
                    "days",
                ],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for units::Length {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (n, unit_str) = deserialize_unit(deserializer)?;

        match unit_str {
            "htmm" => Ok(units::Length::new::<units::htmm>(n as i32)),
            "mm" | "millimeter" | "millimeters" => {
                let c = units::Length::new::<units::millimeter>(1).get::<units::htmm>() as f32;
                Ok(units::Length::new::<units::htmm>((c * n) as i32))
            }
            "cm" | "centimeter" | "centimeters" => {
                let c = units::Length::new::<units::centimeter>(1).get::<units::htmm>() as f32;
                Ok(units::Length::new::<units::htmm>((c * n) as i32))
            }
            "m" | "meter" | "meters" => {
                let c = units::Length::new::<units::meter>(1).get::<units::htmm>() as f32;
                Ok(units::Length::new::<units::htmm>((c * n) as i32))
            }
            _ => Err(Error::unknown_variant(
                unit_str,
                &[
                    "htmm",
                    "mm",
                    "millimeter",
                    "millimeters",
                    "cm",
                    "centimeter",
                    "centimeters",
                    "m",
                    "meter",
                    "meters",
                ],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for units::Velocity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (n, unit_str) = deserialize_unit(deserializer)?;

        match unit_str {
            "htmm/ms" => Ok(units::Velocity::new::<units::htmm_per_millisecond>(
                n as i32,
            )),
            "mm/s" | "millimeter per second" | "millimeters per second" => {
                let c = units::Velocity::new::<units::millimeter_per_second>(1)
                    .get::<units::htmm_per_millisecond>() as f32;
                Ok(units::Velocity::new::<units::htmm_per_millisecond>(
                    (c * n) as i32,
                ))
            }
            "cm/s" | "centimeter per second" | "centimeters per second" => {
                let c = units::Velocity::new::<units::centimeter_per_second>(1)
                    .get::<units::htmm_per_millisecond>() as f32;
                Ok(units::Velocity::new::<units::htmm_per_millisecond>(
                    (c * n) as i32,
                ))
            }
            "m/s" | "meter per second" | "meters per second" => {
                let c = units::Velocity::new::<units::meter_per_second>(1)
                    .get::<units::htmm_per_millisecond>() as f32;
                Ok(units::Velocity::new::<units::htmm_per_millisecond>(
                    (c * n) as i32,
                ))
            }
            _ => Err(Error::unknown_variant(
                unit_str,
                &[
                    "htmm/ms",
                    "mm/s",
                    "millimeter per second",
                    "millimeters per second",
                    "cm/s",
                    "centimeter per second",
                    "centimeters per second",
                    "m/s",
                    "meter per second",
                    "meters per second",
                ],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for units::Acceleration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (n, unit_str) = deserialize_unit(deserializer)?;

        match unit_str {
            "htmm/ms^2" | "htmm/ms²" => Ok(units::Acceleration::new::<
                units::htmm_per_millisecond_squared,
            >(n as i32)),
            "mm/s^2"
            | "mm/s²"
            | "millimeter per second squared"
            | "millimeters per second squared" => {
                // Conversion factor is less than 1 so it will get truncated
                let c = (units::Acceleration::new::<units::htmm_per_millisecond_squared>(1)
                    .get::<units::millimeter_per_second_squared>() as f32);

                Ok(units::Acceleration::new::<
                    units::htmm_per_millisecond_squared,
                >((n / c) as i32))
            }
            "cm/s^2"
            | "cm/s²"
            | "centimeter per second squared"
            | "centimeters per second squared" => {
                // Conversion factor is less than 1 so it will get truncated
                let c = (units::Acceleration::new::<units::htmm_per_millisecond_squared>(1)
                    .get::<units::centimeter_per_second_squared>() as f32);

                Ok(units::Acceleration::new::<
                    units::htmm_per_millisecond_squared,
                >((n / c) as i32))
            }
            "m/s^2" | "m/s²" | "meter per second squared" | "meters per second squared" => {
                // Conversion factor is less than 1 so it will get truncated
                let c = (units::Acceleration::new::<units::htmm_per_millisecond_squared>(1)
                    .get::<units::meter_per_second_squared>() as f32);

                Ok(units::Acceleration::new::<
                    units::htmm_per_millisecond_squared,
                >((n / c) as i32))
            }
            _ => Err(Error::unknown_variant(
                unit_str,
                &[
                    "htmm/ms^2",
                    "htmm/ms²",
                    "mm/s^2",
                    "mm/s²",
                    "millimeter per second squared",
                    "millimeters per second squared",
                    "cm/s^2",
                    "cm/s²",
                    "centimeter per second squared",
                    "centimeters per second squared",
                    "m/s^2",
                    "m/s²",
                    "meter per second squared",
                    "meters per second squared",
                ],
            )),
        }
    }
}
