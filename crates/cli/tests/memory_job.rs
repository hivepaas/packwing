//! On Windows, pkw in a job object that lets each of its processes take
//! 600 MB: a level whose one thread would take more is lowered, and said,
//! where memory would have been refused part way. Alone in its file, as the
//! test puts its own process in the job, which pkw is then started in.

#![cfg(windows)]
#![allow(unsafe_code)]

use std::path::PathBuf;
use std::process::{Command, Stdio};

use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// Puts this process, and what it starts from now on, in a new job whose
/// processes may each take `bytes`.
fn into_a_job_of(bytes: usize) {
    // SAFETY: no attributes, no name.
    let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    assert!(!job.is_null(), "a job object");
    // SAFETY: a plain C struct, whose zeros are a valid value.
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY;
    info.ProcessMemoryLimit = bytes;
    let size = u32::try_from(size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>()).expect("its size");
    // SAFETY: the job just made, and the class's struct and size.
    let set = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            (&raw const info).cast(),
            size,
        )
    };
    assert_ne!(set, 0, "the job's limit");
    // SAFETY: the job, and this process's pseudo-handle.
    let assigned = unsafe { AssignProcessToJobObject(job, GetCurrentProcess()) };
    assert_ne!(assigned, 0, "this process in the job");
}

#[test]
fn in_a_job_of_little_memory_a_level_too_large_is_lowered_and_said() {
    let dir = std::env::temp_dir().join(format!("pkw-job-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).expect("a folder");
    std::fs::write(dir.join("src/notes.txt"), "xin chào\n".repeat(10_000)).expect("a file");
    into_a_job_of(600 << 20);
    let pkw = PathBuf::from(env!("CARGO_BIN_EXE_pkw"));
    for name in ["a.7z", "a.tar.xz"] {
        let made = Command::new(&pkw)
            .args(["create", "-l", "highest", "-T", "0", name, "src"])
            .current_dir(&dir)
            .env_remove("PACKWING_MEMORY")
            .env("APPDATA", dir.join("config"))
            .stdin(Stdio::null())
            .output()
            .expect("pkw runs");
        let said = String::from_utf8_lossy(&made.stderr).into_owned();
        assert!(made.status.success(), "{name}: {said}");
        assert!(said.contains("it is made at"), "{name}: {said}");
        let tested = Command::new(&pkw)
            .args(["test", "-q", name])
            .current_dir(&dir)
            .env("APPDATA", dir.join("config"))
            .stdin(Stdio::null())
            .status()
            .expect("pkw runs");
        assert!(tested.success(), "{name} tests sound");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
