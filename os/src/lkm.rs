// use core::slice::SlicePattern;
use crate::fs::{open_file, OpenFlags};
use alloc::vec::Vec;
use crate::mm::{KERNEL_SPACE, MemorySet};
use lazy_static::*;
use alloc::sync::Arc;
use core::mem::transmute;
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;

lazy_static! {
    pub static ref BASIC_RT_VEC: Arc<Vec<u8>> = Arc::new(open_file("basic_rt_so", OpenFlags::RDONLY).unwrap().read_all());
    pub static ref BASIC_RT_MEMORYSET: MemorySet = MemorySet::from_module(
        BASIC_RT_VEC.clone().as_slice()
    );    
}

pub fn init(){
    println!("lkm init");
    add_lkm_image();
    println!("lkm init done");
}

fn add_lkm_image(){
    let _v = BASIC_RT_VEC.clone();

    KERNEL_SPACE.exclusive_access().add_kernel_module(&BASIC_RT_MEMORYSET);

    KERNEL_SPACE.exclusive_access().activate();
    // async fn test1() {
    //     log::debug!("43");
    // }
    // async fn test2() {
    //     log::debug!("444");
    // }
    // _start() 位于 0x87000000
    // 执行共享调度器的_start() 函数，填写好符号表
    let basic_start_addr = 0x8700_0000usize;
    unsafe {
        let basic_start: fn() = transmute(basic_start_addr);
        basic_start();
    }
    // add_task_with_prority(Box::pin(test1()), 0, 0);
    // add_task_with_prority(Box::pin(test2()), 1, 0);
}


pub const SYMBOL_ADDR: *const usize = 0x8701a000usize as *const usize;

pub fn alloc_task_id() -> usize {
    unsafe {
        let alloc_task_id: fn() -> usize = transmute(*(SYMBOL_ADDR.add(10)) as usize);
        alloc_task_id()
    }
}

pub fn add_task_with_prority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize,
                             pid: usize, thread_id: usize, tid: usize) {
    // log::warn!("kernel add task");
    unsafe {
        let add_task_with_prority: fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize,
                                      pid: usize, thread_id: usize, tid: usize) = transmute(*SYMBOL_ADDR as usize);
        add_task_with_prority(future, prio, pid, thread_id, tid);
    }
} 

pub fn kernel_thread_main() {
    unsafe {
        let kernel_thread_main: fn() = transmute(*(SYMBOL_ADDR.add(1)) as usize);
        kernel_thread_main();
    }
}

pub fn update_global_bitmap() {
    unsafe {
        let update_global_bitmap: fn() = transmute(*(SYMBOL_ADDR.add(3)) as usize);
        update_global_bitmap();
    }
}

pub fn check_prio_pid(pid: usize, thread_id: usize) -> bool{
    unsafe {
        let check_prio_pid: fn(usize, usize) -> bool = transmute(*(SYMBOL_ADDR.add(4)) as usize);
        check_prio_pid(pid, thread_id)
    }
}

/// 根据 key 唤醒对应的 kernel_tid
pub fn wake_kernel_tid(pid: usize, key: usize) {
    // log::warn!("wake_kernel_tid");
    unsafe {
        let wake_kernel_tid: fn(pid: usize, key: usize) = transmute(*(SYMBOL_ADDR.add(5)) as usize);
        wake_kernel_tid(pid, key)
    }
}

/// 根据 write_tid 向 WRMAP 中注册 (write_tid, read_tid)
pub fn wrmap_register(key: usize, kernel_tid: usize) {
    // log::warn!("{} register kernel_tid {}", key, kernel_tid);
    unsafe {
        let wrmap_register: fn(key: usize, kernel_tid: usize) = transmute(*(SYMBOL_ADDR.add(6)) as usize);
        wrmap_register(key, kernel_tid)
    }
}

/// 获取内核目前正在运行的协程 id
pub fn kernel_current_corotine() -> usize {
    unsafe {
        let kernel_current_corotine: fn() -> usize = transmute(*(SYMBOL_ADDR.add(7)) as usize);
        kernel_current_corotine()
    }
}

pub fn add_callback(pid: usize, thread_id: usize, tid: usize) -> bool {
    unsafe {
        let add_callback: fn(usize, usize, usize) -> bool = transmute(*(SYMBOL_ADDR.add(8)) as usize);
        add_callback(pid, thread_id, tid)
    }
}

pub fn update_callback() {
    unsafe {
        let update_callback: fn() = transmute(*(SYMBOL_ADDR.add(11)) as usize);
        update_callback()
    }
}


