---
title: PowerShell Environment Variables
slug: /powershell/syntax/environment-variables/
summary: 'Reference for reading, writing, listing, persisting, and removing environment variables in PowerShell using the $env: drive and the .NET Environment class.'
topic: powershell/syntax
type: reference
tags: [powershell, environment-variables, env, path, setx, dotnet]
aliases: [powershell env var, $env powershell, set environment variable powershell, persist env var powershell, add to path powershell, list environment variables powershell]
platforms: [windows, powershell]
related:
  - /powershell/syntax/cmdlet-patterns-and-filtering/
  - /powershell/profiles/console-and-profile-customization/
  - /powershell/syntax/modules-and-psgallery/
  - /powershell/syntax/credentials-and-secure-strings/
status: published
updated: 2026-05-14
---

## Synopsis

Environment variables in PowerShell are exposed through the `Env:` PSDrive. The `$env:` prefix gives you fast, scriptable access to read and write them in the current process, while `[System.Environment]` lets you persist values to the User or Machine scope so they survive across sessions.

## Syntax

```powershell
$env:<NAME>                                       # read
$env:<NAME> = '<value>'                           # write (current process only)
Get-ChildItem Env:                                # list all
[Environment]::GetEnvironmentVariable('NAME', 'User')
[Environment]::SetEnvironmentVariable('NAME', 'value', 'User')
```

## Parameters/Flags

- `$env:<NAME>` — alias into the `Env:` PSDrive, reads/writes the current process scope.
- `Get-ChildItem Env:` / `Get-Item Env:NAME` — provider-based access for listing and inspection.
- `Remove-Item Env:NAME` — delete a variable from the current process.
- `[Environment]::GetEnvironmentVariable(<name>, <scope>)` — read a specific scope explicitly.
- `[Environment]::SetEnvironmentVariable(<name>, <value>, <scope>)` — write to a specific scope; pass `$null` as value to delete.
- Scope values: `'Process'` (current shell), `'User'` (persisted in `HKCU\Environment`), `'Machine'` (persisted in `HKLM`, requires admin).

## Scopes

Environment variables exist in three scopes. Knowing the difference is the single most important thing about working with them:

- **Process**: lives only in the current PowerShell session. `$env:NAME = 'x'` writes here. Disappears when the shell closes.
- **User**: persisted in the registry under `HKCU\Environment`. Available to all future processes started by this user.
- **Machine**: persisted under `HKLM\System\CurrentControlSet\Control\Session Manager\Environment`. Available to every user on the system. Requires administrator privileges to modify.

When a new process starts, Windows merges Machine and User values into its initial environment block. Changes to User/Machine scope do **not** retroactively update already-running shells — you must open a new session (or update `$env:` manually) to see them.

## Examples

### Read A Variable

```powershell
$env:USERNAME
$env:USERPROFILE
$env:PATH
```

Use inside strings with normal variable interpolation:

```powershell
Write-Host "Hello $env:USERNAME, your home is $env:USERPROFILE"
```

### Check Whether A Variable Is Set

```powershell
if ($env:MY_FLAG) {
    Write-Host "MY_FLAG is set to '$env:MY_FLAG'"
} else {
    Write-Host "MY_FLAG is not set"
}
```

`Test-Path` works against the `Env:` drive too:

```powershell
if (Test-Path Env:MY_FLAG) { 'set' } else { 'not set' }
```

### Set A Variable For The Current Session

```powershell
$env:MY_FLAG = 'true'
$env:API_BASE_URL = 'https://api.example.com'
```

This only affects the current PowerShell process and any child processes it spawns. Closing the shell discards the value.

### List All Environment Variables

```powershell
Get-ChildItem Env:
```

Sort, filter, or format the output like any other cmdlet:

```powershell
Get-ChildItem Env: | Sort-Object Name | Format-Table -AutoSize
Get-ChildItem Env: | Where-Object { $_.Name -like 'PATH*' }
```

### Persist A Variable (User Scope)

```powershell
[Environment]::SetEnvironmentVariable('MY_FLAG', 'true', 'User')
```

The value is written to the registry and is available in every new shell from now on. The current shell does not see it until you also assign `$env:MY_FLAG`, or restart the session.

### Persist A Variable (Machine Scope, Requires Admin)

```powershell
[Environment]::SetEnvironmentVariable('COMPANY_REGION', 'us-east-1', 'Machine')
```

Run this from an elevated PowerShell prompt. The value will be visible to every user on the machine.

### Read A Persisted Value Explicitly

