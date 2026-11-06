//! How much memory a job's threads may take, so that packwing never runs out
//! of it because of them: what runs out cannot be caught (the system ends the
//! program, or Rust does), so it is kept from happening.
//!
//! When a job starts, it asks for the most threads it may take ([`crate::threads`])
//! whose memory fits the budget: half of what the system says is available
//! now ([`available`]), less what the jobs running hold. One thread is always
//! given, as it takes what the job takes without threads. What a job holds is
//! given back when it ends ([`Held`]).
//!
//! What a job holds and has already taken is counted twice, once as held and
//! once as no longer available: jobs at once get fewer threads than they
//! could, never too many. The threads change only how soon a job is done:
//! what is made is the same.
//!
//! What is available is the least of what the system has free and what its
//! limits on packwing leave: a control group's on Linux, `ulimit`'s too, and
//! a job object's on Windows (`memory_limits.rs`).

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use sysinfo::{MemoryRefreshKind, RefreshKind, System};

/// What the jobs running hold between them, in bytes.
static HELD: Mutex<u64> = Mutex::new(0);

/// What [`available`] says, if set: for tests, and for `PACKWING_MEMORY`.
static AVAILABLE: AtomicU64 = AtomicU64::new(0);

/// The address space a thread takes besides what it allocates: its stack
/// (2 MiB), and the allocator's arena it may make (64 MiB on glibc).
const THREAD_ADDRESS_SPACE: u64 = 66 << 20;

#[cfg(target_os = "linux")]
fn address_space_limited() -> bool {
    crate::memory_limits::address_space_limited()
}

#[cfg(not(target_os = "linux"))]
fn address_space_limited() -> bool {
    false
}

/// The least a job is let take, whatever is available: what one thread of
/// most coders needs, so that a busy system is not told nothing at all.
const LEAST: u64 = 64 << 20;

/// Memory a job holds for its threads, given back when it is dropped.
#[derive(Debug)]
pub struct Held(u64);

impl Held {
    /// How many bytes it holds.
    pub fn bytes(&self) -> u64 {
        self.0
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
        *held = held.saturating_sub(self.0);
    }
}

/// The memory the system has available now, in bytes: RAM that can be taken
/// without swapping (Linux's `MemAvailable`, Windows' available physical
/// memory, macOS's free and reclaimable pages), and on Linux no more than
/// the control group packwing runs in has left (a container, Flatpak,
/// systemd's limits). `PACKWING_MEMORY`, in bytes, stands for it if set.
pub fn available() -> u64 {
    match AVAILABLE.load(Ordering::Relaxed) {
        0 => measured(),
        set => set,
    }
}

fn measured() -> u64 {
    if let Some(set) = std::env::var("PACKWING_MEMORY")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v > 0)
    {
        return set;
    }
    // A system that says nothing is taken as having little.
    let Ok(system) = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::nothing().with_ram()),
    ) else {
        return LEAST * 2;
    };
    let mut available = system.available_memory();
    if let Some(group) = system.cgroup_limits() {
        available = available.min(group.free_memory);
    }
    if let Some(left) = control_group_left() {
        available = available.min(left);
    }
    #[cfg(windows)]
    if let Some(left) = crate::memory_limits::job_left() {
        available = available.min(left);
    }
    #[cfg(target_os = "linux")]
    if let Some(left) = crate::memory_limits::ulimit_left() {
        available = available.min(left);
    }
    if available == 0 { LEAST * 2 } else { available }
}

