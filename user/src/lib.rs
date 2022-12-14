#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;
mod syscall6;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use alloc::vec::Vec;
use buddy_system_allocator::LockedHeap;
use syscall::*;

const USER_HEAP_SIZE: usize = 32768;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    let mut v: Vec<&'static str> = Vec::new();
    for i in 0..argc {
        let str_start =
            unsafe { ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile() };
        let len = (0usize..)
            .find(|i| unsafe { ((str_start + *i) as *const u8).read_volatile() == 0 })
            .unwrap();
        v.push(
            core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(str_start as *const u8, len)
            })
            .unwrap(),
        );
    }
    exit(main(argc, v.as_slice()));
}

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    panic!("Cannot find main!");
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
}
pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags.bits)
}
pub fn close(fd: usize) -> isize {
    sys_close(fd)
}
pub fn pipe(pipe_fd: &mut [usize]) -> isize {
    sys_pipe(pipe_fd)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}
pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str, args: &[*const u8]) -> isize {
    sys_exec(path, args)
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid_nb(pid: usize, exit_code: &mut i32) -> isize {
    sys_waitpid(pid as isize, exit_code as *mut _)
}

bitflags! {
    pub struct SignalFlags: i32 {
        const SIGINT    = 1 << 2;
        const SIGILL    = 1 << 4;
        const SIGABRT   = 1 << 6;
        const SIGFPE    = 1 << 8;
        const SIGSEGV   = 1 << 11;
    }
}

pub fn kill(pid: usize, signal: i32) -> isize {
    sys_kill(pid, signal)
}

pub fn sleep(sleep_ms: usize) {
    sys_sleep(sleep_ms);
}

pub fn thread_create(entry: usize, arg: usize) -> isize {
    sys_thread_create(entry, arg)
}
pub fn gettid() -> isize {
    sys_gettid()
}
pub fn waittid(tid: usize) -> isize {
    loop {
        match sys_waittid(tid) {
            -2 => {
                yield_();
            }
            exit_code => return exit_code,
        }
    }
}

pub fn mutex_create() -> isize {
    sys_mutex_create(false)
}
pub fn mutex_blocking_create() -> isize {
    sys_mutex_create(true)
}
pub fn mutex_lock(mutex_id: usize) {
    sys_mutex_lock(mutex_id);
}
pub fn mutex_unlock(mutex_id: usize) {
    sys_mutex_unlock(mutex_id);
}
pub fn semaphore_create(res_count: usize) -> isize {
    sys_semaphore_create(res_count)
}
pub fn semaphore_up(sem_id: usize) {
    sys_semaphore_up(sem_id);
}
pub fn semaphore_down(sem_id: usize) {
    sys_semaphore_down(sem_id);
}
pub fn condvar_create() -> isize {
    sys_condvar_create(0)
}
pub fn condvar_signal(condvar_id: usize) {
    sys_condvar_signal(condvar_id);
}
pub fn condvar_wait(condvar_id: usize, mutex_id: usize) {
    sys_condvar_wait(condvar_id, mutex_id);
}
pub fn create_desktop() {
    sys_create_desktop();
}

#[macro_export]
macro_rules! vstore {
    ($var_ref: expr, $value: expr) => {
        unsafe { core::intrinsics::volatile_store($var_ref as *const _ as _, $value) }
    };
}

#[macro_export]
macro_rules! vload {
    ($var_ref: expr) => {
        unsafe { core::intrinsics::volatile_load($var_ref as *const _ as _) }
    };
}

#[macro_export]
macro_rules! memory_fence {
    () => {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst)
    };
}
/************************ ????????????????????? ********************************************/
use core::mem::transmute;
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use core::task::{Context, Poll};
pub const SYMBOL_ADDR: *const usize = 0x8701a000usize as *const usize;

pub fn alloc_task_id() -> usize {
    unsafe {
        let alloc_task_id: fn() -> usize = transmute(*(SYMBOL_ADDR.add(10)) as usize);
        alloc_task_id()
    }
}

pub fn add_task_with_prority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize,
                             pid: usize, thread_id: usize, tid: usize) {
    unsafe {
        let add_task_with_prority: fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize,
                                      pid: usize, thread_id: usize, tid: usize) = transmute(*SYMBOL_ADDR as usize);
        add_task_with_prority(future, prio, pid, thread_id, tid);
    }
} 

pub fn user_thread_main(pid: usize, thread_id: usize) {
    unsafe {
        let user_thread_main: fn(pid: usize, thread_id: usize) = transmute(*(SYMBOL_ADDR.add(2)) as usize);
        user_thread_main(pid, thread_id);
    }
}

pub fn check_callback(tid: usize) -> bool {
    unsafe {
        let check_callback: fn(tid: usize) -> bool = transmute(*(SYMBOL_ADDR.add(9)) as usize);
        check_callback(tid)
    }
}



/************************ ?????????????????? **************************************/

use syscall6::{async_sys_read, async_sys_write};
pub use syscall6::{ASYNC_SYSCALL_READ, ASYNC_SYSCALL_WRITE};
pub fn async_read(fd: usize, buffer_ptr: usize, buffer_len: usize, tid: usize, pid: usize, thread_id: usize) -> isize {
    async_sys_read(fd, buffer_ptr, buffer_len, tid, pid, thread_id)
}

pub fn async_write(fd: usize, buffer_ptr: usize, buffer_len: usize, tid: usize, pid: usize, read_fd: usize) -> isize {
    async_sys_write(fd, buffer_ptr, buffer_len, tid, pid, read_fd)
}

// ??????????????????
pub struct AsyncCall {
    call_type: usize,   // ???????????????????????? / ???
    fd: usize,          // ???????????????
    buffer_ptr: usize,  // ???????????????
    buffer_len: usize,  // ???????????????
    tid: usize,         // ????????????????????????????????? id
    pid: usize,         // ?????? id
    thread_id: usize,   // ??????id
    cnt: usize,         
}

impl AsyncCall {
    pub fn new( call_type: usize, fd: usize, buffer_ptr: usize, buffer_len: usize, tid: usize,
                pid: usize, thread_id: usize) -> Self {
        Self { 
            call_type, fd, buffer_ptr, buffer_len, tid, pid, thread_id, cnt: 0
        }
    }
}

impl Future for AsyncCall {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // submit async task to kernel and return immediately
        if self.cnt == 0 {
            match self.call_type {
                ASYNC_SYSCALL_READ => async_sys_read(self.fd, self.buffer_ptr, self.buffer_len, self.tid, self.pid, self.thread_id),
                ASYNC_SYSCALL_WRITE => async_sys_write(self.fd, self.buffer_ptr, self.buffer_len, self.tid, self.pid, self.thread_id),
                _ => {0},
            };
            self.cnt += 1;
        }

        if check_callback(self.tid) {
            return Poll::Ready(());
        } else {
            return Poll::Pending;
        }
    
    }
}
