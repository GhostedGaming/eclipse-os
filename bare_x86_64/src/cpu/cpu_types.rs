use core::arch::x86_64::{__cpuid, CpuidResult,};

pub struct CPUFunctions {
    pub has_apic: bool,
}

impl CPUFunctions {
    pub fn new() -> Self {
        Self {
            has_apic: Self::check_apic(),
        }
    }

    fn check_apic() -> bool {
        let result: CpuidResult = unsafe { __cpuid(1) };
        (result.edx & (1 << 9)) != 0
    }
}