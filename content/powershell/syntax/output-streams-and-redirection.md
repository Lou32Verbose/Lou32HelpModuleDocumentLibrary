---
title: PowerShell Output Streams And Redirection
slug: /powershell/syntax/output-streams-and-redirection/
summary: Reference for PowerShell's six output streams, the Write-* cmdlets, preference variables that control visibility, redirection operators, and the Write-Host vs Write-Output distinction.
topic: powershell/syntax
type: reference
tags: [powershell, streams, write-host, write-output, redirection, verbose, warning, debug]
aliases: [powershell write-host vs write-output, powershell redirect stream, powershell 2>&1, powershell suppress output, powershell verbosepreference]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/syntax/error-handling/
  - /powershell/syntax/pause-and-output-patterns/
  - /powershell/syntax/functions-and-parameters/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell separates console output into six independent streams. Only stream 1 (Success) flows down the pipeline; the others are diagnostic channels for errors, warnings, and progress. Knowing which stream a `Write-*` cmdlet targets — and how to redirect or silence each — is what separates clean scripts from scripts that flood the console.

## Syntax

```powershell
Write-Output  <value>            # stream 1 (Success / pipeline)
Write-Error   <message>          # stream 2 (Error)
Write-Warning <message>          # stream 3 (Warning)
Write-Verbose <message>          # stream 4 (Verbose)
Write-Debug   <message>          # stream 5 (Debug)
Write-Information <message>      # stream 6 (Information)
Write-Host    <value>            # writes to Information stream + console

<command> > file.txt              # redirect Success to file
<command> 2>&1                    # merge Error into Success
<command> *> all.log              # redirect every stream to file
<command> > $null                 # discard Success
```

## Parameters/Flags

The six streams by number:

| # | Name        | Default visibility | Cmdlet              | Preference variable      |
| - | ----------- | ------------------ | ------------------- | ------------------------ |
| 1 | Success     | shown              | `Write-Output`      | n/a                      |
| 2 | Error       | shown (red)        | `Write-Error`       | `$ErrorActionPreference` |
| 3 | Warning     | shown (yellow)     | `Write-Warning`     | `$WarningPreference`     |
| 4 | Verbose     | hidden             | `Write-Verbose`     | `$VerbosePreference`     |
| 5 | Debug       | hidden             | `Write-Debug`       | `$DebugPreference`       |
| 6 | Information | shown (PS5+)       | `Write-Information` | `$InformationPreference` |

Preference variable values: `SilentlyContinue` (hide), `Continue` (show), `Inquire` (prompt), `Stop` (throw).

## Examples

### Write-Output Vs Write-Host

```powershell
function Get-Greeting {
    Write-Output 'hello'         # goes to pipeline — usable
    Write-Host   'starting...'   # console only — informational
}

$result = Get-Greeting           # $result = 'hello'; 'starting...' was printed
```

`Write-Output` (and bare expressions) emit objects that downstream cmdlets and variable assignments can consume. `Write-Host` writes to the host display only — perfect for status banners, never use it to "return" data.

In PowerShell 5+, `Write-Host` actually writes to the Information stream (6) and is no longer the deprecated antipattern it once was. Use it intentionally for human-facing output.

### Implicit Output

Bare expressions are equivalent to `Write-Output`:

```powershell
function Get-Names {
    'alice'                      # same as Write-Output 'alice'
    'bob'
}

Get-Names                        # alice, bob
```

This is why accidental output (a stray expression you forgot to assign or pipe to `Out-Null`) leaks into the return value of functions.

### Enabling Verbose And Debug Output

`Write-Verbose` and `Write-Debug` are silent unless turned on. Two ways to enable them:

```powershell
# Per-call (function must use [CmdletBinding()]):
Get-Foo -Verbose
Get-Foo -Debug

# Globally for the session:
$VerbosePreference = 'Continue'
$DebugPreference   = 'Continue'
```

A function that participates correctly:

```powershell
function Get-Foo {
    [CmdletBinding()]
    param()
    Write-Verbose 'starting'
    Write-Debug   "value of x is $x"
    Write-Output  'result'
}
```

### Warning Vs Error

```powershell
Write-Warning 'config file missing — using defaults'
Write-Error   'cannot connect to database'
```

`Write-Error` writes to stream 2 but does **not** stop execution by default. To make a script halt:

```powershell
$ErrorActionPreference = 'Stop'
# or
Write-Error 'fatal' -ErrorAction Stop
# or use throw, which is always terminating:
throw 'fatal'
```

### Redirection Operators

| Operator | Effect                                |
| -------- | ------------------------------------- |
| `>`      | Redirect Success (1) to file (overwrite) |
| `>>`     | Redirect Success (1) to file (append)    |
| `2>`     | Redirect Error to file                   |
| `3>`     | Redirect Warning                         |
| `4>`     | Redirect Verbose                         |
| `5>`     | Redirect Debug                           |
| `6>`     | Redirect Information                     |
| `*>`     | Redirect **all** streams                 |
| `2>&1`   | Merge Error into Success                 |
| `*>&1`   | Merge every stream into Success          |

Common patterns:

```powershell
git status > status.txt                    # save Success output
git status 2> errors.txt                   # save errors separately
git status 2>&1 > combined.txt             # both into one file
git status *> everything.log               # every stream

Get-ChildItem -ErrorAction SilentlyContinue *> $null   # suppress everything
```

### Discarding Output

Five ways to throw away stream 1; pick by intent and readability:

```powershell
$null = New-Item foo.txt
[void](New-Item foo.txt)
New-Item foo.txt | Out-Null
New-Item foo.txt > $null
New-Item foo.txt 1> $null
```

`$null = ...` and `[void]` are the fastest in tight loops. `| Out-Null` is the most idiomatic.

### Capturing One Stream Into A Variable

Common parameters offer per-stream variable capture:

```powershell
Get-Item missing.txt -ErrorVariable err -ErrorAction SilentlyContinue
$err                              # the error record(s)

Get-ChildItem -WarningVariable warns -WarningAction SilentlyContinue
$warns
```

Append (rather than overwrite) by prefixing with `+`:

```powershell
Get-Item a, b, c -ErrorVariable +allErrors -ErrorAction SilentlyContinue
```

### Preference Variables Cheat Sheet

Set at the top of a script to control noise globally:

```powershell
$ErrorActionPreference   = 'Stop'             # treat errors as terminating
$WarningPreference       = 'Continue'         # show warnings (default)
$VerbosePreference       = 'Continue'         # show verbose
$DebugPreference         = 'SilentlyContinue' # hide debug (default)
$InformationPreference   = 'Continue'         # show Write-Information
$ProgressPreference      = 'SilentlyContinue' # hide Write-Progress bars
```

`$ProgressPreference = 'SilentlyContinue'` is especially useful before `Invoke-WebRequest` in PS5.1, where the progress bar dramatically slows downloads.

### Quick Reference: Hide All Noise

```powershell
& {
    $ErrorActionPreference = 'SilentlyContinue'
    $WarningPreference     = 'SilentlyContinue'
    $ProgressPreference    = 'SilentlyContinue'
    Invoke-Something
} *> $null
```

## Related

- [`PowerShell Error Handling`](/powershell/syntax/error-handling/)
- [`PowerShell Pause And Output Patterns`](/powershell/syntax/pause-and-output-patterns/)
- [`PowerShell Functions And Parameters`](/powershell/syntax/functions-and-parameters/)
