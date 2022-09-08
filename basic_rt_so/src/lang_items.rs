use super::kprintln;
use super::sbi::shutdown;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("{}", info);
    shutdown()
}
