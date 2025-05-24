use core::arch::x86_64::__cpuid;
use alloc::string::String;

pub fn cpuid_intruction() -> String {
    let cpuid_result = unsafe{ __cpuid(0) };

    let vendor = [
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
    ];

    String::from_utf8(vendor.to_vec()).unwrap_or_default()
}