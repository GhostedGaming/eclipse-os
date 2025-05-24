use core::arch::x86_64::__cpuid;

pub fn cpuid_instruction() -> [u8; 12] {
    let cpuid_result = unsafe { __cpuid(0) };

    [
        cpuid_result.ebx as u8,
        (cpuid_result.ebx >> 8) as u8,
        (cpuid_result.ebx >> 16) as u8,
        (cpuid_result.ebx >> 24) as u8,
        cpuid_result.edx as u8,
        (cpuid_result.edx >> 8) as u8,
        (cpuid_result.edx >> 16) as u8,
        (cpuid_result.edx >> 24) as u8,
        cpuid_result.ecx as u8,
        (cpuid_result.ecx >> 8) as u8,
        (cpuid_result.ecx >> 16) as u8,
        (cpuid_result.ecx >> 24) as u8,
    ]
}

pub fn print_cpu_vendor() {
    let vendor_bytes = cpuid_instruction();
    
    // Convert to string slice without allocation
    if let Ok(vendor_str) = core::str::from_utf8(&vendor_bytes) {
        crate::println!("CPU Vendor: {}", vendor_str);
    } else {
        crate::println!("CPU Vendor: <invalid UTF-8>");
    }
}

// Alternative version that returns a fixed-size string
pub fn get_cpu_vendor_str() -> &'static str {
    let vendor_bytes = cpuid_instruction();
    
    // Common CPU vendors
    match &vendor_bytes {
        b"GenuineIntel" => "Intel",
        b"AuthenticAMD" => "AMD", 
        b"CyrixInstead" => "Cyrix",
        b"CentaurHauls" => "Centaur",
        _ => "Unknown",
    }
}
