use core::arch::asm;

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Execute a closure without interrupts
fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Save current interrupt flag state
    let flags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) flags,
            options(nomem, preserves_flags)
        );
    }

    // Disable interrupts
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }

    let result = f();

    // Restore interrupt flag if it was previously enabled
    if flags & 0x200 != 0 {
        unsafe {
            core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
        }
    }

    result
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
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
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}

/// Logs a message to the serial port with a given log level prefix.
///
/// # Examples
/// ```
/// serial_log!("[INFO] ", "Hello, world!");
/// serial_log!("[DEBUG] ", "Value: {}", 42);
/// ```
#[macro_export]
macro_rules! serial_log {
    ($level:expr, $msg:expr) => {
        $crate::serial::serial_write_str($level);
        $crate::serial::serial_write_str($msg);
        $crate::serial::serial_write_str("\r\n");
    };
    ($level:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::serial::serial_write_str($level);
        $crate::serial::serial_write_str(&alloc::format!($fmt, $($arg)*));
        $crate::serial::serial_write_str("\r\n");
    };
}

/// Logs a hexadecimal value to the serial port with a given log level prefix.
///
/// # Examples
/// ```
/// serial_log_hex!("[INFO] ", 0xdeadbeef);
/// ```
#[macro_export]
macro_rules! serial_log_hex {
    ($level:expr, $value:expr) => {
        $crate::serial::serial_write_str($level);
        $crate::serial::serial_write_str("0x");
        $crate::serial::serial_write_hex($value);
        $crate::serial::serial_write_str("\r\n");
    };
}

const SERIAL_PORT: u16 = 0x3F8; // COM1

/// Port I/O functions
unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {    
        asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

/// Writes a single byte to the serial port.
///
/// Blocks until the port is ready to accept a byte.
pub fn serial_write_byte(byte: u8) {
    unsafe {
        // Wait for transmit holding register to be empty
        while (inb(SERIAL_PORT + 5) & 0x20) == 0 {}
        // Write the byte
        outb(SERIAL_PORT, byte);
    }
}

/// Writes a string to the serial port.
///
/// Each byte of the string is sent individually.
pub fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

/// Writes a hexadecimal representation of a `u64` value to the serial port.
///
/// Does not include a `0x` prefix.
pub fn serial_write_hex(mut value: u64) {
    let mut buf = [0u8; 16];
    let mut i = buf.len();

    if value == 0 {
        serial_write_str("0");
        return;
    }

    while value != 0 {
        i -= 1;
        let digit = (value & 0xF) as u8;
        buf[i] = match digit {
            0..=9 => b'0' + digit,
            10..=15 => b'A' + (digit - 10),
            _ => b'?', // Should not happen
        };
        value >>= 4;
    }

    serial_write_str(core::str::from_utf8(&buf[i..]).unwrap());
}

/// Logs an info-level message to the serial port.
///
/// # Examples
/// ```
/// serial::info("System started");
/// ```
pub fn info(text: &str) {
    serial_log!("[INFO] ", "{}", text);
}

/// Logs an info-level hexadecimal value to the serial port.
///
/// # Examples
/// ```
/// serial::info_hex(0xdeadbeef);
/// ```
pub(crate) fn info_hex(value: u64) {
    serial_log_hex!("[INFO] ", value);
}

/// Logs an error-level message to the serial port.
///
/// # Examples
/// ```
/// serial::error("An error occurred");
/// ```
pub(crate) fn error(text: &str) {
    serial_log!("[ERROR] ", "{}", text);
}