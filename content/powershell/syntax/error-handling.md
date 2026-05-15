---
title: PowerShell Error Handling
slug: /powershell/syntax/error-handling/
summary: Reference for try/catch/finally, terminating versus non-terminating errors, $ErrorActionPreference, the -ErrorAction and -ErrorVariable common parameters, typed catch blocks, throw, and $LASTEXITCODE.
topic: powershell/syntax
type: reference
tags: [powershell, error-handling, try-catch, throw, erroraction, exceptions]
aliases: [powershell try catch, powershell terminating error, powershell erroractionpreference, powershell catch specific exception, powershell lastexitcode, powershell error variable]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/syntax/functions-and-parameters/
  - /powershell/syntax/output-streams-and-redirection/
  - /powershell/networking/rest-and-web-requests/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell has two flavors of errors — terminating and non-terminating — and `try`/`catch` only catches the terminating kind. Most cmdlet errors are non-terminating by default, which is why scripts that wrap `Get-Item missing.txt` in `try` block do nothing useful. The fix is `-ErrorAction Stop` per call or `$ErrorActionPreference = 'Stop'` globally.

## Syntax

```powershell
try {
    <code>
} catch [ExceptionType], [OtherType] {
    <handler using $_>
} catch {
    <generic handler>
} finally {
    <always runs>
}

throw <message-or-object>
Write-Error <message> -ErrorAction Stop

<cmdlet> -ErrorAction Stop -ErrorVariable err
$Error[0]
```

## Parameters/Flags

`-ErrorAction` accepts:

- `Continue` (default) — report and keep going
- `SilentlyContinue` — suppress entirely (still adds to `$Error`)
- `Stop` — promote to terminating so `try`/`catch` sees it
- `Inquire` — prompt the user
- `Ignore` — suppress and do **not** add to `$Error` (PS3+)

`$ErrorActionPreference` accepts the same values (sans `Ignore`) and sets the default for the whole session.

Automatic variables:

- `$_` (inside `catch`) — the `ErrorRecord`
- `$Error` — circular buffer of recent errors; `$Error[0]` is the most recent
- `$?` — `$true` if the previous command succeeded
- `$LASTEXITCODE` — exit code of the last native executable

## Examples

### Basic Try / Catch / Finally

```powershell
try {
    $content = Get-Content -Path 'C:\missing.txt' -ErrorAction Stop
}
catch {
    Write-Warning "Could not read file: $($_.Exception.Message)"
}
finally {
    Write-Host 'cleanup runs whether or not the try succeeded'
}
```

`finally` runs on success, on caught exception, and on uncaught exception. Use it for releasing resources (closing files, disposing connections).

### Terminating Vs Non-Terminating Errors

```powershell
try {
    Get-Item 'C:\missing.txt'    # non-terminating — try/catch will NOT fire
} catch {
    'caught'                     # never runs
}

try {
    Get-Item 'C:\missing.txt' -ErrorAction Stop   # promoted to terminating
} catch {
    'caught'                     # this runs
}
```

This single mechanic is the source of most "my try/catch doesn't work" questions. Always add `-ErrorAction Stop` to cmdlets you actually want to catch.

### Catching Specific Exception Types

```powershell
try {
    [int]::Parse('not a number')
}
catch [System.FormatException] {
    'bad format'
}
catch [System.OverflowException] {
    'too big'
}
catch {
    "unhandled: $($_.Exception.GetType().FullName)"
}
```

Order matters — first matching `catch` wins. Put specific types before generic ones.

To learn what type to catch, inspect a thrown error:

```powershell
$Error[0].Exception.GetType().FullName
```

### The Error Record

Inside a `catch`, `$_` is a rich `ErrorRecord`:

```powershell
catch {
    $_.Exception.Message             # human message
    $_.Exception.GetType().FullName  # type to catch
    $_.CategoryInfo                  # category, target, action
    $_.FullyQualifiedErrorId         # stable error id (good for switch/if)
    $_.ScriptStackTrace              # where it happened
    $_.InvocationInfo.PositionMessage
}
```

### Throw

`throw` always produces a terminating error:

```powershell
function Test-Even {
    param([int]$n)
    if ($n % 2 -ne 0) { throw "Expected even number, got $n" }
    $true
}
```

Throw a structured exception when callers might want to catch by type:

```powershell
throw [System.ArgumentException]::new('value must be positive', 'count')
```

### Write-Error Vs Throw

| Operation               | Terminating? | Use when                                  |
| ----------------------- | ------------ | ----------------------------------------- |
| `Write-Error 'msg'`     | No           | Reporting a recoverable problem           |
| `Write-Error -EA Stop`  | Yes          | Same, but you also want to halt           |
| `throw 'msg'`           | Yes          | Aborting and unwinding to nearest `catch` |

### Suppressing Errors

```powershell
Get-Item 'C:\missing.txt' -ErrorAction SilentlyContinue
Get-Item 'C:\missing.txt' -ErrorAction Ignore       # not even in $Error

# Per-script:
$ErrorActionPreference = 'SilentlyContinue'
```

`SilentlyContinue` is the right choice when you check a condition and don't care about the noise:

```powershell
$file = Get-Item 'maybe.txt' -ErrorAction SilentlyContinue
if ($file) { 'exists' } else { 'missing' }
```

### Capturing Errors With -ErrorVariable

```powershell
Get-Item a.txt, b.txt, c.txt -ErrorVariable err -ErrorAction SilentlyContinue
if ($err) {
    "$($err.Count) files failed:"
    $err | ForEach-Object { " - $($_.TargetObject): $($_.Exception.Message)" }
}
```

Prefix the variable name with `+` to append instead of overwrite:

```powershell
Get-Item a.txt -ErrorVariable +allErrors -ErrorAction SilentlyContinue
Get-Item b.txt -ErrorVariable +allErrors -ErrorAction SilentlyContinue
```

### Native Commands And $LASTEXITCODE

`try`/`catch` does **not** catch failures from native (non-PowerShell) executables. Inspect `$LASTEXITCODE` instead:

```powershell
git push origin main
if ($LASTEXITCODE -ne 0) {
    throw "git push failed with exit code $LASTEXITCODE"
}
```

PowerShell 7.4+ has `$PSNativeCommandUseErrorActionPreference` which, when set to `$true`, makes native command failures honor `$ErrorActionPreference` — letting you `try`/`catch` them like normal cmdlets.

### Rethrowing

To pass an exception up while adding context, throw the original record:

```powershell
try {
    Connect-Database
}
catch {
    Write-Host 'failed during DB connect'
    throw    # bare throw re-throws the current error record
}
```

### The Trap Statement (Legacy)

`trap` is a function-scoped handler that runs on any terminating error and then continues to the next statement (unless you `break`):

```powershell
function Test-Trap {
    trap {
        Write-Warning "caught: $($_.Exception.Message)"
        continue
    }
    throw 'oops'
    'after throw'                    # still runs because trap used continue
}
```

`try`/`catch` is preferred for new code; `trap` shows up mostly in older scripts.

### A Solid Default For Scripts

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

try {
    Main-Work
}
catch {
    Write-Host "FATAL: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace
    exit 1
}
```

Pairing `Stop` with `Set-StrictMode` catches typos in variable names and unset properties early, before they become silent bugs.

## Related

- [`PowerShell Functions And Parameters`](/powershell/syntax/functions-and-parameters/)
- [`PowerShell Output Streams And Redirection`](/powershell/syntax/output-streams-and-redirection/)
- [`PowerShell REST APIs And Web Requests`](/powershell/networking/rest-and-web-requests/)
