#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use user_lib::*;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

#[no_mangle]
pub fn main() -> i32 {
    let pid = getpid() as usize;
    println!("pid is {}, user test>>>>>>>", pid);
    init_coroutine();
    let max_len  = 1;
    let PRIO_NUM = 8;
    // println!("prio: {}", get_prio());
    for i in 0..max_len {
        add_task_with_priority(Box::pin(test_func(i)), (i % PRIO_NUM as isize) as usize);
    }
    let wake_tid = add_task_with_priority(Box::pin(first()),3);
    add_task_with_priority(Box::pin(wake_func(wake_tid)),0);
    println!("prio: {}", get_prio());
    if fork() == 0 {
        exec("process\0", &[core::ptr::null::<u8>()]);
    } else
    {
        // sleep(1000);
        // let tid = thread_create(coroutine_run as usize, 0);
        coroutine_run();
        // waittid(tid as usize);
    }
    //
    // sleep(1000);

    0
}

async fn test_func(i: isize) {
    sleep_busy(1000);
    println!("hart_id: {}", hart_id());
    println!("test func : {}", i);
}

async fn first() {
    let mut helper = Box::new(ReadHelper::new());
    helper.await;
    println!("wake!!!!!");
}

async fn wake_func(tid: usize) {
    add_callback_task(tid);
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