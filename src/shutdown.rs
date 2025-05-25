use core::arch::asm;

pub fn shutdown() {
    try_apm_shutdown();
}

fn try_keyboard_reset() {
    unsafe {
        // This works on 99% of real x86 hardware
        // PS/2 controller is present even on modern systems
        asm!(
            "mov al, 0xFE",     // Reset pulse command
            "out 0x64, al",     // PS/2 controller command port
            options(nostack)
        );
    }
}

fn try_pci_reset() {
    unsafe {
        // Works on most modern systems with PCI
        asm!(
            "mov dx, 0xCF9",    // PCI reset control register
            "mov al, 0x06",     // Full system reset
            "out dx, al",
            options(nostack)
        );
    }
}

fn try_apm_shutdown() {
    // Your original APM code - works on older systems
    unsafe {
        asm!(
            "mov ax, 0x5307",
            "mov bx, 0x0001", 
            "mov cx, 0x0003",
            "int 0x15",
            options(nostack)
        );
    }
}

fn halt_system() {
    loop {
        unsafe {
            asm!("hlt", options(nostack));
        }
    }
}
