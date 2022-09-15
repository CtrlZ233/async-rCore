
pub const ASYNC_SYSCALL_READ: usize = 2501;
pub const ASYNC_SYSCALL_WRITE: usize = 2502;

fn syscall6(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x13") args[3],
            in("x14") args[4],
            in("x15") args[5],
            in("x17") id
        );
    }
    ret
}



pub fn async_sys_read(fd: usize, buffer_ptr: usize, buffer_len: usize, tid: usize, pid: usize, thread_id: usize) -> isize {
    syscall6(ASYNC_SYSCALL_READ, [fd, buffer_ptr, buffer_len, tid, pid, thread_id])
}

pub fn async_sys_write(fd: usize, buffer_ptr: usize, buffer_len: usize, tid: usize, pid: usize, read_fd: usize) -> isize {
    syscall6(ASYNC_SYSCALL_WRITE, [fd, buffer_ptr, buffer_len, tid, pid, read_fd])
}


