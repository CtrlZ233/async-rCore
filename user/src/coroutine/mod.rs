use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::{Relaxed, SeqCst};
use core::task::Poll;
use lazy_static::lazy_static;
use spin::mutex::Mutex;
use crate::coroutine::config::{CAP, PRIO_NUM};
use crate::coroutine::manager::MANAGER;
use crate::coroutine::scheduler::TaskId;
use crate::coroutine::user_task::UserTask;
use crate::{syscall, yield_};

mod user_task;
mod task_waker;
mod manager;
mod scheduler;
mod config;

lazy_static!{
    pub static ref PROCESS_PRIO: AtomicUsize = AtomicUsize::new(0);
}

pub fn add_task_with_priority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize) -> usize {
    let task = Arc::new(UserTask::new(Mutex::new(future), prio));
    let tid = task.tid.tid;
    MANAGER.add(task, tid, prio);
    PROCESS_PRIO.store(MANAGER.high_prio(), SeqCst);
    tid
}

pub fn init_coroutine() {
    syscall::sys_init_coroutine(&PROCESS_PRIO);
}

pub fn add_callback_task(tid: usize) {
    MANAGER.re_back(tid);
}

pub fn coroutine_run() {
    println!(" > > > > > > > coroutine_run < < < < < < < ");
    loop {
        let task;
        let tid;
        {
            task = MANAGER.fetch();
            if task.is_none() {
                if MANAGER.empty() {
                    break;
                }
                yield_();
                continue;
            }
            PROCESS_PRIO.store(MANAGER.high_prio(), Relaxed);
            tid = task.clone().unwrap().tid;
        }
        // uprintln!("user current task is {}", task.clone().unwrap().tid.get_val());
        match task.clone().unwrap().execute() {
            Poll::Ready(_) => {
                MANAGER.remove(tid.tid);
            },
            Poll::Pending => {
                MANAGER.add_wait_task(tid.tid);
            },
        }
    }
    println!("coroutine_run end!!!!!!!");
}