use arceos_posix_api as api;

pub(crate) fn sys_clock_gettime(clock_id: i32, tp: *mut api::ctypes::timespec) -> i32 {
    unsafe { api::sys_clock_gettime(clock_id, tp) }
}

use arceos_posix_api::ctypes::timeval;
use axhal::time::wall_time;

pub(crate) fn sys_gettimeofday(tv: *mut timeval) -> isize {
    if tv.is_null() {
        return -1;
    }

    let _tv = wall_time().into();

    unsafe {
        *tv = _tv;
    }

    0
}
