use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::port::Port;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

const SERIAL_PORT: u16 = 0x3F8; // COM1

pub fn serial_write_byte(byte: u8) {
    unsafe {
        let mut line_status = Port::<u8>::new(SERIAL_PORT + 5);
        while (line_status.read() & 0x20) == 0 {}
        let mut data = Port::new(SERIAL_PORT);
        data.write(byte);
    }
}

pub fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

pub fn info(text: &str) {
    serial_write_str("[INFO] ");
    serial_write_str(text);
    serial_write_str("\n");
}

pub fn error(text: &str) {
    serial_write_str("[ERROR] ");
    serial_write_str(text);
    serial_write_str("\n");
}