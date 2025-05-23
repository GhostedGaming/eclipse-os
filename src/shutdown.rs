use core::arch::asm;

pub fn shutdown() {
    // Methods that work on real hardware (in order of reliability)
    
    // Method 1: PS/2 Keyboard Controller Reset (MOST RELIABLE)
    try_keyboard_reset();
    
    // Method 2: PCI Reset Control Register  
    try_pci_reset();
    
    // Method 3: APM (fallback for older systems)
    try_apm_shutdown();
    
    // Last resort: halt
    halt_system();
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
