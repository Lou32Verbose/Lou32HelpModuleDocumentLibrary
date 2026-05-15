---
title: PowerShell Path Manipulation
slug: /powershell/filesystem/path-manipulation/
summary: Reference for joining, splitting, resolving, and testing filesystem paths in PowerShell using Join-Path, Split-Path, Resolve-Path, Convert-Path, Test-Path, and the script-location automatic variables.
topic: powershell/filesystem
type: reference
tags: [powershell, paths, join-path, split-path, resolve-path, test-path, psscriptroot]
aliases: [powershell join path, powershell split path, powershell get script directory, powershell psscriptroot, powershell test-path pathtype, powershell resolve relative path]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/filesystem/file-and-text-recipes/
  - /powershell/syntax/environment-variables/
  - /powershell/filesystem/registry-operations/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell treats paths as strings but provides cmdlets so you don't have to hand-roll separator logic that breaks across platforms. Use `Join-Path` and `Split-Path` to build and decompose paths, `Resolve-Path` to canonicalize them, and `$PSScriptRoot` to anchor relative references inside scripts.

## Syntax

```powershell
Join-Path  <parent> <child>                # parent + child with correct separator
Split-Path <path>   -Parent | -Leaf | -Extension | -LeafBase | -Qualifier
Resolve-Path <path>                        # wildcards → matched provider paths
Convert-Path <path>                        # provider path → literal filesystem path
Test-Path    <path>   -PathType Container | Leaf

$PSScriptRoot          # directory containing the running script
$PSCommandPath         # full path of the running script
$MyInvocation          # invocation details
```

## Parameters/Flags

- `Join-Path -Resolve` — also verify the result exists; errors if not
- `Split-Path -Parent` — directory (default if no switch given)
- `Split-Path -Leaf` — filename portion
- `Split-Path -Extension` — `.txt` (PS6+)
- `Split-Path -LeafBase` — filename without extension (PS6+)
- `Split-Path -Qualifier` — drive (`C:`)
- `Split-Path -NoQualifier` — strip drive
- `Split-Path -IsAbsolute` — return `$true`/`$false`
- `Test-Path -PathType Container` — directory only
- `Test-Path -PathType Leaf` — file only

## Examples

### Join-Path

Always use `Join-Path` instead of concatenating with `\` — it picks the right separator per OS and handles trailing slashes:

```powershell
Join-Path 'C:\logs' '2026-05-14.txt'
# C:\logs\2026-05-14.txt

Join-Path 'C:\logs\' '\2026-05-14.txt'
# C:\logs\2026-05-14.txt    (extra slashes normalized)
```

PowerShell 7 accepts multiple child segments:

```powershell
Join-Path 'C:\logs' '2026' '05' '14.txt'
# C:\logs\2026\05\14.txt
```

For PS5.1, pipe instead:

```powershell
'C:\logs' | Join-Path -ChildPath '2026' | Join-Path -ChildPath '05' | Join-Path -ChildPath '14.txt'
```

### Split-Path

```powershell
$p = 'C:\Users\louis\report.txt'

Split-Path $p                       # C:\Users\louis     (parent, default)
Split-Path $p -Parent               # C:\Users\louis
Split-Path $p -Leaf                 # report.txt
Split-Path $p -Extension            # .txt
Split-Path $p -LeafBase             # report
Split-Path $p -Qualifier            # C:
Split-Path $p -NoQualifier          # \Users\louis\report.txt
Split-Path $p -IsAbsolute           # True
```

Walk up multiple levels by chaining:

```powershell
$grandparent = Split-Path (Split-Path 'C:\a\b\c\file.txt' -Parent) -Parent
# C:\a
```

### Resolve-Path

Canonicalize a path and expand wildcards. The path must exist:

```powershell
Resolve-Path '.\src\..\README.md'
# C:\Users\louis\project\README.md

