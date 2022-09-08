const SYSCALL_WRITE: usize = 64;
const SYSCALL_SET_TIMER_INT: usize = 250;
const SYSCALL_YIELD: usize = 124;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}


pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_set_timer_interrupt(enable: usize) {
    syscall(SYSCALL_SET_TIMER_INT, [enable, 0, 0]);
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}


pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn enable_timer_interrupt() {
    sys_set_timer_interrupt(1);
}

pub fn disable_timer_interrupt() {
    sys_set_timer_interrupt(0);
}

pub fn uyield() {
    sys_yield();
}
