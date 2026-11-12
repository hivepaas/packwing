# Policies, for administrators

An organization can fix some of packwing's behaviour on the computers it manages. packwing reads policies from the system when it starts, so a change takes effect at its next start. It needs no account, no server and no license for this.

To see what a computer has, and any mistake in it:

```sh
packwing --policies
```

It exits with 1 if something cannot be understood. Settings › Security shows the same to the user, each policy locked, with its name.

## What policies are not

They bind packwing, not the person. An administrator of the computer can change them. Anyone can build packwing without them, as its license (GPL) allows, or use another app. They are not a security boundary against a user who has either, and packwing does not claim to be one.

## The policies

| Name | Kind | What it does |
|---|---|---|
| `PropagateMotW` | yes or no | Whether what is extracted from a file marked as downloaded keeps the mark (the Mark of the Web on Windows, quarantine on macOS). Replaces the user's switch. It is on by default, without a policy. Linux has no such mark, so there it does nothing. |
| `MarkStorageFiles` | yes or no | Whether a file from a storage across a network (S3, WebDAV, SFTP) counts as from the Internet: what is extracted or copied from one gets the mark. It does without a policy. Turn it off for storages your organization trusts. `PropagateMotW` still decides whether marks are written at all. |
| `RequireEncryption` | yes or no | New archives have to be encrypted, wherever they are made from: New archive, the right-click menu, dragging files in, and to a storage. Files cannot be added to a ZIP archive whose files are not encrypted. Deleting and renaming in one still work, as they bring nothing new in the clear. |
| `AllowedEncryptionMethods` | list | The ways a new archive may be encrypted: `zip-aes256`, `7z-aes256`, `age-passphrase`, `age-key`. Others are refused. It says how, not whether: `RequireEncryption` says whether. |
| `MinPasswordLength` | number, 1 to 1024 | The least length of a password or passphrase for a new archive, in characters. 15 is a good value: an archive's password can be guessed offline, as fast as the attacker's hardware goes. With this set, packwing also refuses the 10,000 most common passwords, one character repeated and plain runs such as `123456789012345`. It makes no rules about kinds of characters. It does not apply to age keys, nor to a password already on an archive. |
| `LicenseKey` | text | Your organization's license key, so that nobody has to enter one. It lifts the limit on storages for everyone who gets it. Nothing is sent anywhere, and `packwing --policies` does not show it. A key that has ended never changes what the other policies do. |
| `LicenseRefresh` | on/off | With `LicenseKey`, packwing asks `portal.hivepaas.com` once a day for a newer key of it, sending the key it holds and nothing else, and uses the newer key the portal answers: a new year, more seats, or the end of a subscription reach every computer within a day, with no key deployed again. It is the only thing packwing ever sends about a license. Off where not set, and never for a key a user entered. |
| `ManagedByOrganizationName` | text | Who is managing this, as the user sees it in Settings. |
| `ManagedByURL` | text | Where the user can ask for help. It has to start with `https://` or `http://`. |
| `AuditLog` | yes or no | Whether packwing keeps an audit log of what it does ([audit.md](audit.md)), on every computer it applies to, whatever the license, and even past a license's end. Replaces the user's switch; without it, a user with a Pro or Enterprise license chooses, off by default. |
| `AuditLogPath` | text | The folder the log is kept in: a whole path, a share's among them (`\\files\audit\{user}`). `{user}` and `{host}` stand for who runs packwing and the computer's name. Without it, the system's own folder for the user ([audit.md](audit.md#where)). |
| `AuditLogRetentionDays` | number, 0 to 36500 | How many days of records are kept; 0 keeps them all, for your own collector to rotate. 365 where `AuditLog` is on and this is not set. |
| `AuditLogMaxSizeMB` | number, 0 to 10000000 | The most the log may take, in MB: past it, the oldest days go, never today's. 0 for no limit. 1024 where `AuditLog` is on and this is not set. |
| `AuditLogDetail` | text | `summary` (the default) or `entries`: whether records name the files packwing comes on in archives, as those damaged or left out and every one extracted into a folder, as well as counting them. What the user picks is named either way. |
| `AuditLogSystem` | on/off | Each record goes to the system's own log as well, for collectors that read it: the Event Log on Windows, from the source `packwing`, which an administrator registers once ([audit.md](audit.md#in-the-systems-log)); the unified log on macOS, as `com.hivepaas.packwing`; journald on Linux, syslog where there is none. Off where not set; only a policy turns it on. |
| `AuditLogRequired` | on/off | A job whose start the log cannot write does not start, and says why: an archive is not extracted, made or opened unrecorded. Written means to the day's file, and to the system's log where `AuditLogSystem` sends it there too. It counts where the log is kept: set `AuditLog` with it. Off where not set. |

