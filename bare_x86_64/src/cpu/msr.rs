//! This file contains code related to Model-Specific Registers (MSRs) on x86_64 CPUs.
//! MSRs are used to control various CPU features and settings.
//! This module provides functions to read from and write to MSRs.

use core::arch::asm;

/// Reads the value of the specified MSR.
pub fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Writes the specified value to the given MSR.
pub fn write_msr(msr: u32, value: u64) {
    let low: u32 = value as u32;
    let high: u32 = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
        );
    }
}