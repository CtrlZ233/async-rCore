use super::{ProcessControlBlock, TaskControlBlock};
use crate::sync::UPSafeCell;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::Ordering::Relaxed;
use lazy_static::*;
use spin::Mutex;

pub struct TaskManager {
    process_queue: VecDeque<Arc<ProcessControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            process_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        if let Some(process) = task.process.upgrade() {
            process.inner_exclusive_access().ready_tasks.push(task);
        }
    }

    pub fn push(&mut self, process: Arc<ProcessControlBlock>) {
        self.process_queue.push_back(process);
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // println!("task size: {}", self.process_queue[0].inner_exclusive_access().ready_tasks.len());
        let mut index: isize = -1;
        let mut max_prio: isize = -1;
        for (i, process) in self.process_queue.iter().enumerate() {
            let mut process_inner = process.inner_exclusive_access();
            if process_inner.ready_tasks.is_empty() {
                continue;
            }
            let prio = process_inner.get_prio();
            // println!("pid: {}, prio: {}", process.pid.0, prio);
            if prio as isize > max_prio {
                index = i as isize;
                max_prio = prio as isize;
            }
        }
        // println!("index: {}, task size: {}",index, self.process_queue[0].inner_exclusive_access().ready_tasks.len());
        if index != -1 {
            let task = self.process_queue[index as usize].inner_exclusive_access().ready_tasks.pop();
            let process = self.process_queue[index as usize].clone();
            self.process_queue.remove(index as usize);
            self.process_queue.push_back(process);
            return task;
        }
        None

    }
    pub fn remove(&mut self, process: Arc<ProcessControlBlock>) {
        if let Some((id, _)) = self.process_queue
            .iter()
            .enumerate()
            .find(|(_, t)| Arc::as_ptr(t) == Arc::as_ptr(&process)) {
                self.process_queue.remove(id);
        }
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
    pub static ref PID2PCB: UPSafeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> =
        unsafe { UPSafeCell::new(BTreeMap::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn add_process(process: Arc<ProcessControlBlock>) {
    TASK_MANAGER.exclusive_access().push(process);
}

pub fn remove_process(process: Arc<ProcessControlBlock>) {
    // println!("remove process: {}", process.pid.0);
    TASK_MANAGER.exclusive_access().remove(process);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}

pub fn pid2process(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    let map = PID2PCB.exclusive_access();
    map.get(&pid).map(Arc::clone)
}

pub fn insert_into_pid2process(pid: usize, process: Arc<ProcessControlBlock>) {
    PID2PCB.exclusive_access().insert(pid, process);
}

pub fn remove_from_pid2process(pid: usize) {
    let mut map = PID2PCB.exclusive_access();
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}
