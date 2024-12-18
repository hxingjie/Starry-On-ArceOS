use core::ffi::c_void;

use axfs::api::write;

use crate::syscall_body;

/// The ioctl() system call manipulates the underlying device parameters
/// of special files.
///
/// # Arguments
/// * `fd` - The file descriptor
/// * `op` - The request code. It is of type unsigned long in glibc and BSD,
/// and of type int in musl and other UNIX systems.
/// * `argp` - The argument to the request. It is a pointer to a memory location
pub(crate) fn sys_ioctl(_fd: i32, _op: usize, _argp: *mut c_void) -> i32 {
    syscall_body!(sys_ioctl, {
        warn!("Unimplemented syscall: SYS_IOCTL");
        Ok(0)
    })
}

use axtask::{current, TaskExtRef};
use memory_addr::VirtAddr;
pub(crate) fn sys_getcwd(buf: *mut u8, len: usize) -> isize {
    // 使用api获取cwd
    let cwd = axfs::api::current_dir().expect("get cwd fail");
    let cwd = cwd.as_bytes();

    if cwd.len() <= len {
        // buf是虚拟地址，获取当前地址空间以写入物理地址
        let curr = current();
        let curr_ext = curr.task_ext();
        let mut aspace = curr_ext.aspace.lock();
        
        aspace.write(VirtAddr::from_ptr_of(buf), cwd);

        buf as isize
    } else {
        warn!("len isn't enough.");
        0
    }
}
