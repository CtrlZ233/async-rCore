use alloc::{vec::Vec, collections::VecDeque};
use super::{bitmap::BitMap, user_task::TaskId};
use crate::config::PRIO_NUM;

pub struct TaskQueue {
    queue: Vec<VecDeque<TaskId>>,
    pub bitmap: BitMap,
    pub task_num: usize,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            queue: (0..PRIO_NUM).map(|_| VecDeque::new() ).collect::<Vec<VecDeque<TaskId>>>(),
            bitmap: BitMap::new(),
            task_num: 0,
        }
    }

    pub fn add_tid(&mut self,  tid: TaskId, prio: usize) {
        self.queue[prio].push_back(tid);
        self.bitmap.update(prio, true);
        self.task_num += 1;
    }

    pub fn pop_tid(&mut self) -> Option<TaskId> {
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

    pub fn is_empty(&self) -> bool {
        self.task_num == 0
    }
}