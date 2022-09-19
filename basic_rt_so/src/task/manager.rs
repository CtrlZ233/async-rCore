use alloc::{boxed::Box, collections::VecDeque};
use lazy_static::*;

use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::{vec, vec::Vec};
use spin::Mutex;

use super::{
    user_task::{UserTask},
    bitmap::BitMap,
};
use crate::config::{MAX_USER, MAX_THREAD, PRIO_NUM, CAP};
use crate::task::task_queue::{Scheduler, TaskId, PrioScheduler};

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
    pub static ref MANAGER: Vec<Vec<Arc<Box<Manager<UserTask, PrioScheduler>>>>> =
    (0..MAX_USER).map(|_|
        (0..MAX_THREAD).map(|_|
            Arc::new(Box::new(Manager::new(PrioScheduler::new())))
        ).collect::<Vec<Arc<Box<Manager<UserTask, PrioScheduler>>>>>()
    ).collect::<Vec<Vec<Arc<Box<Manager<UserTask, PrioScheduler>>>>>>();
}

pub struct Manager<T, S> where S: Scheduler {
    pub tasks: Mutex<BTreeMap<usize, Arc<T>>>,
    pub scheduler: Arc<Mutex<Box<S>>>,
    pub prio_map: Mutex<BTreeMap<usize, usize>>,
    pub current: Mutex<Option<usize>>,
}

impl<T, S: Scheduler> Manager<T, S> {
    /// 新建 Manager
    pub fn new(scheduler: S) -> Self {
        Manager {
            tasks: Mutex::new(BTreeMap::new()),
            scheduler: Arc::new(Mutex::new(Box::new(scheduler))),
            prio_map: Mutex::new(BTreeMap::new()),
            current: Mutex::new(None),
        }
    }

    pub fn fetch(&self) -> Option<Arc<T>> {
        let task_id = self.scheduler.lock().pop();
        match task_id {
            Some(id) => {
                *self.current.lock() = Some(id.tid);
                if let Some(ret) = self.tasks.lock().get(&id.tid) {
                    return Some(ret.clone());
                }
            }
            _ => {}
        }
        None
    }

    pub fn re_back(&self, tid: usize) -> bool {
        let res = self.scheduler.try_lock();
        let prio_map = self.prio_map.try_lock();
        if prio_map.is_none() {
            return false;
        }
        let op_prio = prio_map.unwrap().get(&tid).copied();
        match (res, op_prio) {
            (Some(mut lock), Some(prio)) => lock.push(TaskId{tid, prio}),
            _ => return false,
        }
        true
    }

    /// 取出一个任务，首先会将回调队列中的所有任务唤醒，之后再取出优先级最高的任务
    // pub fn user_fetch(&self) -> Option<Arc<T>> {
    //     let callback_queue = self.callback_queue.clone();
    //     while callback_queue.lock().len() != 0 {
    //         // uprintln!("need wake");
    //         if let Some(id) = callback_queue.lock().pop_front() {
    //             CBTID.lock().add(id.get_val());
    //             self.tasks.lock().get(&id).unwrap().waker.wake_by_ref();
    //         }
    //     }
    //     self.kernel_fetch()
    // }
    /// 添加任务
    pub fn add(&self, task: Arc<T>, tid: TaskId) {
        self.scheduler.lock().push(tid);
        self.tasks.lock().insert(tid.tid, task);
        self.prio_map.lock().insert(tid.tid, tid.prio);
    }

    /// 判断 Manager 中是否有任务
    pub fn empty(&self) -> bool { self.scheduler.lock().empty() }
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




