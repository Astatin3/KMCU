use crate::utils::units;

pub fn now() -> units::LongTime {
    units::LongTime::new::<units::long_millisecond>(0)
}

pub fn sleep(_time: units::LongTime) {}
