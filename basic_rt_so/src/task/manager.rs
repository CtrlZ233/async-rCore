use alloc::{boxed::Box, collections::VecDeque};
use lazy_static::*;

use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::{vec, vec::Vec};
use spin::Mutex;

use super::{
    user_task::{TaskId, UserTask},
    bitmap::BitMap,
    task_queue::TaskQueue,
};
use crate::config::{MAX_USER, MAX_THREAD, PRIO_NUM, CAP};

lazy_static!{
    pub static ref BITMAPS: Vec<Vec<BitMap>> = (0..MAX_USER).map(|_|
        (0..MAX_THREAD).map(|_|
            BitMap::new()
        ).collect::<Vec<BitMap>>()
    ).collect::<Vec<Vec<BitMap>>>();
}

pub struct CallbackTask {
    pub pid: usize,
    pub thread_id: usize,
    pub tid: usize,
    pub is_valid: bool,
}

lazy_static!{
    pub static ref CALLBACKS: Mutex<Vec<Arc<Mutex<CallbackTask>>>> = Mutex::new(Vec::new());
}


lazy_static!{
    pub static ref MANAGER: Vec<Vec<Arc<Box<Manager>>>> = (0..MAX_USER).map(|pid|
        (0..MAX_THREAD).map(|thread_id|
            Arc::new(Box::new(Manager::new(pid, thread_id)))
        ).collect::<Vec<Arc<Box<Manager>>>>()
    ).collect::<Vec<Vec<Arc<Box<Manager>>>>>();
}

pub struct Manager {
    pub tasks: Mutex<BTreeMap<TaskId, Arc<UserTask>>>,
    pub task_queue: Arc<Mutex<Box<TaskQueue>>>,
    pub callback_queue: Arc<Mutex<VecDeque<TaskId>>>,
}

impl Manager {
    /// 新建 Manager
    pub fn new(pid: usize, thread_id: usize) -> Self {
        Manager {
            tasks: Mutex::new(BTreeMap::new()),
            task_queue: Arc::new(Mutex::new(Box::new(TaskQueue::new(pid, thread_id)))),
            callback_queue: Arc::new(Mutex::new(VecDeque::<TaskId>::new())),
        }
    }

    pub fn kernel_fetch(&self) -> Option<Arc<UserTask>> {
        let tid = self.task_queue.lock().pop_tid();
        if tid.is_none() { return None; }
        if let Some(ret) = self.tasks.lock().get(&tid.unwrap()) {
            return Some(ret.clone());
        } else {
            return None;
        }
    }

    /// 取出一个任务，首先会将回调队列中的所有任务唤醒，之后再取出优先级最高的任务
    pub fn user_fetch(&self) -> Option<Arc<UserTask>> {
        let callback_queue = self.callback_queue.clone();
        while callback_queue.lock().len() != 0 {
            // uprintln!("need wake");
            if let Some(id) = callback_queue.lock().pop_front() {
                CBTID.lock().add(id.get_val());
                self.tasks.lock().get(&id).unwrap().waker.wake_by_ref();
            }
        }
        self.kernel_fetch()
    }
    /// 添加任务
    pub fn add(&self, task: Arc<UserTask>) {
        let tid = task.tid;
        let prio = task.prio;
        self.task_queue.lock().add_tid(tid, prio);
        self.tasks.lock().insert(tid, task);
    }
    /// 判断 Manager 中是否有任务
    pub fn is_empty(&self) -> bool { self.task_queue.lock().is_empty() }
    /// 添加回调任务
    pub fn add_callback(&self, tid: TaskId) -> bool {
        let mut op_queue = self.callback_queue.try_lock();
        if let Some(mut queue) = op_queue {
            queue.push_back(tid);
            return true;
        }
        false
    }
}


lazy_static! {
    pub static ref CBTID: Arc<Mutex<CBTid>> = Arc::new(Mutex::new(CBTid::new()));
}

pub struct CBTid {
    tids: Vec<bool>,
}

impl CBTid {
    pub fn new() -> Self {
        Self { tids: vec![false; CAP], }
    }

    pub fn add(&mut self, t: usize) {
        self.tids[t] = true;
    }

    pub fn contains_tid(&mut self, t: usize) -> bool {
        if self.tids[t] == true {
            self.tids[t] = false;
            return true;
        }
        false
    }
}

#[no_mangle]
pub fn check_callback(tid: usize) -> bool {
    CBTID.lock().contains_tid(tid)
}




