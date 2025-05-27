use uart_16550::SerialPort;
use x86_64::instructions::port::Port;

const COM3_PORT: u16 = 0x03E8; // COM3 address

/// Initialize UART with 115200 baud rate
pub fn init_serial() {
    let mut serial_port = unsafe { SerialPort::new(COM3_PORT) };
    serial_port.init(); // Default baud rate is 38400

    // Manually set baud rate (115200)
    let divisor = 1; // 115200 / 115200 = 1
    unsafe {
        Port::<u8>::new(COM3_PORT + 3).write(0x80); // Enable Divisor Latch
        Port::<u8>::new(COM3_PORT).write((divisor & 0xFF) as u8); // Low byte
        Port::<u8>::new(COM3_PORT + 1).write((divisor >> 8) as u8); // High byte
        Port::<u8>::new(COM3_PORT + 3).write(0x03); // 8 bits, no parity, 1 stop bit
    }
}

/// Send UTF-8 encoded data through UART
pub fn send_serial_data(message: &str) {
    let mut serial_port = unsafe { SerialPort::new(COM3_PORT) };
    for byte in message.as_bytes() {
        serial_port.send(*byte);
    }
}

/// Read a byte from the UART port
pub fn read_serial_data() -> u8 {
    let mut serial_port = unsafe { SerialPort::new(COM3_PORT) };
    serial_port.receive()
}
