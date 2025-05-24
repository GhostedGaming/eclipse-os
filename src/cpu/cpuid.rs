use core::arch::x86_64::__cpuid;

#[derive(Debug, Clone, Copy)]
pub struct CpuInfo {
    pub vendor: CpuVendor,
    pub base_frequency_mhz: Option<u32>,
    pub max_frequency_mhz: Option<u32>,
    pub bus_frequency_mhz: Option<u32>,
    pub tsc_frequency_hz: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CpuVendor {
    Intel,
    AMD,
    Unknown,
}

impl CpuInfo {
    pub fn new() -> Self {
        let vendor = get_cpu_vendor();
        let (base_freq, max_freq, bus_freq) = get_processor_frequencies();
        let tsc_freq = get_tsc_frequency();

        CpuInfo {
            vendor,
            base_frequency_mhz: base_freq,
            max_frequency_mhz: max_freq,
            bus_frequency_mhz: bus_freq,
            tsc_frequency_hz: tsc_freq,
        }
    }

    /// Get the best available frequency for timing calculations
    pub fn get_timing_frequency_hz(&self) -> Option<u64> {
        // Prefer TSC frequency if available
        if let Some(tsc_freq) = self.tsc_frequency_hz {
            return Some(tsc_freq);
        }

        // Fall back to base frequency
        if let Some(base_freq) = self.base_frequency_mhz {
            return Some(base_freq as u64 * 1_000_000);
        }

        // Fall back to max frequency
        if let Some(max_freq) = self.max_frequency_mhz {
            return Some(max_freq as u64 * 1_000_000);
        }

        None
    }
}

fn get_cpu_vendor() -> CpuVendor {
    let cpuid_result = unsafe { __cpuid(0) };

    let vendor_bytes = [
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

    match &vendor_bytes {
        b"GenuineIntel" => CpuVendor::Intel,
        b"AuthenticAMD" => CpuVendor::AMD,
        _ => CpuVendor::Unknown,
    }
}

fn get_processor_frequencies() -> (Option<u32>, Option<u32>, Option<u32>) {
    // Check if CPUID leaf 0x16 is supported
    let max_leaf = unsafe { __cpuid(0) }.eax;
    if max_leaf < 0x16 {
        return (None, None, None);
    }

    let freq_info = unsafe { __cpuid(0x16) };
    
    let base_freq = if freq_info.eax != 0 { Some(freq_info.eax) } else { None };
    let max_freq = if freq_info.ebx != 0 { Some(freq_info.ebx) } else { None };
    let bus_freq = if freq_info.ecx != 0 { Some(freq_info.ecx) } else { None };

    (base_freq, max_freq, bus_freq)
}

fn get_tsc_frequency() -> Option<u64> {
    // Check if CPUID leaf 0x15 is supported
    let max_leaf = unsafe { __cpuid(0) }.eax;
    if max_leaf < 0x15 {
        return None;
    }

    let tsc_info = unsafe { __cpuid(0x15) };
    
    // EAX: denominator, EBX: numerator, ECX: crystal clock frequency
    let denominator = tsc_info.eax;
    let numerator = tsc_info.ebx;
    let crystal_freq = tsc_info.ecx;

    if denominator == 0 || numerator == 0 {
        return None;
    }

    if crystal_freq != 0 {
        // Calculate TSC frequency: (crystal_freq * numerator) / denominator
        Some((crystal_freq as u64 * numerator as u64) / denominator as u64)
    } else {
        // Some processors don't report crystal frequency, try to estimate
        estimate_tsc_frequency_fallback()
    }
}

fn estimate_tsc_frequency_fallback() -> Option<u64> {
    // This is a rough estimation method using PIT timer
    // You might want to implement a more sophisticated method
    None
}

pub fn print_cpu_info() {
    let cpu_info = CpuInfo::new();
    
    let vendor_str = match cpu_info.vendor {
        CpuVendor::Intel => "Intel",
        CpuVendor::AMD => "AMD",
        CpuVendor::Unknown => "Unknown",
    };
    
    crate::println!("CPU Vendor: {}", vendor_str);
    
    if let Some(base_freq) = cpu_info.base_frequency_mhz {
        crate::println!("Base Frequency: {} MHz", base_freq);
    }
    
    if let Some(max_freq) = cpu_info.max_frequency_mhz {
        crate::println!("Max Frequency: {} MHz", max_freq);
    }
    
    if let Some(tsc_freq) = cpu_info.tsc_frequency_hz {
        crate::println!("TSC Frequency: {} Hz ({} MHz)", tsc_freq, tsc_freq / 1_000_000);
    }
}

// Global CPU info - initialized once
static mut CPU_INFO: Option<CpuInfo> = None;
static CPU_INFO_INIT: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

pub fn init_cpu_info() {
    if CPU_INFO_INIT.compare_exchange(
        false, 
        true, 
        core::sync::atomic::Ordering::SeqCst, 
        core::sync::atomic::Ordering::SeqCst
    ).is_ok() {
        unsafe {
            CPU_INFO = Some(CpuInfo::new());
        }
    }
}

pub fn get_cpu_info() -> Option<CpuInfo> {
    if CPU_INFO_INIT.load(core::sync::atomic::Ordering::SeqCst) {
        unsafe { CPU_INFO }
    } else {
        None
    }
}
