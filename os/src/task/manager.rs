use super::{ProcessControlBlock, TaskControlBlock};
use crate::sync::UPSafeCell;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use lazy_static::*;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // self.ready_queue.pop_front()
        let n = self.ready_queue.len();
        if n == 0 { return None; }
        let mut peek;
        let mut cnt = 0;
        loop {
            peek = self.ready_queue.pop_front().unwrap();
            let pid = peek.process.upgrade().unwrap().getpid();
            let thread_id = peek.inner_exclusive_access()
                .res
                .as_ref()
                .unwrap()
                .tid;
            if crate::lkm::check_prio_pid(pid, thread_id) {
                // pid 为 0，且 PRIO_PIDS 中存在，表示内核有协程需要执行，返回到 run_task 的 loop 循环的 else 分支来执行
                if pid == 0 {
                    self.ready_queue.push_back(peek);
                    return None;
                }
                return Some(peek); 
            }
            self.ready_queue.push_back(peek);
            cnt += 1;
            if cnt >= n { break; }
        }
        self.ready_queue.pop_front()
    }
    pub fn remove(&mut self, task: Arc<TaskControlBlock>) {
        if let Some((id, _)) = self.ready_queue
            .iter()
            .enumerate()
            .find(|(_, t)| Arc::as_ptr(t) == Arc::as_ptr(&task)) {
                self.ready_queue.remove(id);
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

pub fn remove_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().remove(task);
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
