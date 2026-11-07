//! How much memory a job's threads may take, so that packwing never runs out
//! of it because of them: what runs out cannot be caught (the system ends the
//! program, or Rust does), so it is kept from happening.
//!
//! When a job starts, it asks for the most threads it may take ([`crate::threads`])
//! whose memory fits the budget: 70% of what the system says is available now
//! ([`available`]), leaving at least [`LEFT`] of it to everything else, and
//! never less than half of it ([`share_of`]), less what the jobs running hold. One thread is always
//! given, as it takes what the job takes without threads. What a job holds is
//! given back when it ends ([`Held`]).
//!
//! What a job holds and has already taken is counted twice, once as held and
//! once as no longer available: jobs at once get fewer threads than they
//! could, never too many. The threads change only how soon a job is done:
//! what is made is the same.
//!
//! What is available is measured by hivepack's `memory`: the least of what
//! the system has free and what its limits on packwing leave, a control
//! group's and `ulimit`'s on Linux, a job object's on Windows. The policy is
//! packwing's: [`share_of`] it, never less than [`LEAST`], and `PACKWING_MEMORY`.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use hivepack::memory::Budget;
pub use hivepack::memory::Held;

/// What [`available`] says, if set: for tests.
static AVAILABLE: AtomicU64 = AtomicU64::new(0);

/// The least a job is let take, whatever is available: what one thread of
/// most coders needs, so that a busy system is not told nothing at all.
const LEAST: u64 = 64 << 20;

/// What the jobs' threads leave to everything else of what is available, at
/// least, once there is enough of it that half would leave more: 2 GiB, for
/// the programs that grow while a long job runs.
const LEFT: u64 = 2 << 30;

/// What the jobs' threads may take of `available` bytes: 70% of them, but no
/// more than leaves [`LEFT`], and never less than half, as it always was.
/// Half up to 4 GiB available, then more, 70% from about 6.7 GiB on.
fn share_of(available: u64) -> u64 {
    let most = available / 10 * 7;
    let leaving = available.saturating_sub(LEFT);
    (available / 2).max(most.min(leaving))
}

/// The jobs' budget, shared between them: its bytes are measured again each
/// time a job asks ([`refreshed`]). Each thread's overhead under `ulimit -v`
/// (its stack, and the allocator's arena) is the system's, measured once.
fn shared() -> &'static Budget {
    static BUDGET: OnceLock<Budget> = OnceLock::new();
    BUDGET.get_or_init(|| {
        let overhead = hivepack::memory::available().map_or(0, |a| a.thread_overhead);
        Budget::new(0).with_thread_overhead(overhead)
    })
}

/// The budget, its bytes [`share_of`] what is available now, and never so few
/// that less than [`LEAST`] is left past what the jobs running hold.
fn refreshed() -> &'static Budget {
    refresh(shared())
}

/// `budget`, its bytes set as [`refreshed`] sets the shared one's.
fn refresh(budget: &Budget) -> &Budget {
    budget.set_bytes(share_of(available()).max(budget.held().saturating_add(LEAST)));
    budget
}

/// The memory packwing may still take now, in bytes: RAM that can be taken
/// without swapping (Linux's `MemAvailable`, Windows' available physical
/// memory, macOS's free and reclaimable pages), and no more than the limits
/// put on packwing leave (a container's, Flatpak's or systemd's control
/// group, `ulimit`, a Windows job object). `PACKWING_MEMORY`, in bytes,
/// stands for it if set.
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
    hivepack::memory::available()
        .map(|a| a.bytes)
        .filter(|bytes| *bytes > 0)
        .unwrap_or(LEAST * 2)
}

/// Makes [`available`] say `bytes` from now on: 0 measures again.
pub fn set_available(bytes: u64) {
    AVAILABLE.store(bytes, Ordering::Relaxed);
}

/// What a job starting now may take for its threads: [`share_of`] what is
/// available, less what the jobs running hold, and never less than
/// [`LEAST`].
pub fn budget() -> u64 {
    refreshed().left()
}

/// The budget itself, its bytes measured now, for a job that knows what its
/// threads need only once it has started, and holds them from it then:
/// extracting RAR, each thread with a window as large as the archive's
/// dictionary, once its headers are read. What it holds is given back when
/// the job ends, as with [`hold_threads`].
pub fn shared_budget() -> Budget {
    refreshed().clone()
}

/// The most threads, up to `wanted`, whose memory (`need` of their number)
/// fits the [`budget`], and that memory, held until the job ends. One thread
/// is always given, holding what it needs, even past the budget.
pub fn hold_threads(wanted: u32, need: impl Fn(u32) -> u64) -> (u32, Held) {
    refreshed().hold_threads(wanted, need)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: u64 = 1 << 20;
    const GIB: u64 = 1 << 30;

    #[test]
    fn half_of_little_memory_and_seventy_percent_of_more_leaving_two_gib() {
        // Half while it leaves more than 2 GiB would.
        assert_eq!(share_of(3 * GIB), 3 * GIB / 2);
        assert_eq!(share_of(4 * GIB), 2 * GIB);
        // Between, all but 2 GiB.
        assert_eq!(share_of(6 * GIB), 4 * GIB);
        // 70% once that leaves 2 GiB or more.
        assert_eq!(share_of(8 * GIB), 8 * GIB / 10 * 7);
        assert_eq!(share_of(50 * GIB), 50 * GIB / 10 * 7);
        assert_eq!(share_of(0), 0);
        // Never less than half nor more than 70%, and never less for more.
        let mut before = 0;
        for mib in (0..=64 * 1024).step_by(97) {
            let bytes = mib * MIB;
            let share = share_of(bytes);
            assert!(
                share >= bytes / 2 && share <= bytes / 10 * 7,
                "{mib} MiB: {share}"
            );
            assert!(share >= before, "{mib} MiB: {share} after {before}");
            before = share;
        }
    }

    // A budget of its own: the tests that make archives hold from the
    // shared one at the same time.
    #[test]
    fn what_is_held_is_left_out_until_given_back() {
        let own = Budget::new(0);
        set_available(4000 * MIB);
        let need = |n: u32| u64::from(n) * 500 * MIB;
        let (first, a) = refresh(&own).hold_threads(16, need);
        assert!(first >= 1);
        assert!(a.bytes() >= need(first));
        assert_eq!(own.held(), a.bytes());
        let (second, b) = refresh(&own).hold_threads(16, need);
        assert!(second <= first, "{second} after {first}");
        drop(a);
        drop(b);
        assert_eq!(own.held(), 0);
        set_available(10 * MIB);
        assert!(refresh(&own).left() >= LEAST);
        set_available(0);
    }

    #[test]
    fn the_system_says_something() {
        assert!(measured() > 0);
    }
}
