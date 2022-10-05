#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use user_lib::*;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

async fn test1() {
    println!("4333333");
}
async fn test2() {
    println!("443333333333334");
}

#[no_mangle]
pub fn main() -> i32 {
    let pid = getpid() as usize;
    println!("pid is {}, user test>>>>>>>", pid);
    init_coroutine();
    add_task_with_priority(Box::pin(test1()), 5);
    add_task_with_priority(Box::pin(test2()), 0);
    // sleep(1000);
    coroutine_run();
    0
}