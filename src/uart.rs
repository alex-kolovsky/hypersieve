use core::ptr::{read_volatile, write_volatile};
use lazy_static::lazy_static;
use spin::Mutex;

pub const QEMU_UART_ADDR: u64 = 0x1000_0000;

lazy_static! {
    pub static ref UART: Mutex<Uart> = Mutex::new(Uart::new(QEMU_UART_ADDR));
}

pub struct Uart {
    base_addr: u64,
}

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.put(byte);
        }
        Ok(())
    }
}

impl Uart {
    pub fn new(base_addr: u64) -> Self {
        let uart = Self { base_addr };
        uart.init();
        uart
    }

    fn init(&self) {
        let uart_ptr = self.base_addr as *mut u8;
        unsafe {
            // Enable hardware FIFO buffers in FCR
            write_volatile(uart_ptr.offset(2), 1 << 0);

            // Enable receiver buffer interrupts in IER
            write_volatile(uart_ptr.offset(1), 1 << 1);

            // Set DLAB in LCR to enable access to DLL and DLM registers
            write_volatile(uart_ptr.offset(3), 1 << 7);

            // Divisor = ( system_clock_speed ) / ( 16 * desired_baud_rate )
            let divisor = 1_000_000 / (16 * 9600);

            let divisor_least: u8 = (divisor & 0xff).try_into().unwrap();
            let divisor_most: u8 = (divisor >> 8).try_into().unwrap();

            // Set DLL and DLM
            write_volatile(uart_ptr.offset(0), divisor_least);
            write_volatile(uart_ptr.offset(1), divisor_most);

            // Set UART word length as 8 bits in LCR
            // and unset DLAB to enable receiver and transmitter holding registers
            const UART_WORD_LENGTH_8: u8 = 1 << 0 | 1 << 1;
            write_volatile(uart_ptr.offset(3), UART_WORD_LENGTH_8);

            // Wait until THR gets ready to write
            while (read_volatile(uart_ptr.offset(5)) & 1 << 5) == 0 {}
        }
    }

    pub fn put(&mut self, byte: u8) {
        let uart_ptr = self.base_addr as *mut u8;

        unsafe {
            write_volatile(uart_ptr.offset(0), byte);
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)))
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    UART.lock().write_fmt(args).unwrap();
}
