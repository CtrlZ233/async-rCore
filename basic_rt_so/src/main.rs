#![no_std]
#![feature(alloc_error_handler)]
#![no_main]
#![allow(unused)]
#![feature(linkage)]
#![feature(asm_sym)]
#![feature(asm_const)]


extern crate alloc;

#[macro_use]
mod console;


mod config;
// mod cbq;
mod sbi;
pub mod lang_items;
pub mod init;
mod syscall;
mod task;
use task::{TaskId,
    add_task_with_priority, 
    kernel_thread_main,
    user_thread_main,
    update_global_bitmap,
    wake_kernel_tid,
    wrmap_register,
    check_prio_pid,
    kernel_current_corotine,
    add_callback,
    check_callback,
    alloc_task_id,
};

use core::task::*;
use alloc::boxed::Box;
use crate::task::update_callback;
// mod ccmap;
// mod thread;

// use alloc::{vec::Vec, boxed::Box, sync::Arc};
// use task::{add_task_with_priority, kernel_thread_main, user_thread_main, kernel_current_corotine};
// use manager::EXECUTOR;
// use spin::Mutex;
// use crate::manager::Executor;
// use bitmap::update_global_bitmap;
// use bitmap::check_prio_pid;
// use ccmap::{wake_read_tid, wrmap_register};
// use cbq::{check_callback, cbq_init, cbq_add};

// core::arch::global_asm!(include_str!("entry.asm"));


#[link_section = ".bss.interface"]
pub static mut INTERFACE: [usize; 0x1000 / core::mem::size_of::<usize>()] = [0usize; 0x1000 / core::mem::size_of::<usize>()];


#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() {
    init::init_environment();
    unsafe{
        INTERFACE[0] = add_task_with_priority as usize;
        INTERFACE[1] = kernel_thread_main as usize;
        INTERFACE[2] = user_thread_main as usize;
        INTERFACE[3] = update_global_bitmap as usize;
        INTERFACE[4] = check_prio_pid as usize;
        INTERFACE[5] = wake_kernel_tid as usize;
        INTERFACE[6] = wrmap_register as usize;
        INTERFACE[7] = kernel_current_corotine as usize;
        INTERFACE[8] = add_callback as usize;
        INTERFACE[9] = check_callback as usize;
        INTERFACE[10] = alloc_task_id as usize;
        INTERFACE[11] = update_callback as usize;
        crate::kprintln!("[basic_rt] lib start-----------------------------");
        for addr in &INTERFACE {
            if *addr != 0 {
                crate::kprintln!("{:#x}", addr);
            }
        }
        crate::kprintln!("BASIC_RT_SO GOT addr {:#x}", &mut INTERFACE as *mut usize as usize);
    }
    // add_task_with_priority(Box::pin(test()), 0, 0);
    // kernel_thread_main();
    // panic!("shutdown")
}


// async fn test() {
//     kprintln!("test");
// }

  


