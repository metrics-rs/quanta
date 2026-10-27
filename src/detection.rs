#[allow(dead_code)]
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
pub fn has_counter_support() -> bool {
    let cpuid = raw_cpuid::CpuId::new();
    let has_invariant_tsc = cpuid
        .get_advanced_power_mgmt_info()
        .map_or(false, |apm| apm.has_invariant_tsc());
    let has_rdtscp = cpuid
        .get_extended_processor_and_feature_identifiers()
        .map_or(false, |epf| epf.has_rdtscp());

    has_invariant_tsc && has_rdtscp && kernel_reports_stable_tsc()
}

// Linux never registers the `tsc` clocksource if it finds the TSC unsynchronized across CPUs at
// boot, and stops listing it as available once it later marks the TSC unstable, as long as the
// kernel runs with nohz or high-resolution timers. Like `minstant`, we trust that verdict rather
// than detecting it ourselves, but match the exact clocksource name:
// https://github.com/tikv/minstant/blob/master/src/tsc_now.rs
//
// The kernel skips these checks when the TSC is marked reliable (`tsc=reliable`), so an
// unsynchronized TSC can still be listed there.
//
// This is checked once per process.
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_feature = "sse2"))]
fn kernel_reports_stable_tsc() -> bool {
    static STABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

    *STABLE.get_or_init(|| {
        kernel_lists_tsc(std::path::Path::new(
            "/sys/devices/system/clocksource/clocksource0/available_clocksource",
        ))
    })
}

#[cfg(all(
    not(target_os = "linux"),
    target_arch = "x86_64",
    target_feature = "sse2"
))]
fn kernel_reports_stable_tsc() -> bool {
    true
}

#[cfg(all(target_arch = "aarch64", not(target_os = "ios")))]
pub fn has_counter_support() -> bool {
    // AArch64 implies ARMv8 or above, where the system counter is always present.
    //
    // However, the instruction necessary to read the raw counter (`mrs`) is not always available,
    // specifically in the case of iOS.
    true
}

#[allow(dead_code)]
#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "sse2"),
    all(target_arch = "aarch64", not(target_os = "ios"))
)))]
pub fn has_counter_support() -> bool {
    false
}

// If the list can't be read, we can't confirm the TSC is stable, so we fall back to the OS clock.
#[allow(dead_code)]
fn kernel_lists_tsc(path: &std::path::Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|available| lists_tsc(&available))
}

// Match whole names only, since `tsc-early` and `hyperv_clocksource_tsc_page` are separate
// clocksources.
#[allow(dead_code)]
fn lists_tsc(available: &str) -> bool {
    available.split_whitespace().any(|name| name == "tsc")
}

#[cfg(test)]
mod tests {
    use super::lists_tsc;

    #[test]
    fn lists_tsc_accepts_tsc_among_others() {
        assert!(lists_tsc("tsc hpet acpi_pm \n"));
        assert!(lists_tsc("kvm-clock tsc acpi_pm \n"));
    }

    #[test]
    fn lists_tsc_rejects_missing_tsc() {
        assert!(!lists_tsc("hpet acpi_pm \n"));
        assert!(!lists_tsc("\n"));
        assert!(!lists_tsc(""));
    }

    #[test]
    fn lists_tsc_rejects_other_tsc_based_clocksources() {
        assert!(!lists_tsc("tsc-early hpet acpi_pm \n"));
        assert!(!lists_tsc("hyperv_clocksource_tsc_page acpi_pm \n"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn kernel_lists_tsc_reads_file() {
        use super::kernel_lists_tsc;

        let dir = std::env::temp_dir().join(format!("quanta-detection-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let with_tsc = dir.join("with_tsc");
        let without_tsc = dir.join("without_tsc");
        std::fs::write(&with_tsc, "tsc hpet acpi_pm \n").unwrap();
        std::fs::write(&without_tsc, "hpet acpi_pm \n").unwrap();

        let (listed, not_listed) = (kernel_lists_tsc(&with_tsc), kernel_lists_tsc(&without_tsc));
        std::fs::remove_dir_all(&dir).unwrap();

        assert!(listed);
        assert!(!not_listed);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn kernel_lists_tsc_rejects_unreadable_file() {
        use super::kernel_lists_tsc;

        let missing = std::env::temp_dir().join("quanta-detection-missing/available_clocksource");
        assert!(!kernel_lists_tsc(&missing));
    }
}
