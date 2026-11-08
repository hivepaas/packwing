# Audit log: design and phases

What the audit log does today is in [../audit.md](../audit.md). This is why it is built as it is, and what comes next.

## Decisions

- **The policy decides, not the license.** `packwing-license` says it: "No key ever turns a policy or a security feature on or off: a bill unpaid must not be a security incident." Where an organization sets `AuditLog`, the log is kept on every plan, and past a license's end. The license matters only to the user's own switch: Pro and Enterprise have it, Free does not. A Pro user whose license ends stops writing new records; what is there stays, and goes as its days pass.
- **On the computer, in files.** One JSON line a record, a file a day, in the system's folder for a program's state. Every log collector reads it, a person can read it with `jq`, and taking old days away is taking files away. packwing sends nothing anywhere, as it promises; an organization's collector takes the records off the computer.
- **A job is two records.** A start, with what it is of, and an end, with how it went. A job that crashed leaves its start.
- **A chain, not a lock.** Each record holds the hash of the line before it. It shows a record changed or taken out between others, which is what a hand editing a file does; it cannot stop someone who writes the whole chain again. The person at the computer controls it, and packwing is free software: no design changes that. What makes records last is taking them off the computer.
- **What the user chose is named; what packwing came on, only when asked.** File names say what people work on (salaries, clients). What the user picked, on the computer or in an archive, is what the job is of, and a record without it says nothing; a file they opened is named by its copy anyway. What packwing comes on as it works, as files damaged or left out, can be thousands of names nobody looked at: records count them, and `AuditLogDetail=entries` names them.
- **A preview once a run.** Showing a file is seeing what is in it, and is recorded as opening one is, or an archive could be read file by file and leave nothing. Going through an archive shows many: each is recorded the first time it shows in a run, with `via: preview`.
- **A mistake in a policy never loosens.** As with the other policies: a value that cannot be understood keeps the log on, keeps every record, and names the files.

## Phase 1 (this change)

- `packwing-audit`: the records, the chain, the days' files, retention by days and by size, `verify`, `read`.
- Policies: `AuditLog`, `AuditLogPath` (`{user}`, `{host}`), `AuditLogRetentionDays`, `AuditLogMaxSizeMB`, `AuditLogDetail`.
- The core's jobs record their start and end, once each, a job inside another the outer one's.
- Settings › Security: the switch (Pro and Enterprise), how long, where, Open folder; locked under a policy.
- `pkw audit list | verify | status`.
- Browse › Audit log, while the log is kept: a line a job, filters by days, action, result and words, each job's records beside, the chain checked, and saving as CSV. It reads the latest 50,000 records at most; `pkw audit list` reads all.

## Phase 2

- **The system's own logs**, by policy (`AuditLogSystem`): Windows' Event Log (an event source of packwing's, in Application), journald, macOS's unified log (`com.hivepaas.packwing`). Collectors already read them, and clearing Windows' takes an administrator and is an event of its own (1102). A second copy: the day's files stay the record.
  - journald: done. Its native protocol on its socket, no library; a record too long for a datagram in a sealed memory file, as journald takes those. Syslog where there is none, and on macOS until the unified log.
  - Windows: `ReportEventW` to the source `packwing`. The installer registers the source, with a message table in `packwing.exe`, so that Event Viewer reads the event as a sentence; unregistered, as a portable copy runs, events are still written whole, and Event Viewer says their description cannot be found before showing them.
  - macOS: `os_log` with the subsystem `com.hivepaas.packwing`, through a few lines of C.
- **`AuditLogRequired`**: a job whose start cannot be recorded does not start, and says why. Worth it once records go off the computer; before, the person it would stop can as well stop the log.
- **Every file extracted named**, with `entries`: hivepack tells each one written (a callback in `extract::Options`), and records hold them, to a limit.
- **Windows' SID** of the user, besides the name.

## Phase 3, if asked for

- **Checkpoints signed** with a key the user cannot read: TPM on Windows and Linux, the Secure Enclave on macOS. A checkpoint is the hash of the chain so far; one missing, or not matching, shows the log rewritten.
- **An endpoint of the organization's** (`AuditLogEndpoint`), sent each record or each checkpoint. Only where the organization sets it: packwing sends nothing anywhere otherwise.