A list can also be written as one text with commas between the names, for tools that cannot write lists.

`RequireEncryption` covers what packwing makes. It says nothing about files on the computer or what other apps make, and a ZIP archive's file names stay readable even when its files are encrypted (use `7z-aes256` with its hidden names, or age, for that).

## When a value is wrong

A mistake never loosens anything, as a typing error must not quietly take a lock off:

- `PropagateMotW` that cannot be understood keeps the mark on, locked.
- `MarkStorageFiles` that cannot be understood keeps marking.
- `RequireEncryption` that cannot be understood refuses making any archive.
- `AllowedEncryptionMethods` that cannot be understood, or names a method that does not exist, or names none, refuses encrypting.
- `MinPasswordLength` that cannot be understood refuses passwords.
- A name that is not a policy is reported, and does nothing else.
- `ManagedByURL` and `ManagedByOrganizationName` guard nothing: they are reported, and not shown.
- `AuditLog` that cannot be understood keeps the log, locked.
- `AuditLogPath` that is not a whole path is reported, and the system's own folder taken.
- `AuditLogRetentionDays` and `AuditLogMaxSizeMB` that cannot be understood keep every record.
- `AuditLogDetail` that cannot be understood names the files.
- `AuditLogSystem` that cannot be understood writes to the system's log too.
- `AuditLogRequired` that cannot be understood requires the log.

Where the policies cannot be read at all, as in a file that is not JSON, all of the above apply together.

## Windows

Values in the registry, under:

```
HKEY_LOCAL_MACHINE\SOFTWARE\Policies\HivePaaS\Packwing
HKEY_CURRENT_USER\SOFTWARE\Policies\HivePaaS\Packwing
```

The computer's value wins over the user's. Yes or no is a `REG_DWORD` of 0 or 1, a number a `REG_DWORD`, text a `REG_SZ`, a list a `REG_MULTI_SZ`.

```powershell
$key = "HKLM:\SOFTWARE\Policies\HivePaaS\Packwing"
New-Item -Path $key -Force
New-ItemProperty -Path $key -Name RequireEncryption -PropertyType DWord -Value 1
New-ItemProperty -Path $key -Name MinPasswordLength -PropertyType DWord -Value 15
New-ItemProperty -Path $key -Name AllowedEncryptionMethods -PropertyType MultiString -Value "7z-aes256","age-key"
```

## macOS

A configuration profile with a `com.apple.ManagedClient.preferences` payload for the domain `com.hivepaas.packwing`, its values under `Forced`. Only forced values count: what a user writes with `defaults write` is not a policy. Yes or no is a boolean, a number an integer, text a string, a list an array of strings.

## Linux and the BSDs

One JSON object in `/etc/packwing/policy.json`:

```json
{
  "ManagedByOrganizationName": "IT department",
  "RequireEncryption": true,
  "MinPasswordLength": 15,
  "AllowedEncryptionMethods": ["7z-aes256", "age-key"],
  "AuditLog": true,
  "AuditLogPath": "/var/log/packwing/{user}"
}
```

The file and its folder have to belong to root and be writable by no one else. If not, packwing does not believe the file and refuses as above.