`$env:NAME` returns the merged process value. To read the persisted User or Machine value directly:

```powershell
[Environment]::GetEnvironmentVariable('MY_FLAG', 'User')
[Environment]::GetEnvironmentVariable('MY_FLAG', 'Machine')
[Environment]::GetEnvironmentVariable('MY_FLAG', 'Process')
```

This is useful when you suspect the User value differs from what the current process sees.

### Delete A Variable

For the current session:

```powershell
Remove-Item Env:MY_FLAG
```

For persisted scopes, set the value to `$null`:

```powershell
[Environment]::SetEnvironmentVariable('MY_FLAG', $null, 'User')
[Environment]::SetEnvironmentVariable('MY_FLAG', $null, 'Machine')
```

### Append To PATH

Append a directory to PATH for the current session only:

```powershell
$env:PATH += ';C:\Tools\bin'
```

Append a directory to the User PATH permanently:

```powershell
$current = [Environment]::GetEnvironmentVariable('Path', 'User')
[Environment]::SetEnvironmentVariable('Path', "$current;C:\Tools\bin", 'User')
```

Read the User PATH first instead of `$env:PATH` — `$env:PATH` is the merged Machine+User+Process value, and writing that back into User scope would duplicate every Machine entry into your User PATH.

### Split PATH Into Individual Entries

PATH is one long string with `;` separators. Split it to inspect:

```powershell
$env:PATH -split ';'
```

Or list them numbered:

```powershell
$env:PATH -split ';' | ForEach-Object { $i++; "{0,2}: {1}" -f $i, $_ }
```

### Remove A Directory From PATH

```powershell
$dirs = ($env:PATH -split ';') | Where-Object { $_ -ne 'C:\Tools\bin' }
$env:PATH = $dirs -join ';'
```

Persist the same change to User scope:

```powershell
$current = [Environment]::GetEnvironmentVariable('Path', 'User')
$cleaned = ($current -split ';' | Where-Object { $_ -and $_ -ne 'C:\Tools\bin' }) -join ';'
[Environment]::SetEnvironmentVariable('Path', $cleaned, 'User')
```

### Use Variables With External Commands

Environment variables you set in PowerShell are inherited by any process the shell launches. This is the standard way to pass configuration into CLI tools:

```powershell
$env:NODE_ENV = 'production'
npm run build

$env:AWS_PROFILE = 'staging'
aws s3 ls
```

### Set A Variable For A Single Command

Unlike bash, PowerShell does not support `VAR=value command` inline syntax. Wrap the assignment and the command in a script block, or set it in a child scope:

```powershell
& { $env:NODE_ENV = 'production'; npm run build }
```

Once the script block exits, `$env:NODE_ENV` returns to whatever it was before (or unset).

### Common Built-In Variables

| Variable                  | What it holds                                  |
| ------------------------- | ---------------------------------------------- |
| `$env:USERNAME`           | Current user's login name                      |
| `$env:USERPROFILE`        | User's home directory (e.g., `C:\Users\louis`) |
| `$env:COMPUTERNAME`       | Machine's NetBIOS name                         |
| `$env:OS`                 | Operating system family (e.g., `Windows_NT`)   |
| `$env:PATH`               | Executable search path                         |
| `$env:PATHEXT`            | Extensions treated as executable               |
| `$env:TEMP` / `$env:TMP`  | Temp directory for the current user            |
| `$env:APPDATA`            | Roaming app data                               |
| `$env:LOCALAPPDATA`       | Local (non-roaming) app data                   |
| `$env:PROGRAMFILES`       | 64-bit Program Files directory                 |
| `$env:PSModulePath`       | Where PowerShell looks for modules             |

### setx vs. [Environment]::SetEnvironmentVariable

`setx` is the classic command-line tool for persisting env vars:

```powershell
setx MY_FLAG "true"
```

It works, but has two gotchas to know about:
1. Values are silently truncated at 1024 characters — bad for PATH edits.
2. It does not update the current session, so `$env:MY_FLAG` is still empty after running it.

Prefer `[Environment]::SetEnvironmentVariable` for anything programmatic, especially PATH manipulation.

## Related

- [`PowerShell Cmdlet Patterns And Filtering`](/powershell/syntax/cmdlet-patterns-and-filtering/)
- [`PowerShell Console And Profile Customization`](/powershell/profiles/console-and-profile-customization/)
- [`PowerShell Pause And Output Patterns`](/powershell/syntax/pause-and-output-patterns/)
