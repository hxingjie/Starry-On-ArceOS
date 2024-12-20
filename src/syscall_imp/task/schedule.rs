use core::time::Duration;

use arceos_posix_api as api;
use axtask::{sleep, TaskExtMut};

use crate::task::{spawn_user_task, TaskExt};

pub(crate) fn sys_sched_yield() -> i32 {
    api::sys_sched_yield()
}

pub(crate) fn sys_nanosleep(
    req: *const api::ctypes::timespec,
    rem: *mut api::ctypes::timespec,
) -> i32 {
    unsafe { api::sys_nanosleep(req, rem) }
}

use axtask::{current, TaskExtRef};
use memory_addr::VirtAddr;
pub(crate) fn sys_clone(a0: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> isize {
    //warn!("{}, {}, {}, {}, {}", a0, a1, a2, a3, a4);
    let cur = current();
    unsafe {
        let tmp = cur.task_ext_ptr() as *mut TaskExt;
        let tmp = tmp.as_mut().unwrap();
        let new_task_ref = spawn_user_task(tmp.aspace.clone(), tmp.uctx.clone());
        tmp.child = new_task_ref.id().as_u64() as usize;
        warn!("{}, {}, {}", cur.name(), cur.id().as_u64(), tmp.child);
        
        new_task_ref.id().as_u64() as isize
    }
}


pub(crate) fn sys_wait4(pid: usize, watatus: usize) -> isize {
    sleep(Duration::new(2, 0));
    // wait_child();

    let curr = current();
    let curr_ext = curr.task_ext();

    let mut aspace = curr_ext.aspace.lock();
    let num: i32 = 1;
    aspace.write(VirtAddr::from_ptr_of(watatus as *const i32), &num.to_be_bytes());
    
    curr_ext.child as isize
}
