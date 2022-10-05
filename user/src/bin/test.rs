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
    let max_len  = 4000;
    let PRIO_NUM = 8;
    for i in 0..max_len {
        let mut tid = alloc_task_id();
        add_task_with_prority(Box::pin(test_func(i)), (i % PRIO_NUM as isize) as usize, pid, 0, tid);
    }
    let mut wake_tid = alloc_task_id();
    add_task_with_prority(Box::pin(first()),0, pid, 0, wake_tid);
    let new_tid = alloc_task_id();
    add_task_with_prority(Box::pin(wake_func(wake_tid)), 3, pid, 0, new_tid);
    // let tid1 = alloc_task_id();
    // add_task_with_prority(Box::pin(test1()), 0, pid, tid1);
    // let tid2 = alloc_task_id();
    // add_task_with_prority(Box::pin(test2()), 1, pid, tid2);
    // // 执行一段时间空操作使得用户进程的时间片用完，进入时钟中断
    // for _i in 0..(1 << 25) {
    //     unsafe {
    //         core::arch::asm!(
    //             "addi x0,x0,0"
    //         );
    //     }
    // }
    user_thread_main(pid, 0);
    0
}

async fn test_func(i: isize) {
    println!("test func : {}", i);
}

async fn first() {
    let mut helper = Box::new(ReadHelper::new());
    helper.await;
    println!("wake!!!!!");
}

async fn wake_func(tid: usize) {
    println!("wake func : {}", tid);;
}

pub struct ReadHelper(usize);

impl ReadHelper {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Future for ReadHelper {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0 += 1;
        if (self.0 & 1) == 1 {
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}