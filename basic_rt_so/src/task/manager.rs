use alloc::{boxed::Box, collections::VecDeque};
use lazy_static::*;

use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::{vec, vec::Vec};
use core::borrow::BorrowMut;
use spin::Mutex;

use super::{
    user_task::{UserTask},
    bitmap::BitMap,
};
use crate::config::{MAX_USER, MAX_THREAD, PRIO_NUM, CAP};
use crate::task::schduler::{Scheduler, TaskId, PrioScheduler};

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
    tasks: Mutex<BTreeMap<usize, Arc<T>>>,
    pub scheduler: Arc<Mutex<Box<S>>>,
    prio_map: Mutex<BTreeMap<usize, usize>>,
    current: Mutex<Option<usize>>,
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
        if let Some(tid) = task_id {
            *self.current.lock() = Some(tid.tid);
            return self.get(tid.tid);
        }
        None
    }

    pub fn get(&self, tid: usize) -> Option<Arc<T>> {
        if let Some(ret) = self.tasks.lock().get(&tid) {
            return Some(ret.clone());
        }
        None
    }

    pub fn re_back(&self, tid: usize) -> bool {
        let res = self.scheduler.try_lock();
        let prio_map = self.prio_map.try_lock();
        if res.is_none() {
            kprintln!("acquire scheduler failed.");
            return false;
        }
        if prio_map.is_none() {
            kprintln!("acquire prio_map failed.");
            return false;
        }
        let op_prio = prio_map.unwrap().get(&tid).copied();
        match op_prio {
            Some(prio) => res.unwrap().push(TaskId{tid, prio}),
            _ => return false,
        }
        true
    }

    pub fn add(&self, task: Arc<T>, tid: usize, prio: usize) {
        self.scheduler.lock().push(TaskId{tid, prio});
        self.tasks.lock().insert(tid, task);
        self.prio_map.lock().insert(tid, prio);
    }

    pub fn remove(&self, tid: usize) {
        self.tasks.lock().remove(&tid);
        self.prio_map.lock().remove(&tid);
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




