use alloc::{vec::Vec, collections::VecDeque};
use core::sync::atomic::{AtomicUsize, Ordering};
use super::{bitmap::BitMap};
use crate::config::PRIO_NUM;
use crate::task::manager::BITMAPS;


#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct TaskId {
    pub tid: usize,
    pub prio: usize,
}

impl TaskId {
    pub(crate) fn generate() -> usize {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        if id > usize::MAX / 2 {
            // TODO: 不让系统 Panic
            panic!("too many tasks!")
        }
        id
    }
}

pub trait Scheduler: 'static {
    // 如果 tid 不存在，表明将一个新线程加入线程调度
    // 否则表明一个已有的线程要继续运行
    fn push(&mut self, tid: TaskId);

    // 从若干可运行线程中选择一个运行
    fn pop(&mut self) -> Option<TaskId>;

    fn empty(&self) -> bool;
}

pub struct PrioScheduler {
    queue: Vec<VecDeque<TaskId>>,
    pub bitmap: BitMap,
    pub task_num: usize,
}

impl PrioScheduler {
    pub fn new() -> Self {
        Self {
            queue: (0..PRIO_NUM).map(|_| VecDeque::new() ).collect::<Vec<VecDeque<TaskId>>>(),
            bitmap: BitMap::new(),
            task_num: 0,
        }
    }
}

impl Scheduler for PrioScheduler {
    fn push(&mut self,  tid: TaskId) {
        let prio = tid.prio;
        self.queue[prio].push_back(tid);
        self.bitmap.update(prio, true);
        self.task_num += 1;
    }

    fn pop(&mut self) -> Option<TaskId> {
        if self.task_num == 0 {
            return None;
        }
        let prio = self.bitmap.get_priority();
        if prio == PRIO_NUM {
            None
        } else {
            self.task_num -= 1;
            let ret = self.queue[prio].pop_front();
            if self.queue[prio].len() == 0 { self.bitmap.update(prio, false); }
            ret
        }
    }

    fn empty(&self) -> bool {
        self.task_num == 0
    }
}