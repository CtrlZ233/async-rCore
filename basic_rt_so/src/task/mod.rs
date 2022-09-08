mod user_task;
mod bitmap;
mod task_waker;
mod task_queue;
mod manager;
mod ccmap;

pub use user_task::{UserTask, TaskId};
use manager::{MANAGER, Manager};
use crate::config::{MAX_USER, PRIO_NUM};
use alloc::{vec::Vec, collections::BTreeSet};

use crate::syscall::{enable_timer_interrupt, disable_timer_interrupt, uyield};

use alloc::sync::Arc;
use alloc::boxed::Box;
use core::{future::Future, pin::Pin, task::Poll};

use spin::Mutex;
use lazy_static::*;

pub use manager::check_callback;
pub use ccmap::{wake_kernel_tid, wrmap_register};


#[no_mangle]
pub fn user_thread_main(pid: usize) {
    uprintln!(" > > > > > > > pid_{} thread_main < < < < < < < ", pid);
    let mut wait_task = BTreeSet::<usize>::new();
    loop {
        let task;
        let tid;
        {
            disable_timer_interrupt();
            task = MANAGER[pid].lock().fetch();
            enable_timer_interrupt();
            if task.is_none() {
                if wait_task.is_empty() {
                    break;
                }
                uyield();
                // uprintln!("test");
                continue;
            }
            tid = task.clone().unwrap().tid;
        }
        // uprintln!("user current task is {}", task.clone().unwrap().tid.get_val());
        match task.clone().unwrap().execute() {
            Poll::Ready(_) => {
                uprintln!("remove tid: {}", tid.0);
                wait_task.remove(&tid.0);
                // MANAGER[pid].lock().tasks.remove(&tid);
            },
            Poll::Pending => {
                uprintln!("insert tid: {}", tid.0);
                wait_task.insert(tid.0);
                // MANAGER[pid].lock().add(task.unwrap());
            },
        }
    }
    crate::uprintln!("user coroutine end!!!!!!!");
}


pub static mut CUR_COROUTINE: usize = 0;
#[no_mangle]
pub fn kernel_thread_main() {
    kprintln!(" > > > > > > > [kernel] thread_main < < < < < < < ");

    loop {
        let task;
        let tid;
        {
            let mut ex = MANAGER[0].lock();
            task = ex.fetch();
            if task.is_none() { break; }
            tid = task.clone().unwrap().tid;
        }
        unsafe {
            CUR_COROUTINE = tid.get_val();
            // crate::kprintln!("kernel cur_coroutine is {}", CUR_COROUTINE);                             
        }  
        match task.clone().unwrap().execute() {
            Poll::Ready(_) => {
                // MANAGER[0].lock().tasks.remove(&tid);
            },
            Poll::Pending => {
                // MANAGER[0].lock().add(task.unwrap());
            },
        }
    }
    kprintln!("kernel coroutine end!!!!!!!");
}


#[no_mangle]
#[inline(never)]
pub fn add_task_with_priority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize){
    disable_timer_interrupt();
    let task_queue = MANAGER[pid].lock().task_queue.clone();
    let task = Arc::new(UserTask::new(Mutex::new(future), prio, task_queue));
    MANAGER[pid].lock().add(task);
    enable_timer_interrupt();
}

/*********** 内核使用接口 ***************/
/// 返回最靠右的一位（最高优先级）
pub fn get_right_one_mask(num: usize) -> usize {
    let mut pos: usize = 1;
    for _ in 0..PRIO_NUM {
        if (num & pos) != 0 { return pos; }
        pos = pos << 1;
    }
    1
}

lazy_static! {
    pub static ref PRIO_PIDS: Arc<Mutex<BTreeSet<usize>>> = Arc::new(Mutex::new(BTreeSet::new()));
}

#[no_mangle]
pub fn update_global_bitmap() {
    let mut ans = 0;
    let mut u_maps:Vec<usize> = Vec::new();
    for i in 0..MAX_USER {
        let bitmap_val = MANAGER[i].lock().task_queue.lock().bitmap.get_val();
        u_maps.push(bitmap_val);
        ans = ans | bitmap_val;
    }

    let mask = get_right_one_mask(ans);
    PRIO_PIDS.lock().clear();
    for i in 0..MAX_USER {
        if (u_maps[i] & mask) != 0 { 
            PRIO_PIDS.lock().insert(i); 
        }
    }
}

#[no_mangle]
pub fn check_prio_pid(pid: usize) -> bool {
    if PRIO_PIDS.lock().len() == 0 {
        return false;
    }
    if PRIO_PIDS.lock().contains(&pid) { 
        PRIO_PIDS.lock().remove(&pid);
        return true; 
    }
    false
}

#[no_mangle]
pub fn kernel_current_corotine() -> usize {
    unsafe {
        CUR_COROUTINE
    }
}

#[no_mangle]
pub fn add_callback(pid: usize, tid: usize) {
    let mut ex = MANAGER[pid].lock();
    kprintln!("process {} add_callback {}", pid, tid);
    ex.add_callback(TaskId::get_tid_by_usize(tid));
}

