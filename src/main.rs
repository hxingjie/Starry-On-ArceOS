#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate log;
extern crate alloc;
extern crate axstd;

#[rustfmt::skip]
mod config {
    include!(concat!(env!("OUT_DIR"), "/uspace_config.rs"));
}
mod loader;
mod mm;
mod syscall_imp;
mod task;

use alloc::sync::Arc;

use axhal::arch::UspaceContext;
use axsync::Mutex;

static TESTCASES: [&str; 33] = [
    "brk", "chdir", "clone", "close", "dup", "dup2", "execve", "exit",
    "fork", "fstat", "getcwd", "getdents", "getpid", "getppid", "gettimeofday", "mkdir",
    "mmap", "mount", "munmap", "open", "openat", "pipe", "read", "sleep", 
    "test_echo", "times", "umount", "uname", "unlink", "wait", "waitpid", "write", 
    "yield", 
];

fn run_test_from_disk(name: &str) {
    let (entry_vaddr, ustack_top, uspace) = mm::load_user_app(name).unwrap();
    let user_task = task::spawn_user_task(
        Arc::new(Mutex::new(uspace)),
        UspaceContext::new(entry_vaddr.into(), ustack_top, 2333),
    );
    let exit_code = user_task.join();
}

#[no_mangle]
fn main() {
    run_test_from_disk("getcwd");

    // loader::list_apps();
    // let testcases = option_env!("AX_TESTCASES_LIST")
    //     .unwrap_or_else(|| "Please specify the testcases list by making user_apps")
    //     .split(',')
    //     .filter(|&x| !x.is_empty());
    // for testcase in testcases {
    //     info!("Running testcase: {}", testcase);
    //     let (entry_vaddr, ustack_top, uspace) = mm::load_user_app(testcase).unwrap();
    //     let user_task = task::spawn_user_task(
    //         Arc::new(Mutex::new(uspace)),
    //         UspaceContext::new(entry_vaddr.into(), ustack_top, 2333),
    //     );
    //     let exit_code = user_task.join();
    //     info!("User task {} exited with code: {:?}", testcase, exit_code);
    // }
}
