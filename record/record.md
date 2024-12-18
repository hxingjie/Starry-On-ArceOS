# Starry-On-ArceOS



### 1.从文件加载测例

修改依赖文件

```shell
# 修改 ./scripts/get_deps.sh 中的 arceos-org/arceos 为 Azure-stars/Starry

./scripts/get_deps.sh
./build_img.sh -a riscv64 -file base_riscv64 # 打包测例
mv disk.img ./.arceos
make run ARCH=riscv64 BLK=y FEATURES=fs # 运行内核
```

修改Cargo.toml

```toml
axstd = { path = ".arceos/ulib/axstd", features = ["paging"] }
axhal = { path = ".arceos/modules/axhal", features = ["uspace"] }
axmm = { path = ".arceos/modules/axmm" }
axtask = { path = ".arceos/modules/axtask" }
axsync = { path = ".arceos/modules/axsync" }
axruntime = { path = ".arceos/modules/axruntime", features = ["multitask"] }
arceos_posix_api = { path = ".arceos/api/arceos_posix_api" }
axfs = { path = ".arceos/modules/axfs" }
```

修改load.rs load_elf

```rust
pub(crate) fn load_elf(name: &str, base_addr: VirtAddr) -> ELFInfo {
    let data_vec = read(name).expect("can't find app in disk");
    let data;
    unsafe {
        data = core::slice::from_raw_parts(&data_vec[0] as *const u8, data_vec.len());
    }
    let elf = ElfFile::new(data).expect("invalid ELF file");

    // let elf = ElfFile::new(
    //     get_app_data_by_name(name).unwrap_or_else(|| panic!("failed to get app: {}", name)),
    // )
    // .expect("invalid ELF file");
}
```

修改main.rs

```rust
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
    run_test_from_disk("gettimeofday");
}
```



### 2.实现 gettimeofday

```rust
// Starry-On-ArceOS/src/syscall_imp/time.rs
use arceos_posix_api::api::ctypes::timeval;
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
```

![image-20241218205622042](./record.assets/gettimeofday.png)



### 3.实现 getcwd

```rust
// Starry-On-ArceOS/src/syscall_imp/fs/ctl.rs

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
```

![image-20241218220430705](./record.assets/getcwd.png)





### 4.