/// What the control groups packwing runs in let it take still, on Linux:
/// the least, between each group's limit (`memory.max`) and what it holds
/// (`memory.current`), from its own group up to the root. sysinfo reads only
/// the root's, which is the group's own in a container but not under
/// systemd's slices and scopes. None where no group has a limit.
#[cfg(target_os = "linux")]
fn control_group_left() -> Option<u64> {
    let mine = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    // cgroup v2's line: `0::/user.slice/…/app.scope`.
    let path = mine.lines().find_map(|line| line.strip_prefix("0::"))?;
    let read = |dir: &std::path::Path, name: &str| -> Option<u64> {
        std::fs::read_to_string(dir.join(name))
            .ok()?
            .trim()
            .parse()
            .ok()
    };
    let mut dir = std::path::Path::new("/sys/fs/cgroup").join(path.trim_start_matches('/'));
    let mut least: Option<u64> = None;
    loop {
        if let Some(max) = read(&dir, "memory.max") {
            let left = max.saturating_sub(read(&dir, "memory.current").unwrap_or(0));
            least = Some(least.map_or(left, |l| l.min(left)));
        }
        if dir == std::path::Path::new("/sys/fs/cgroup") || !dir.pop() {
            return least;
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn control_group_left() -> Option<u64> {
    None
}

/// Makes [`available`] say `bytes` from now on: 0 measures again.
pub fn set_available(bytes: u64) {
    AVAILABLE.store(bytes, Ordering::Relaxed);
}

/// What a job starting now may take for its threads: half of what is
/// available, less what the jobs running hold, and never less than
/// [`LEAST`].
pub fn budget() -> u64 {
    let held = *HELD.lock().unwrap_or_else(|e| e.into_inner());
    (available() / 2).saturating_sub(held).max(LEAST)
}

/// The most threads, up to `wanted`, whose memory (`need` of their number)
/// fits the [`budget`], and that memory, held until the job ends. One thread
/// is always given, holding what it needs, even past the budget.
pub fn hold_threads(wanted: u32, need: impl Fn(u32) -> u64) -> (u32, Held) {
    // Under `ulimit -v`, each thread takes address space besides what it
    // allocates: its stack, and the allocator's arena it may take.
    let more = if address_space_limited() {
        THREAD_ADDRESS_SPACE
    } else {
        0
    };
    let need = |n: u32| need(n).saturating_add(u64::from(n) * more);
    let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
    let budget = (available() / 2).saturating_sub(*held).max(LEAST);
    let threads = threads_within(wanted, budget, &need);
    let bytes = need(threads);
    *held = held.saturating_add(bytes);
    (threads, Held(bytes))
}

/// The most threads, up to `wanted`, whose `need` is within `budget`; one if
/// none is.
fn threads_within(wanted: u32, budget: u64, need: &dyn Fn(u32) -> u64) -> u32 {
    // What threads need grows with their number: the most that fit are
    // found by halving.
    let (mut fits, mut over) = (1, wanted.max(1) + 1);
    while over - fits > 1 {
        let mid = fits + (over - fits) / 2;
        if need(mid) <= budget {
            fits = mid;
        } else {
            over = mid;
        }
    }
    fits
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: u64 = 1 << 20;

    #[test]
    fn as_many_threads_as_fit_and_never_none() {
        let need = |n: u32| 100 * MIB + u64::from(n) * 200 * MIB;
        assert_eq!(threads_within(32, 1100 * MIB, &need), 5);
        assert_eq!(threads_within(4, 1100 * MIB, &need), 4);
        assert_eq!(threads_within(32, 10 * MIB, &need), 1);
        assert_eq!(threads_within(0, 10_000 * MIB, &need), 1);
        assert_eq!(threads_within(1, u64::MAX, &need), 1);
    }

    #[test]
    fn what_is_held_is_left_out_until_given_back() {
        set_available(4000 * MIB);
        let base = *HELD.lock().unwrap_or_else(|e| e.into_inner());
        let need = |n: u32| u64::from(n) * 500 * MIB;
        let (first, a) = hold_threads(16, need);
        assert!(first >= 1);
        assert_eq!(a.bytes(), need(first));
        assert_eq!(
            *HELD.lock().unwrap_or_else(|e| e.into_inner()),
            base + a.bytes()
        );
        let (second, b) = hold_threads(16, need);
        assert!(second <= first, "{second} after {first}");
        drop(a);
        drop(b);
        assert_eq!(*HELD.lock().unwrap_or_else(|e| e.into_inner()), base);
        set_available(0);
    }

    #[test]
    fn the_system_says_something() {
        assert!(measured() > 0);
    }
}
