use alloc::collections::VecDeque;
use core::convert::Infallible;
use lazy_static::*;
use spin::Mutex;

pub const DEFAULT_TX_BUFFER_SIZE: usize = 1_000;
pub const DEFAULT_RX_BUFFER_SIZE: usize = 1_000;

trait Read<T> {
    type Error;
    fn try_read(&mut self) -> nb::Result<u8, Self::Error>;
}

trait Write<T> {
    type Error;
    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error>;
    fn try_flush(&mut self) -> nb::Result<(), Self::Error>;
}

use uart8250::{InterruptType, MmioUart8250};

mod serial_config {
    use super::MmioUart8250;
    pub type SerialHardware = MmioUart8250<'static>;
    pub const FIFO_DEPTH: usize = 16;
    pub const SERIAL_NUM: usize = 4;
    pub const SERIAL_BASE_ADDRESS: usize = 0x1000_2000;
    pub const SERIAL_ADDRESS_STRIDE: usize = 0x1000;
    pub fn irq_to_serial_id(irq: u16) -> usize {
        match irq {
            12 => 0,
            13 => 1,
            14 => 2,
            15 => 3,
            _ => 0,
        }
    }
}


pub use serial_config::*;

pub fn get_base_addr_from_irq(irq: u16) -> usize {
    SERIAL_BASE_ADDRESS + irq_to_serial_id(irq) * SERIAL_ADDRESS_STRIDE
}
pub struct BufferedSerial {
    pub hardware: SerialHardware,
    pub rx_buffer: VecDeque<u8>,
    pub tx_buffer: VecDeque<u8>,
    pub rx_count: usize,
    pub tx_count: usize,
    pub intr_count: usize,
}

impl BufferedSerial {
    pub fn new(base_address: usize) -> Self {
        unsafe {
            BufferedSerial {
                hardware: SerialHardware::new(base_address),
                rx_buffer: VecDeque::with_capacity(DEFAULT_RX_BUFFER_SIZE),
                tx_buffer: VecDeque::with_capacity(DEFAULT_TX_BUFFER_SIZE),
                rx_count: 0,
                tx_count: 0,
                intr_count: 0,
            }
        }

    }

    pub fn hardware_init(&mut self, baud_rate: usize) {
        let hardware = &mut self.hardware;
        hardware.write_ier(0);
        let _ = hardware.is_carrier_detect();
        let _ = hardware.is_carrier_detect();
        hardware.init(100_000_000, baud_rate);
    }

    pub fn interrupt_handler(&mut self) {
        let hardware = &self.hardware;
        if let Some(int_type) = hardware.read_interrupt_type() {
            self.intr_count += 1;
            match int_type {
                InterruptType::ReceivedDataAvailable | InterruptType::Timeout => {
                    // trace!("Received data available");
                    while let Some(ch) = hardware.read_byte() {
                        if self.rx_buffer.len() < DEFAULT_TX_BUFFER_SIZE {
                            self.rx_buffer.push_back(ch);
                            self.rx_count += 1;
                        } else {
                            println!("Serial rx buffer overflow!");
                        }
                    }
                }
                InterruptType::TransmitterHoldingRegisterEmpty => {
                    // trace!("TransmitterHoldingRegisterEmpty");
                    for _ in 0..FIFO_DEPTH {
                        if let Some(ch) = self.tx_buffer.pop_front() {
                            hardware.write_byte(ch);
                            self.tx_count += 1;
                        } else {
                            hardware.disable_transmitter_holding_register_empty_interrupt();
                            break;
                        }
                    }
                }
                InterruptType::ModemStatus => {
                }
                _ => {
                    println!("[SERIAL] {:?} not supported!", int_type);
                }
            }
        }
    }
}


lazy_static! {
    pub static ref BUFFERED_SERIAL: [Mutex<BufferedSerial>; SERIAL_NUM] =
        array_init::array_init(|i| Mutex::new(BufferedSerial::new(
            SERIAL_BASE_ADDRESS + i * SERIAL_ADDRESS_STRIDE,
        )));
}


pub fn init() {
    for serial_id in 0..2 {
        BUFFERED_SERIAL[serial_id].lock().hardware_init(115200);
    }
    for serial_id in 2..SERIAL_NUM {
        BUFFERED_SERIAL[serial_id].lock().hardware_init(6_250_000);
    }
}


pub fn handle_interrupt(irq: u16) {
    BUFFERED_SERIAL[irq_to_serial_id(irq)]
        .lock()
        .interrupt_handler();
}



pub fn serial_putchar(serial_id: usize, c: u8) {
    let _ = BUFFERED_SERIAL[serial_id].lock().try_write(c);
}

pub fn serial_getchar(serial_id: usize) -> nb::Result<u8, Infallible> {
    BUFFERED_SERIAL[serial_id].lock().try_read()
}


impl Write<u8> for BufferedSerial {
    type Error = Infallible;

    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let serial = &mut self.hardware;
        // if serial.is_transmitter_holding_register_empty() {
        //     for _ in 0..FIFO_DEPTH {
        //         if let Some(ch) = self.tx_buffer.pop_front() {
        //             serial.write_byte(ch);
        //             self.tx_count += 1;
        //         }
        //     }
        // }

        if self.tx_buffer.len() < DEFAULT_TX_BUFFER_SIZE {
            self.tx_buffer.push_back(word);
            if !serial.is_transmitter_holding_register_empty_interrupt_enabled() {
                serial.enable_transmitter_holding_register_empty_interrupt();
            }
        } else {
            println!("Serial tx buffer overflow!");
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }

    fn try_flush(&mut self) -> nb::Result<(), Self::Error> {
        todo!()
    }
}

impl Read<u8> for BufferedSerial {
    type Error = Infallible;

    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        if let Some(ch) = self.rx_buffer.pop_front() {
            Ok(ch)
        } else {
            #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
            {
                // Drain UART Rx FIFO
                while let Some(ch_read) = self.hardware.read_byte() {
                    self.rx_buffer.push_back(ch_read);
                    self.rx_count += 1;
                }
            }
            self.rx_buffer.pop_front().ok_or(nb::Error::WouldBlock)
        }
    }
}