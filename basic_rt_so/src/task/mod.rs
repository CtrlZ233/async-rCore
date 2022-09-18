mod user_task;
mod bitmap;
mod task_waker;
mod task_queue;
mod manager;
mod ccmap;

pub use user_task::{UserTask, TaskId, alloc_task_id};
use manager::{MANAGER, Manager};
use crate::config::{MAX_USER, MAX_THREAD, PRIO_NUM};
use alloc::{vec::Vec, collections::BTreeSet};

use crate::syscall::{enable_timer_interrupt, disable_timer_interrupt, uyield};

use alloc::sync::Arc;
use alloc::boxed::Box;
use core::{future::Future, pin::Pin, task::Poll};
use core::borrow::BorrowMut;

use spin::Mutex;
use lazy_static::*;

pub use manager::check_callback;
pub use ccmap::{wake_kernel_tid, wrmap_register};
use crate::task::manager::{BITMAPS, CALLBACKS, CallbackTask};

#[no_mangle]
pub fn user_thread_main(pid: usize, thread_id: usize) {
    uprintln!(" > > > > > > > pid_{} thread_id: {} thread_main < < < < < < < ", pid, thread_id);
    let mut wait_task = BTreeSet::<usize>::new();
    loop {
        let task;
        let tid;
        {
            // disable_timer_interrupt();
            task = MANAGER[pid][thread_id].user_fetch();
            // enable_timer_interrupt();
            if task.is_none() {
                if wait_task.is_empty() {
                    break;
                }
                uyield();
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
    crate::uprintln!("pid_{} thread_id: {} thread_main coroutine end!!!!!!!", pid, thread_id);
}


pub static mut CUR_COROUTINE: usize = 0;
#[no_mangle]
pub fn kernel_thread_main() {
    kprintln!(" > > > > > > > [kernel] thread_main < < < < < < < ");

    loop {
        let task;
        let tid;
        {
            let mut ex = MANAGER[0][0].clone();
            task = ex.kernel_fetch();
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
pub fn add_task_with_priority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize,
                              pid: usize, thread_id: usize, tid: usize){
    // disable_timer_interrupt();
    let task_queue = MANAGER[pid][thread_id].task_queue.clone();
    let task = Arc::new(UserTask::new(Mutex::new(future), prio, task_queue, tid));
    MANAGER[pid][thread_id].add(task);
    // enable_timer_interrupt();
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
    pub static ref PRIO_PIDS: Arc<Mutex<BTreeSet<(usize, usize)>>> = Arc::new(Mutex::new(BTreeSet::new()));
}

#[no_mangle]
pub fn update_global_bitmap() {
    let mut ans = 0;
    let mut u_maps:Vec<usize> = Vec::new();
    for i in 0..MAX_USER {
        for j in 0..MAX_THREAD {
            // let bitmap_val = MANAGER[i][j].lock().task_queue.lock().bitmap.get_val();
            let bitmap_val = BITMAPS[i][j].get_val();
            u_maps.push(bitmap_val);
            ans = ans | bitmap_val;
        }
    }
    let mask = get_right_one_mask(ans);
    PRIO_PIDS.lock().clear();
    for i in 0..MAX_USER {
        for j in 0..MAX_THREAD {
            if (u_maps[i * MAX_USER + j] & mask) != 0 {
                PRIO_PIDS.lock().insert((i, j));
            }
        }
    }
}

#[no_mangle]
pub fn check_prio_pid(pid: usize, thread_id: usize) -> bool {
    if PRIO_PIDS.lock().len() == 0 {
        return false;
    }
    if PRIO_PIDS.lock().contains(&(pid, thread_id)) {
        PRIO_PIDS.lock().remove(&(pid, thread_id));
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
pub fn add_callback(pid: usize, thread_id: usize, tid: usize) -> bool {
    let mut ex = MANAGER[pid][thread_id].clone();
    kprintln!("process {} add_callback {}", pid, tid);
    let res = ex.add_callback(TaskId::get_tid_by_usize(tid));
    if !res {
        CALLBACKS.lock().push(Arc::new(
            Mutex::new(
                CallbackTask {
                    pid,
                    thread_id,
                    tid,
                    is_valid: true,
                }
            )
        ));
        return false;
    }
    true
}

#[no_mangle]
pub fn update_callback() {
    let len = CALLBACKS.lock().len();
    for index in 0..len {
        let mut callback_task = CALLBACKS.lock().get(index).unwrap().clone();
        let mut ex = MANAGER[callback_task.lock().pid][callback_task.lock().thread_id].clone();
        let res = ex.add_callback(TaskId::get_tid_by_usize(callback_task.lock().tid));
        if res {
            callback_task.lock().is_valid = false;
        }
    }
    CALLBACKS.lock().retain(|x| x.lock().is_valid == true);
}