Resolve-Path 'C:\Users\*\Desktop'
# C:\Users\Public\Desktop
# C:\Users\louis\Desktop
```

Use `-Relative` to express the canonicalized path relative to the current location:

```powershell
Set-Location C:\projects\app
Resolve-Path C:\projects\app\src\index.ts -Relative
# .\src\index.ts
```

To probe without erroring on non-existent paths:

```powershell
$resolved = Resolve-Path 'maybe.txt' -ErrorAction SilentlyContinue
if (-not $resolved) { 'not found' }
```

### Convert-Path

Like `Resolve-Path` but always returns a plain filesystem path, stripping PowerShell provider syntax:

```powershell
Set-Location HKCU:\Software
(Resolve-Path .).Path        # Microsoft.PowerShell.Core\Registry::HKEY_CURRENT_USER\Software
Convert-Path .               # HKEY_CURRENT_USER\Software
```

When you're writing scripts that hand paths to external tools, `Convert-Path` is the safer choice.

### Test-Path

```powershell
Test-Path 'C:\Windows'                              # True
Test-Path 'C:\Windows' -PathType Container          # True (it's a directory)
Test-Path 'C:\Windows' -PathType Leaf               # False
Test-Path 'C:\Windows\notepad.exe' -PathType Leaf   # True

Test-Path -Path Env:USERNAME                        # works against any provider
Test-Path HKCU:\Software\Microsoft                  # registry path
```

Guard pattern for "create if missing":

```powershell
$dir = 'C:\logs\app'
if (-not (Test-Path -Path $dir -PathType Container)) {
    New-Item -ItemType Directory -Path $dir | Out-Null
}
```

### Script-Location Variables

Inside a `.ps1`:

```powershell
$PSScriptRoot                       # directory of THIS script
$PSCommandPath                      # full path of THIS script
$MyInvocation.MyCommand.Name        # script filename
```

Anchor sibling resources relative to the script, not the current directory:

```powershell
$configPath = Join-Path $PSScriptRoot 'config.json'
$config     = Get-Content $configPath -Raw | ConvertFrom-Json
```

Without `$PSScriptRoot`, the script breaks when invoked from a different working directory.

Inside a **function** (not a script), `$PSScriptRoot` resolves to the directory of the file that defined the function — useful for modules.

### Common Patterns

**Change extension:**

```powershell
$p = 'C:\data\report.txt'
$new = Join-Path (Split-Path $p -Parent) ((Split-Path $p -LeafBase) + '.log')
# C:\data\report.log
```

**Make a backup name with timestamp:**

```powershell
$src = 'C:\data\report.txt'
$dir = Split-Path $src -Parent
$base = Split-Path $src -LeafBase
$ext  = Split-Path $src -Extension
$stamp = (Get-Date).ToString('yyyyMMdd-HHmmss')
Join-Path $dir "$base.$stamp$ext"
# C:\data\report.20260514-141530.txt
```

**Get current location as a literal path:**

```powershell
$here = (Get-Location).Path         # provider-style
$here = (Convert-Path .)            # literal filesystem path
```

### UNC And Long Paths

UNC paths work transparently:

```powershell
Test-Path '\\server\share\folder'
Get-ChildItem '\\server\share\folder'
```

On Windows, paths over 260 characters require either the `\\?\` prefix or the LongPathsEnabled registry setting (PS7 enables long-path support by default). Inside `\\?\` paths, separators must be `\` and no normalization happens.

### Relative Vs Absolute

```powershell
[System.IO.Path]::IsPathRooted('C:\foo')        # True
[System.IO.Path]::IsPathRooted('foo\bar')       # False
[System.IO.Path]::IsPathRooted('/usr/local')    # True

# PowerShell-native version:
Split-Path 'foo\bar' -IsAbsolute                # False
```

To turn a relative path into absolute without requiring it to exist:

```powershell
[System.IO.Path]::GetFullPath('..\sibling.txt')
```

## Related

- [`PowerShell File And Text Recipes`](/powershell/filesystem/file-and-text-recipes/)
- [`PowerShell Environment Variables`](/powershell/syntax/environment-variables/)
- [`PowerShell Registry Operations`](/powershell/filesystem/registry-operations/)
