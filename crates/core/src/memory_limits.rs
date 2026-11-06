//! The limits on memory the system puts on packwing beside what is free,
//! which no crate reads: on Windows, a job object's (containers, sandboxes,
//! some launchers and CI runners put programs in one); on Linux, `ulimit`'s
//! (`RLIMIT_AS`, `RLIMIT_DATA`). Past either, memory is refused, and Rust
//! ends the program. macOS keeps no such limit on an app's memory: short of
//! it, it compresses and swaps.
//!
//! The calls to the system are the only `unsafe` in the crate.

#![allow(unsafe_code)]

/// What packwing may still take under its job object's limits, in bytes: the
/// least of what the job's limit for each process leaves this one, and what
/// its limit for all its processes leaves them, counted from the most they
/// have taken, which is no less than what they hold. None outside a job, or
/// where it has no limit on memory.
#[cfg(windows)]
pub(crate) fn job_left() -> Option<u64> {
    use windows_sys::Win32::System::JobObjects::{
        IsProcessInJob, JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        QueryInformationJobObject,
    };
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    let mut in_job = 0;
    // SAFETY: the current process's pseudo-handle, no job (any), and a BOOL
    // to write.
    let asked = unsafe { IsProcessInJob(GetCurrentProcess(), std::ptr::null_mut(), &mut in_job) };
    if asked == 0 || in_job == 0 {
        return None;
    }
    // SAFETY: a plain C struct, whose zeros are a valid value.
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: no handle is the job of this process; the struct and its size
    // are the class's.
    let read = unsafe {
        QueryInformationJobObject(
            std::ptr::null_mut(),
            JobObjectExtendedLimitInformation,
            (&raw mut info).cast(),
            u32::try_from(size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>()).ok()?,
            std::ptr::null_mut(),
        )
    };
    if read == 0 {
        return None;
    }
    let flags = info.BasicLimitInformation.LimitFlags;
    let mut least: Option<u64> = None;
    let mut keep = |left: u64| least = Some(least.map_or(left, |l| l.min(left)));
    if flags & JOB_OBJECT_LIMIT_PROCESS_MEMORY != 0 {
        // SAFETY: as above; the counters and their size.
        let mut counters: PROCESS_MEMORY_COUNTERS_EX = unsafe { std::mem::zeroed() };
        let size = u32::try_from(size_of::<PROCESS_MEMORY_COUNTERS_EX>()).ok()?;
        counters.cb = size;
        // SAFETY: the current process, and counters of the size given.
        let got = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                (&raw mut counters).cast::<PROCESS_MEMORY_COUNTERS>(),
                size,
            )
        };
        let used = if got == 0 { 0 } else { counters.PrivateUsage };
        keep((info.ProcessMemoryLimit.saturating_sub(used)) as u64);
    }
    if flags & JOB_OBJECT_LIMIT_JOB_MEMORY != 0 {
        keep((info.JobMemoryLimit.saturating_sub(info.PeakJobMemoryUsed)) as u64);
    }
    least
}

/// What packwing may still take under `ulimit`'s limits on its address
/// space and its data, in bytes: each less what it holds of it now (its
/// `VmSize` and `VmData`). None where neither is set.
#[cfg(target_os = "linux")]
pub(crate) fn ulimit_left() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let held = |field: &str| -> u64 {
        status
            .lines()
            .find_map(|line| line.strip_prefix(field))
            .and_then(|rest| {
                rest.trim()
                    .trim_end_matches("kB")
                    .trim()
                    .parse::<u64>()
                    .ok()
            })
            .map_or(0, |kib| kib * 1024)
    };
    let mut least: Option<u64> = None;
    for (resource, field) in [(libc::RLIMIT_AS, "VmSize:"), (libc::RLIMIT_DATA, "VmData:")] {
        if let Some(limit) = limit_of(resource) {
            let left = limit.saturating_sub(held(field));
            least = Some(least.map_or(left, |l| l.min(left)));
        }
    }
    least
}

/// Whether `ulimit` limits the address space, which each thread takes some of
/// besides what it allocates: its stack, and its share of the allocator's
/// arenas.
#[cfg(target_os = "linux")]
pub(crate) fn address_space_limited() -> bool {
    limit_of(libc::RLIMIT_AS).is_some()
}

/// What `getrlimit` takes for a resource, as the C library has it.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
type Resource = libc::__rlimit_resource_t;
#[cfg(all(target_os = "linux", not(target_env = "gnu")))]
type Resource = libc::c_int;

#[cfg(target_os = "linux")]
fn limit_of(resource: Resource) -> Option<u64> {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: a resource the system knows, and a struct to write.
    if unsafe { libc::getrlimit(resource, &mut limit) } != 0
        || limit.rlim_cur == libc::RLIM_INFINITY
    {
        return None;
    }
    Some(limit.rlim_cur)
}
