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

An organization can name another with `AuditLogPath`, a share among them. Settings › Security shows where it is, and Open folder opens it.

Each day's records are a file, `2026-11-08.jsonl` by the UTC date: one JSON object a line. Filebeat, Elastic Agent, Splunk's forwarder, Wazuh, Fluent Bit and Vector read it as it is.

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
| `host`, `user`, `uid` | the computer, and who ran packwing (`uid` on Unix) |
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
| `read_entry` | `source`, `entry`: `pkw cat` | `bytes` |
| `add`, `delete`, `rename` | `archive`, and what goes in, out, or the new name | |
| `copy`, `move` | `from`, `to`, `names` | `files`, `bytes`, `skipped`, `links_left` |
| `sign` | `file`, `fingerprint` | |
| `verify` | `file` | `signature` (`good`, `mismatch`, `untrusted_key`…), `fingerprint` |
| `keygen` | `folder` | `key`, `fingerprint` |
| `split`, `join` | `file` and `part_size`, or `part` | `parts`, or `made` |
| `checksum` | `file` | `sums` |
| `audit.started`, `audit.stopped`, `audit.changed` | `by` (`user`, `license`, `policy` or `startup`), `retention_days`, `max_bytes`, `folder` | |
| `log.pruned` | `removed`, `last_removed` | |

Paths are whole paths on the computer, or as a storage writes them. What the user chose to work on (`path_names`, `names`) is always named, the first 1,000 of it and how many in all.

The files inside an archive are counted, not named, unless the organization sets `AuditLogDetail` to `entries`: then records name the files picked to extract, those damaged, left out or renamed, and those deleted or renamed in a ZIP archive. A record never names every file extracted, as packwing is not told them one by one.

Never in a record: a password, a passphrase, a private key, a storage's secrets, or what is in a file.

## Changes

Each record holds `prev`, the SHA-256 of the line before it, the first of a day's file the last of the file before. `pkw audit verify` follows the chain: a record changed, taken out or put in between others shows where, and it exits with 1.

It does not stop someone who can write the files. They can write a new chain from the start, or take the last records away, and a user who runs packwing can turn the log off where no policy keeps it; the record that it was turned off stays. Whoever administers the computer can do anything to it, and anyone can build packwing without the log, as its license (GPL) allows. The log tells what packwing did. It is not a lock against the people it records, and packwing does not claim it is.

For records the user cannot change, take them off the computer as they are written: your log collector, reading the folder, or `AuditLogPath` on a share where people can add to files and not change or delete them (on Windows, *Create files / write data* denied and *Create folders / append data* allowed).

## pkw audit

```sh
pkw audit status                 # whether it is kept, where, how long, and who decides
pkw audit list --since 2026-11-01 # the records, a line each; --json, or --csv for a spreadsheet
pkw audit verify                 # the chain, from the oldest record
```

## Not yet

Planned, in [design/audit-log.md](design/audit-log.md): the system's own logs (Windows' Event Log, journald, macOS's unified log), a policy that stops a job whose record cannot be written, a page in the app to read and filter the log, and every file extracted named.
