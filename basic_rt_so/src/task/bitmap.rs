use core::slice::SliceIndex;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;
use bit_field::BitField;
use crate::config::PRIO_NUM;



/// 协程优先级位图
pub struct BitMap(pub AtomicUsize);

impl BitMap {
    pub fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub fn update(&self, prio: usize, val: bool) {
        // self.0.set_bit(prio, val);
        let mut tmp = self.0.load(Relaxed);
        tmp.set_bit(prio, val);
        self.0.store(tmp, Relaxed);
    }


    pub fn get(&self, id: usize) -> bool {
        self.0.load(Relaxed).get_bit(id)
    }
    /// 获取最高优先级
    pub fn get_priority(&self) -> usize {
        let cur = self.0.load(Relaxed);
        for i in 0..PRIO_NUM {
            if cur.get_bit(i) {
                return i;
            }
        }
        PRIO_NUM
    }
    /// 
    pub fn get_val(&self) -> usize {
        self.0.load(Relaxed)
    }
}