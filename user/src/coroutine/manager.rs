use alloc::{boxed::Box, collections::VecDeque};
use lazy_static::*;

use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::{vec, vec::Vec};
use alloc::collections::BTreeSet;
use core::borrow::BorrowMut;
use spin::Mutex;

use super::{
    user_task::{UserTask},
};
use crate::coroutine::config::PRIO_NUM;
use crate::coroutine::scheduler::{Scheduler, TaskId, PrioScheduler};


lazy_static!{
    pub static ref MANAGER: Arc<Box<Manager<UserTask, PrioScheduler>>> =
        Arc::new(Box::new(Manager::new(PrioScheduler::new())));

}

pub struct Manager<T, S> where S: Scheduler {
    tasks: Mutex<BTreeMap<usize, Arc<T>>>,
    pub scheduler: Arc<Mutex<Box<S>>>,
    prio_map: Mutex<BTreeMap<usize, usize>>,
    current: Mutex<Option<usize>>,
    wait_task: Mutex<BTreeSet<usize>>,
}

impl<T, S: Scheduler> Manager<T, S> {
    /// 新建 Manager
    pub fn new(scheduler: S) -> Self {
        Manager {
            tasks: Mutex::new(BTreeMap::new()),
            scheduler: Arc::new(Mutex::new(Box::new(scheduler))),
            prio_map: Mutex::new(BTreeMap::new()),
            current: Mutex::new(None),
            wait_task: Mutex::new(BTreeSet::new()),
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
        let mut res = self.scheduler.lock();
        let prio_map = self.prio_map.lock();
        let op_prio = prio_map.get(&tid).copied();
        match op_prio {
            Some(prio) => {
                res.push(TaskId { tid, prio });
                self.wait_task.lock().remove(&tid);
            },
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
    pub fn empty(&self) -> bool { self.scheduler.lock().empty() && self.wait_task.lock().is_empty() }

    pub fn high_prio(&self) -> usize {
        self.scheduler.lock().high_prio()
    }

    pub fn add_wait_task(&self, tid: usize) {
        self.wait_task.lock().insert(tid);
    }
}







