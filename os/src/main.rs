//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`syscall`]: System call handling and implementation
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`batch::run_next_app()`] and for the first time go to
//! userspace.

// #![deny(missing_docs)]
// #![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![allow(unused_assignments)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

use core::arch::{global_asm};
use core::hint;
use core::sync::atomic::{AtomicBool, Ordering};
use log::debug;
use crate::config::CPU_NUM;
use crate::mm::init_kernel_space;
use crate::sbi::send_ipi;


#[path = "boards/qemu.rs"]
mod board;
#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

mod logging;

global_asm!(include_str!("entry.asm"));

/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

static AP_CAN_INIT: AtomicBool = AtomicBool::new(false);
/// the rust entry-point of os
#[no_mangle]
pub fn rust_main(hart_id: usize) -> ! {

    if hart_id == 0 {
        println!("hart id: {}", hart_id);
        clear_bss();
        logging::init();
        log::info!("[kernel] Hello, world!");
        mm::init();
        mm::remap_test();
        trap::init();
        task::add_initproc();
        for i in 1..CPU_NUM {
            debug!("[kernel {}] Start {}", hart_id, i);
            let mask: usize = 1 << i;
            send_ipi(&mask as *const _ as usize);
        }
        AP_CAN_INIT.store(true, Ordering::Relaxed);
    } else {
        while !AP_CAN_INIT.load(Ordering::Relaxed) {
            hint::spin_loop();
        }
        let hart_id = task::hart_id();
        init_kernel_space();
        trap::init();
    }
    timer::set_next_trigger();

    if hart_id == 0 {
        fs::list_apps();
    }
    // task::add_user_test();

    task::run_tasks();
    log::debug!("here4");
    panic!("Unreachable in rust_main!");
}


