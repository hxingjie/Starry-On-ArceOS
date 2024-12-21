use core::time::Duration;

use arceos_posix_api as api;
use axstd::process::exit;
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

use axtask::{current, TaskExtRef, TaskInner};
use memory_addr::VirtAddr;
use axmm::AddrSpace;
use crate::mm::load_user_app;
use alloc::sync::Arc;
use axhal::arch::UspaceContext;
use axsync::Mutex;
use alloc::string::String;
pub(crate) fn sys_clone(a0: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> isize {
    let cur = current();
    //warn!("sysclone, {}", cur.id().as_u64());
    unsafe {
        let cur_ext = cur.task_ext_ptr() as *mut TaskExt;
        let cur_ext = cur_ext.as_mut().unwrap();
        
        cur_ext.uctx.set_ip(cur_ext.uctx.get_ip() + 4);

        let mut child_uctx = cur_ext.uctx.clone_ctx();
        //child_uctx.set_ip(child_uctx.get_ip() + 4);
        child_uctx.set_retval(0);

        //cur_ext.uctx.set_ip(cur_ext.uctx.get_ip() + 4);

        //let (entry_vaddr, ustack_top, uspace) = load_user_app(cur.name()).unwrap();
        //let uspace = Arc::new(Mutex::new(uspace));
        let uspace = cur_ext.aspace.clone();

        let name = String::from(cur.name());
        let mut child_task = TaskInner::new(
            || {
                let curr = cur;
                let kstack_top = curr.kernel_stack_top().unwrap();
                unsafe { curr.task_ext().uctx.enter_uspace(kstack_top) };
            },
            name,
            crate::config::KERNEL_STACK_SIZE,
        );
        child_task.ctx_mut()
            .set_page_table_root(uspace.lock().page_table_root());
        child_task.init_task_ext(TaskExt::new(child_uctx, uspace));

        let id: u64 =  child_task.id().as_u64();
        axtask::spawn_task(child_task);

        cur_ext.child = id as usize;
        id as isize
    }
}

// 简单的放入等待队列，让下一个退出的进程唤醒
pub(crate) fn sys_wait4(pid: usize, watatus: usize) -> isize {
    sleep(Duration::new(2, 0));
    //axtask::wait_child();

    let cur = current();
    warn!("syswait, {}", cur.id().as_u64());
    let cur_ext = cur.task_ext();

    let mut aspace = cur_ext.aspace.lock();
    let num: i32 = 1;
    aspace.write(VirtAddr::from_ptr_of(watatus as *const i32), &num.to_be_bytes());
    
    cur_ext.child as isize
}

// 直接另起一个进程
pub(crate) fn sys_execve(path: usize) -> isize {
    let path = path as *const u8;
    let name = crate::syscall_imp::get_path(path);

    let (entry_vaddr, ustack_top, uspace) = crate::mm::load_user_app(name.as_str()).unwrap();
    let user_task = crate::task::spawn_user_task(
        Arc::new(Mutex::new(uspace)),
        UspaceContext::new(entry_vaddr.into(), ustack_top, 2333),
        name.as_str(),
    );

    let exit_code = user_task.join();
    exit(0)
}
