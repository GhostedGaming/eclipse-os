//! APIC support for x86_64 architecture
//! 
//! The APIC is the modern replacement for the obsolete PIT timer,
//! with better multi-core support and additional features.

use super::cpu_types::CPUFunctions;
use super::msr::{read_msr, write_msr};

const APIC_BASE_MSR: u32 = 0x1B;
const APIC_BASE_MSR_ENABLE: u64 = 0x800;
const APIC_SPURIOUS_INTERRUPT_VECTOR: usize = 0xFF;
const APIC_SOFTWARE_ENABLE: u32 = 0x100;

fn is_apic_enabled() -> bool {
    let msr_value: u64 = read_msr(APIC_BASE_MSR);
    (msr_value & APIC_BASE_MSR_ENABLE) != 0
}

fn set_apic_base(apic: usize) {
    let eax: u32 = ((apic & 0xfffff000) | APIC_BASE_MSR_ENABLE as usize) as u32;
    let edx: u32 = 0;
    write_msr(APIC_BASE_MSR, ((edx as u64) << 32) | (eax as u64));
}

fn get_apic_base() -> usize {
    let msr_value: u64 = read_msr(APIC_BASE_MSR);
    (msr_value as usize) & 0xfffff000
}

/// Read from an APIC register at the given offset
fn read_apic_register(offset: usize) -> u32 {
    let apic_base = get_apic_base();
    let register = (apic_base + offset) as *const u32;
    unsafe { core::ptr::read_volatile(register) }
}

/// Write to an APIC register at the given offset
fn write_apic_register(offset: usize, value: u32) {
    let apic_base = get_apic_base();
    let register = (apic_base + offset) as *mut u32;
    unsafe { core::ptr::write_volatile(register, value) };
}

/// Enables the APIC if it is not enabled and if it is supported by the CPU.
pub fn enable_apic() {
    let cpu_functions = CPUFunctions::new();
    if !cpu_functions.has_apic {
        panic!("APIC not supported on this CPU");
    }

    set_apic_base(get_apic_base());

    let svr = read_apic_register(APIC_SPURIOUS_INTERRUPT_VECTOR);
    write_apic_register(APIC_SPURIOUS_INTERRUPT_VECTOR, svr | APIC_SOFTWARE_ENABLE);
}