# The audit log

packwing can keep a record of what it does on a computer: which archives were extracted, made, converted, tested and opened, which files were copied or moved between storages, signed or checked, by whom, when and how it went. It is kept on the computer, in files any log collector reads, and nothing is sent anywhere.

## Who decides

| | The log is | Who changes it |
|---|---|---|
| An organization's policy sets `AuditLog` | as the policy says, on every plan, even past a license's end | the organization ([policies.md](policies.md)) |
| No policy, a Pro or Enterprise license | off by default | the user, in Settings › Security |
| No policy, no license (Free) | off | nobody: it takes a Pro or Enterprise license |

The policy decides whatever the license, as an organization that requires a log must not lose it because a bill is late. With the policy set, the user's switch and their choice of how long are locked, and Settings shows the policy's name.

pkw keeps the log as the app does: it reads the policy, the license and the app's settings, and never changes them.

## Where

| System | Folder |
|---|---|
| Linux and the BSDs | `~/.local/state/packwing/audit/`, or `$XDG_STATE_HOME/packwing/audit/` |
| macOS | `~/Library/Logs/packwing/audit/` |
| Windows | `%LOCALAPPDATA%\HivePaaS\packwing\audit\` |

An organization can name another with `AuditLogPath`, a share among them. Settings › Security shows where it is: View records shows them in packwing, and Open folder opens the folder.

Each day's records are a file, `2026-11-08.jsonl` by the UTC date: one JSON object a line. Filebeat, Elastic Agent, Splunk's forwarder, Wazuh, Fluent Bit and Vector read it as it is.

## In the system's log

Where the organization sets `AuditLogSystem`, each record goes to the system's own log as well, as it is written. Collectors read that log already, and a user who can change the folder's files cannot take a record back out of it. The day's files stay the record, chain and all: this is a second copy. A record whose file cannot be written still goes there, with an empty `prev`.

On Linux it goes to journald, as `pkw` or `packwing` (`SYSLOG_IDENTIFIER`), with a line to read in `MESSAGE` (`extract ended: failed, /home/an/a.zip (wrong password)`) and these fields:

| Field | |
|---|---|
| `PRIORITY` | 4 (warning) for a job that failed, 5 (notice) for one partly done or cancelled and for `audit.stopped`, 6 (information) for the rest |
| `PACKWING_ACTION`, `PACKWING_PHASE`, `PACKWING_RESULT`, `PACKWING_JOB` | as the record's `action`, `phase`, `result` and `job` |
| `PACKWING_RECORD` | the record whole, its line in the day's file |
| `PACKWING_HASH` | the SHA-256 of that line: the next record's `prev` |

```sh
journalctl -t packwing -t pkw PACKWING_RESULT=failed
journalctl -t pkw -o json --all | jq -r .PACKWING_RECORD
```

`journalctl -o json` leaves out fields over 4 KiB, as a record naming many files can be: `--all` shows them. Where there is no journald, as on the BSDs, it goes to syslog, the user facility, the line to read and then the record whole.

On Windows it goes to the Application log, from the source `packwing` (the app's records and pkw's alike), as the user who ran it. The event's first string is what Event Viewer shows: the line to read, then the record whole. The next ones are for collectors, which read them as `param2` to `param6` in the event's data: `action`, `phase`, `result`, `job` and the record's hash. A job that failed is a warning, the rest information, and the event's id says how the job went:

| Event ID | |
|---|---|
| 1 | a job started |
| 2 | done |
| 3 | partly done |
| 4 | cancelled |
| 5 | failed |
| 9 | a record that is no job's: the log's own, as `audit.started` or `log.pruned` |

A record longer than an event takes, 31,839 characters, goes as its line to read and its hash; the day's file holds it whole.

Register the source once a computer, as an administrator, where you set the policy. packwing installs for the user, with no administrator's rights, so it cannot do it itself:

```powershell
New-EventLog -LogName Application -Source packwing
```

That is Windows PowerShell 5.1's, which every Windows has. Through Group Policy or Intune, it is the registry key `HKLM\SYSTEM\CurrentControlSet\Services\EventLog\Application\packwing` with `EventMessageFile` (expandable text) `%SystemRoot%\Microsoft.NET\Framework64\v4.0.30319\EventLogMessages.dll` and `TypesSupported` (DWORD) 7. Without it, events are written all the same, whole: Event Viewer only says their description cannot be found before it shows them.

On macOS it goes to the unified log, as the subsystem `com.hivepaas.packwing` and the category `audit`, the app's records and pkw's alike: the line to read, then the record whole on the next line. It is written at the default level, which the log keeps on disk, and a job that failed as an error. A record of more than 1,000 bytes, which the unified log would cut, goes as its line to read and its hash; the day's file holds it whole.

```sh
log show --last 1d --predicate 'subsystem == "com.hivepaas.packwing"'
log show --last 1d --style ndjson --predicate 'subsystem == "com.hivepaas.packwing" AND messageType == error'
```

The unified log keeps what it has for as long as the system decides, days to weeks, and a configuration profile can keep more: collectors that read it (an EDR, osquery's `unified_log`, Elastic's) take it off the computer.

`pkw audit status` says where it goes, and a record it could not be sent to shows in Settings › Security as the last that could not be written.

## When it cannot be written

A record that cannot be written, as on a share that is not there, shows in Settings › Security and in `pkw audit status`, and the job goes on: a log the user can turn off is no reason to stop their work.

Where the organization sets `AuditLogRequired`, a job whose start cannot be written does not start, and says so: *Your organization requires each job to be recorded, and the audit log cannot be written*, and why. Nothing is extracted, made, opened or copied unrecorded. Written means to the day's file, and to the system's log too where `AuditLogSystem` sends records there. A job's end that cannot be written does not undo what the job did; it shows as above, and the job's start, with no end, stays.

## How long

| | Kept | At most |
|---|---|---|
| The user's own | 30 days, 90 (the default), a year, or all, as Settings › Security says | 100 MiB |
| The organization's | `AuditLogRetentionDays`, 365 days if not set; 0 for all | `AuditLogMaxSizeMB`, 1 GiB if not set; 0 for no limit |

Past either, the oldest days' files are taken away, when packwing or pkw starts and when the setting changes, never today's. Taking them away is itself a record (`log.pruned`), with the files taken and where the chain of what is left starts.

## What a record holds

Every record has:

| Field | |
|---|---|
| `v` | the version of what records hold: 1 |
| `time` | UTC, to the millisecond |
| `prev` | the SHA-256 of the line before it (see [Changes](#changes)) |
| `host`, `user`, `uid`, `sid` | the computer, and who ran packwing (`uid` on Unix, `sid` on Windows) |
| `app`, `version`, `pid`, `session` | `packwing` or `pkw`, its version, its process and run |
| `action` | what was done |

A job is two records with the same `job`: `phase: start`, with what it is of, and `phase: end`, with `result` (`ok`, `partial`, `failed` or `cancelled`), `error` when it failed, and `duration_ms`. A job that started and has no end was stopped with packwing.

| `action` | Of | At its end |
|---|---|---|
| `extract` | `source`, `destination`, and how: `overwrite`, `keep_mark`, `portable_names`, `links_outside`, `owners`, `xattrs`, `limit`; `picked` where only some were | `format`, `layers`, `files`, `directories`, `links`, `bytes`, how many files were `damaged`, `left_out` and `renamed`, `owners_left`. `partial` where some were damaged or left out |
| `create` | `destination`, `kind`, `level`, `encryption` (`zip-aes256`, `7z-aes256`, `age-passphrase`, `age-key`), `hide_names`, `recipients` (the age keys, which are public), `part_size`, `keep_owners`, `keep_xattrs`, `paths` and `path_names` | `made` |
| `convert` | `source`, `destination`, and as `create` | as `extract`, and `made` |
| `test` | `source` | `files`, `bytes` |
| `open_entry` | `source`, `entry`: a file opened from an archive | `copy`: where its copy is |
| `read_entry` | `source`, `entry`: `pkw cat`, or a file shown beside its archive (`via: preview`) | `bytes` |
| `add`, `delete`, `rename` | `archive`, and what goes in, out, or the new name | |
| `copy`, `move` | `from`, `to`, `names` | `files`, `bytes`, `skipped`, `links_left` |
| `sign` | `file`, `fingerprint` | |
| `verify` | `file` | `signature` (`good`, `mismatch`, `untrusted_key`…), `fingerprint` |
| `keygen` | `folder` | `key`, `fingerprint` |
| `split`, `join` | `file` and `part_size`, or `part` | `parts`, or `made` |
| `checksum` | `file` | `sums` |
| `audit.started`, `audit.stopped`, `audit.changed` | `by` (`user`, `license`, `policy` or `startup`), `retention_days`, `max_bytes`, `folder`, `system_log` | |
| `log.pruned` | `removed`, `last_removed` | |

Paths are whole paths on the computer, or as a storage writes them. What the user chose to work on is always named, the first 1,000 of it and how many in all: the files and folders to put in an archive, copy or move (`path_names`, `names`), and in an archive those picked to extract, opened, shown, deleted or renamed (`picked_names`, `entry`, `from_name`, `to_name`).

What packwing comes on as it works is counted, not named, unless the organization sets `AuditLogDetail` to `entries`: then records name the files damaged, left out or renamed as they were extracted. A record never names every file extracted, as packwing is not told them one by one.

A file shown beside its archive is recorded the first time in a run only: going through an archive shows many, and seeing one again says nothing new. `pkw cat` is recorded each time.

Never in a record: a password, a passphrase, a private key, a storage's secrets, or what is in a file.

## Changes

Each record holds `prev`, the SHA-256 of the line before it, the first of a day's file the last of the file before. `pkw audit verify` follows the chain: a record changed, taken out or put in between others shows where, and it exits with 1.

It does not stop someone who can write the files. They can write a new chain from the start, or take the last records away, and a user who runs packwing can turn the log off where no policy keeps it; the record that it was turned off stays. Whoever administers the computer can do anything to it, and anyone can build packwing without the log, as its license (GPL) allows. The log tells what packwing did. It is not a lock against the people it records, and packwing does not claim it is.

For records the user cannot change, take them off the computer as they are written: your log collector, reading the folder, or `AuditLogPath` on a share where people can add to files and not change or delete them (on Windows, *Create files / write data* denied and *Create folders / append data* allowed).

## Reading it

While the log is kept, Browse has an **Audit log** tab, after Recent; **View records** in Settings › Security leads there too. It shows a line a job, the latest first: what was done, what it was of, when (UTC), how it went and how long it took. Choose the days (today, the last 7 or 30, or all), an action, done or not, or type part of a path, a name or an error to find. A line clicked shows its records whole beside the list.

Above the list, packwing checks the chain, as `pkw audit verify` does, and says whether a record was changed or taken out. **Save as CSV…** saves the lines shown, a job a row: `time`, `action`, `result` (`ok`, `partial`, `failed`, `cancelled`, or `unfinished` for a job with no end), `subject`, `error`, `duration_ms`, `user`, `host`, `app`, `job`.

The tab reads the latest 50,000 records at most, and says so when there are more: choose fewer days, or read them all with `pkw audit list`. It never changes the log.

## pkw audit

```sh
pkw audit status                 # whether it is kept, where, how long, and who decides
pkw audit list --since 2026-11-01 # the records, a line each; --json, or --csv for a spreadsheet
pkw audit verify                 # the chain, from the oldest record
```

## Not yet

Planned, in [design/audit-log.md](design/audit-log.md): every file extracted named.
