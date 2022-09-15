#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use user_lib::*;
use alloc::boxed::Box;

async fn test1() {
    println!("4333333");
}
async fn test2() {
    println!("443333333333334");
}

#[no_mangle]
pub fn main() -> i32 {
    let pid = getpid() as usize;
    println!("pid is {}, user test>>>>>>>", pid);
    async_write(0, 0, 0, 0, 0, 0);
    async_read(0, 0, 0, 0, 0, 0);
    let tid1 = alloc_task_id();
    add_task_with_prority(Box::pin(test1()), 0, pid, tid1);
    let tid2 = alloc_task_id();
    add_task_with_prority(Box::pin(test2()), 1, pid, tid2);
    // 执行一段时间空操作使得用户进程的时间片用完，进入时钟中断
    for _i in 0..(1 << 25) {
        unsafe {
            core::arch::asm!(
                "addi x0,x0,0"
            );
        }
    }
    user_thread_main(pid);
    0
}