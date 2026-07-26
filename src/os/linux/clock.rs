use crate::utils::units;

#[static_init::dynamic]
static START_LONG_TIME: units::LongTime = get_libc_time();

pub fn sleep(duration: units::LongTime) {
    unsafe {
        libc::usleep(duration.get::<units::long_microsecond>() as libc::c_uint);
    }
}

fn get_libc_time() -> units::LongTime {
    let time = unsafe {
        let mut ts: libc::timespec = core::mem::zeroed();
        // Use CLOCK_REALTIME for epoch time, or CLOCK_PROCESS_CPUTIME_ID for CPU time
        if libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts) == 0 {
            // Convert to milliseconds.
            // WARNING: This truncates to u32 range (overflows in 2038)
            ((ts.tv_sec as u64 * 1000) + (ts.tv_nsec as u64 / 1_000_000)) as u32
        } else {
            0 // Handle error appropriately
        }
    };

    units::LongTime::new::<units::long_millisecond>(time)
}

pub fn now() -> units::LongTime {
    get_libc_time() - *START_LONG_TIME
}
