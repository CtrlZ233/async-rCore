use crate::trap::{push_trap_record, UserTrapRecord, USER_EXT_INT_MAP};
use crate::uart;
use rv_plic::{Priority, PLIC};

pub const PLIC_BASE: usize = 0xc00_0000;
pub const PLIC_PRIORITY_BIT: usize = 3;

pub type Plic = PLIC<{ PLIC_BASE }, { PLIC_PRIORITY_BIT }>;

pub fn get_context(hart_id: usize, mode: char) -> usize {
    const MODE_PER_HART: usize = 3;
    hart_id * MODE_PER_HART
        + match mode {
        'M' => 0,
        'S' => 1,
        'U' => 2,
        _ => panic!("Wrong Mode"),
    }
}

pub fn init() {
    Plic::set_priority(12, Priority::lowest());
    Plic::set_priority(13, Priority::lowest());
    Plic::set_priority(14, Priority::lowest());
    Plic::set_priority(15, Priority::lowest());
}


pub fn init_hart(hart_id: usize) {
    let context = get_context(hart_id, 'S');
    Plic::enable(context, 12);
    Plic::enable(context, 13);
    Plic::enable(context, 14);
    Plic::enable(context, 15);
    Plic::set_threshold(context, Priority::any());
}


pub fn handle_external_interrupt(hart_id: usize) {
    let context = get_context(hart_id, 'S');
    while let Some(irq) = Plic::claim(context) {
        let mut can_user_handle = false;
        if let Some(pid) = USER_EXT_INT_MAP.lock().get(&irq) {
            println!("[PLIC] irq {:?} mapped to pid {:?}", irq, pid);
            if push_trap_record(
                *pid,
                UserTrapRecord {
                    // User External Interrupt
                    cause: 8,
                    message: irq as usize,
                },
            )
                .is_ok()
            {
                can_user_handle = true;
            }
            // prioritize_task(*pid);
        }
        if !can_user_handle {
            match irq {
                12 | 13 | 14 | 15 => {
                    uart::handle_interrupt(irq);
                    println!("[PLIC] irq {:?} handled by kenel", irq);
                }
                _ => {
                    println!("[PLIC]: irq {:?} not supported!", irq);
                }
            }
            Plic::complete(context, irq);
        }
    }
}