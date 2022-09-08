//! SBI console driver, for text output

use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// print string macro
#[macro_export]
macro_rules! kprint {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

/// println string macro
#[macro_export]
macro_rules! kprintln {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}


/*************************** 用户态print *************************************/
const STDOUT: usize = 1;

use crate::syscall::write;

struct UserStdout;

impl Write for UserStdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn userprint(args: fmt::Arguments) {
    UserStdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! uprint {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::userprint(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! uprintln {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::userprint(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}