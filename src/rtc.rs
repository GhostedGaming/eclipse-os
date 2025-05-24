use x86_64::instructions::port::Port;
use core::fmt;

#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    pub fn format_12h(&self) -> (u8, u8, bool) {
        let (hour_12, is_pm) = if self.hour == 0 {
            (12, false) // 12 AM
        } else if self.hour < 12 {
            (self.hour, false) // AM
        } else if self.hour == 12 {
            (12, true) // 12 PM
        } else {
            (self.hour - 12, true) // PM
        };
        (hour_12, self.minute, is_pm)
    }

    pub fn format_24h(&self) -> (u8, u8) {
        (self.hour, self.minute)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (hour, minute, is_pm) = self.format_12h();
        let ampm = if is_pm { "PM" } else { "AM" };
        write!(f, "{:02}:{:02} {}", hour, minute, ampm)
    }
}

pub struct RTC {
    cmos_address: Port<u8>,
    cmos_data: Port<u8>,
}

impl RTC {
    pub fn new() -> Self {
        RTC {
            cmos_address: Port::new(0x70),
            cmos_data: Port::new(0x71),
        }
    }

    fn read_register(&mut self, register: u8) -> u8 {
        unsafe {
            self.cmos_address.write(register);
            self.cmos_data.read()
        }
    }

    fn bcd_to_binary(&self, bcd: u8) -> u8 {
        (bcd & 0x0F) + ((bcd >> 4) * 10)
    }

    pub fn read_datetime(&mut self) -> DateTime {
        // Wait for update to complete
        while self.read_register(0x0A) & 0x80 != 0 {}

        let second = self.read_register(0x00);
        let minute = self.read_register(0x02);
        let hour = self.read_register(0x04);
        let day = self.read_register(0x07);
        let month = self.read_register(0x08);
        let year = self.read_register(0x09);

        // Read status register B to check format
        let status_b = self.read_register(0x0B);
        let is_24h = (status_b & 0x02) != 0;
        let is_binary = (status_b & 0x04) != 0;

        // Convert from BCD if necessary
        let (second, minute, hour, day, month, year) = if is_binary {
            (second, minute, hour, day, month, year)
        } else {
            (
                self.bcd_to_binary(second),
                self.bcd_to_binary(minute),
                self.bcd_to_binary(hour),
                self.bcd_to_binary(day),
                self.bcd_to_binary(month),
                self.bcd_to_binary(year),
            )
        };

        // Handle 12-hour format if needed
        let hour = if !is_24h && hour & 0x80 != 0 {
            // PM bit is set in 12-hour mode
            (hour & 0x7F) + 12
        } else {
            hour
        };

        DateTime {
            year: 2000 + year as u16, // Assuming 21st century
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

// Global RTC instance
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref RTC_INSTANCE: Mutex<RTC> = Mutex::new(RTC::new());
}

pub fn get_current_time() -> DateTime {
    RTC_INSTANCE.lock().read_datetime()
}
