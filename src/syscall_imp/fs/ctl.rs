use core::default;
use core::ffi::c_void;
use axfs::api::write;
use xmas_elf::program::Flags;

use crate::syscall_body;
use crate::syscall_imp::sys_mmap;

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

use core::ffi::c_int;
use arceos_posix_api::sys_dup as dup;
pub(crate) fn sys_dup(old_fd: c_int) -> isize {
    dup(old_fd) as isize
}

use arceos_posix_api::sys_dup2 as dup2;
pub(crate) fn sys_dup2(old_fd: c_int, new_fd: c_int) -> isize {
    dup2(old_fd, new_fd) as isize
}

use alloc::format;
use alloc::string::String;
pub fn get_path(path: *const u8) -> String {
    let curr = current();
    let curr_ext = curr.task_ext();
    let mut aspace = curr_ext.aspace.lock();
    
    let mut path = path as usize;
    let mut buf: [u8; 128] = [0; 128];
    let mut idx = 0;
    let mut c: [u8; 1] = [0];
    while idx < 128 {
        aspace.read(VirtAddr::from_ptr_of(path as *const u8), &mut c);
        if c[0] == 0 {
            break;
        } else {
            buf[idx] = c[0];
            idx += 1;
            path += 1;
        }
    }
    
    let mut res = String::new();
    unsafe {
        let tmp = core::str::from_utf8_unchecked(&buf[..idx]);
        res = format!("{}{}", res, tmp);
        
    }

    res
}

pub(crate) fn sys_chdir(path: *const u8) -> isize {
    let mut path = get_path(path);
    axfs::api::set_current_dir(path.as_str());
    0
}

pub(crate) fn sys_mkdirat(dirfd: usize, path: *const u8, mode: u32) -> isize {
    let cwd = axfs::api::current_dir().unwrap(); // "/"
    let mut path = get_path(path); // "test_chdir"
    let res = format!("{}{}/", cwd, path); // "/test_chdir/"

    if dirfd == -100isize as usize {
        if axfs::api::path_exists(path.as_str()) {
            // 文件已存在
            warn!("dir is exist");
            return -1;
        }
        let _ = axfs::api::create_dir(res.as_str());
        // 只要文件夹存在就返回0
        if axfs::api::path_exists(path.as_str()) {
            warn!("create dir success");
            0
        } else {
            warn!("create dir fail");
            -1
        }
    } else {
        -1
    }
    
}

use arceos_posix_api::sys_open;
pub(crate) fn sys_openat(_dirfd: c_int, path: *const i8, mode: usize) -> isize {
    //warn!("");
    sys_open(path, mode as c_int, 0) as isize
}

use arceos_posix_api::sys_close;
pub(crate) fn sys_close_with_fd(fd: usize) -> isize {
    //warn!("");
    sys_close(fd as c_int) as isize
}

// struct utsname {
//     sysname: [u8; 65],
// 	char nodename[65],
// 	char release[65],
// 	char version[65],
// 	char machine[65],
// 	char domainname[65],
// }
pub(crate) fn sys_uname(mut uname: usize) -> isize {
    let mut tmp: [u8; 65] = [0; 65];

    let names = [
        b"Starry-On-ArceOS\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        b"Starry - machine[0]\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        b"10.0.0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        b"10.0.0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        b"RISC-V 64 on SIFIVE FU740\0\0\0\0\0\0\0\0\0\0\0\0",
        b"https://github.com/Azure-stars/arceos",
    ];

    for name in names.iter() {
        for (i, c) in name.iter().enumerate() {
            tmp[i] = *c;
        }
        tmp[name.len()] = 0;
    
        let curr = current();
        let curr_ext = curr.task_ext();
        let mut aspace = curr_ext.aspace.lock();
        
        aspace.write(VirtAddr::from_ptr_of(uname as *mut u8), &tmp);
        uname += 65;
    }

    0
}

pub(crate) fn sys_fstat(fd: usize, kst: usize) -> isize {
    unsafe {
        arceos_posix_api::sys_fstat(fd as i32, kst as *mut arceos_posix_api::ctypes::stat) as isize
    }
}


