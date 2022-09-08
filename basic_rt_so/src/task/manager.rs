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
use crate::config::{MAX_USER, PRIO_NUM, CAP};

lazy_static!{
    pub static ref MANAGER: Vec<Arc<Mutex<Box<Manager>>>> = 
        (0..MAX_USER).map(|_| 
            Arc::new(Mutex::new(Box::new(Manager::new())))
        ).collect::<Vec<Arc<Mutex<Box<Manager>>>>>();
}

pub struct Manager {
    pub tasks: BTreeMap<TaskId, Arc<UserTask>>,
    pub task_queue: Arc<Mutex<Box<TaskQueue>>>,
    pub callback_queue: Arc<Mutex<VecDeque<TaskId>>>,
}

impl Manager {
    /// 新建 Manager
    pub fn new() -> Self {
        Manager {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(Mutex::new(Box::new(TaskQueue::new()))),
            callback_queue: Arc::new(Mutex::new(VecDeque::<TaskId>::new())),
        }
    }
    /// 取出一个任务，首先会将回调队列中的所有任务唤醒，之后再取出优先级最高的任务
    pub fn fetch(&mut self) -> Option<Arc<UserTask>> {
        let callback_queue = self.callback_queue.clone();
        while callback_queue.lock().len() != 0 {
            // uprintln!("need wake");
            if let Some(id) = callback_queue.lock().pop_front() {
                CBTID.lock().add(id.get_val());
                self.tasks.get(&id).unwrap().waker.wake_by_ref();
            }
        }
        let tid = self.task_queue.lock().pop_tid();
        if tid.is_none() { return None; }
        if let Some(ret) = self.tasks.get(&tid.unwrap()) {
            return Some(ret.clone());
        } else {
            return None;
        }
    }
    /// 添加任务
    pub fn add(&mut self, task: Arc<UserTask>) {
        let tid = task.tid;
        let prio = task.prio;
        self.task_queue.lock().add_tid(tid, prio);
        self.tasks.insert(tid, task);
    }
    /// 判断 Manager 中是否有任务
    pub fn is_empty(&self) -> bool { self.task_queue.lock().is_empty() }
    /// 添加回调任务
    pub fn add_callback(&mut self, tid: TaskId) {
        self.callback_queue.lock().push_back(tid);
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




