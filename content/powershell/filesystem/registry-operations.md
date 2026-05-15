---
title: PowerShell Registry Operations
slug: /powershell/filesystem/registry-operations/
summary: Reference for reading, writing, and deleting Windows registry keys and values through PowerShell's Registry PSDrive using Get-Item, Get-ItemProperty, New-Item, New-ItemProperty, and Set-ItemProperty.
topic: powershell/filesystem
type: reference
tags: [powershell, registry, hkcu, hklm, get-itemproperty, set-itemproperty, psdrive]
aliases: [powershell read registry, powershell write registry, powershell hkcu hklm, powershell new-itemproperty, powershell dword value, powershell registry psdrive]
platforms: [windows, powershell]
related:
  - /powershell/syntax/environment-variables/
  - /powershell/querying/system-inspection-patterns/
  - /powershell/filesystem/path-manipulation/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell exposes the Windows registry as a PSDrive, so the same `Get-Item` / `New-Item` / `Remove-Item` cmdlets work — but with a twist: **keys are items, values are properties of items**. That distinction (item vs. itemproperty) is the single thing to internalize.

## Syntax

```powershell
Get-Item         <key-path>                                    # the key itself
Get-ItemProperty <key-path> [-Name <value-name>]               # values under that key
New-Item         <key-path> -Force                             # create key
New-ItemProperty <key-path> -Name <name> -Value <v> -PropertyType <type>
Set-ItemProperty <key-path> -Name <name> -Value <v>
Remove-ItemProperty <key-path> -Name <name>
Remove-Item      <key-path> -Recurse                           # delete key (and children)
```

## Parameters/Flags

Built-in registry PSDrives:
- `HKCU:` → `HKEY_CURRENT_USER`
- `HKLM:` → `HKEY_LOCAL_MACHINE`
- `HKCR:`, `HKU:`, `HKCC:` available via `New-PSDrive` or explicit `Registry::` prefix

`-PropertyType` for `New-ItemProperty`:
- `String` (REG_SZ) — most common
- `ExpandString` (REG_EXPAND_SZ) — string with `%VAR%` expansion
- `Binary` (REG_BINARY) — `byte[]`
- `DWord` (REG_DWORD) — 32-bit int
- `QWord` (REG_QWORD) — 64-bit int
- `MultiString` (REG_MULTI_SZ) — `string[]`

HKLM writes require **administrator** privileges. HKCU is per-user, no elevation needed.

## Examples

### Read A Single Value

```powershell
Get-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer' `
                 -Name 'Shell Icon Size'
```

This returns an object whose property *is* the value name. Access it directly:

```powershell
(Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer').'Shell Icon Size'
```

### List All Values Under A Key

```powershell
Get-ItemProperty 'HKLM:\Software\Microsoft\Windows NT\CurrentVersion'
```

The output is a single object with one property per value. To get a clean name/value list:

```powershell
$key = 'HKLM:\Software\Microsoft\Windows NT\CurrentVersion'
(Get-Item $key).GetValueNames() | ForEach-Object {
    [PSCustomObject]@{
        Name  = $_
        Value = (Get-ItemProperty $key -Name $_).$_
        Type  = (Get-Item $key).GetValueKind($_)
    }
}
```

### List Subkeys

`Get-ChildItem` walks subkeys (the registry equivalent of subdirectories):

```powershell
Get-ChildItem 'HKLM:\Software'
Get-ChildItem 'HKLM:\Software' -Recurse -Depth 1
```

### Create A Key

```powershell
New-Item -Path 'HKCU:\Software\MyApp' -Force
```

`-Force` makes it idempotent — no error if it already exists. To create a nested path in one call:

```powershell
New-Item -Path 'HKCU:\Software\MyApp\Settings\v1' -Force
```

`New-Item` creates intermediate keys automatically.

### Write A Value

```powershell
# DWord (the most common type for boolean-style flags)
New-ItemProperty -Path 'HKCU:\Software\MyApp' `
                 -Name  'EnableLogging' `
                 -Value 1 `
                 -PropertyType DWord `
                 -Force

# String
New-ItemProperty -Path 'HKCU:\Software\MyApp' `
                 -Name  'LogFile' `
                 -Value 'C:\logs\myapp.log' `
                 -PropertyType String `
                 -Force

# MultiString (REG_MULTI_SZ)
New-ItemProperty -Path 'HKCU:\Software\MyApp' `
                 -Name  'WatchedFolders' `
                 -Value @('C:\a', 'C:\b', 'C:\c') `
                 -PropertyType MultiString `
                 -Force
```

`-Force` overwrites an existing value of any type. Without it, `New-ItemProperty` fails if the value already exists.

### Update An Existing Value

`Set-ItemProperty` preserves the existing type:

```powershell
Set-ItemProperty -Path 'HKCU:\Software\MyApp' -Name 'EnableLogging' -Value 0
```

If the value doesn't exist yet, `Set-ItemProperty` creates it as a `String` — which is rarely what you want for numeric data. Use `New-ItemProperty -Force` for create-or-update on typed values.

### Delete A Value

```powershell
Remove-ItemProperty -Path 'HKCU:\Software\MyApp' -Name 'EnableLogging'
```

### Delete A Key

```powershell
Remove-Item -Path 'HKCU:\Software\MyApp' -Recurse
```

Always use `-Recurse` if the key has subkeys — without it, the command fails.

### Test For Key Or Value

```powershell
Test-Path 'HKCU:\Software\MyApp'                              # does the key exist?

# Does a specific value exist? Test-Path doesn't drill into properties:
$exists = $null -ne (Get-ItemProperty 'HKCU:\Software\MyApp' `
                       -Name 'EnableLogging' -ErrorAction SilentlyContinue)
```

A cleaner property-existence check:

```powershell
function Test-RegistryValue {
    param([string]$Path, [string]$Name)
    try {
        $null = Get-ItemProperty -Path $Path -Name $Name -ErrorAction Stop
        $true
    } catch { $false }
}
```

### Get The Type Of A Value

```powershell
(Get-Item 'HKCU:\Software\MyApp').GetValueKind('EnableLogging')
# DWord
```

### Other Hives (HKCR, HKU)

`HKCR:` and `HKU:` aren't auto-mounted. Use the `Registry::` provider prefix:

```powershell
Get-Item 'Registry::HKEY_CLASSES_ROOT\.txt'
Get-ChildItem 'Registry::HKEY_USERS'
```

Or mount a drive:

```powershell
New-PSDrive -Name HKCR -PSProvider Registry -Root HKEY_CLASSES_ROOT
Get-ChildItem HKCR:\.txt
```

### Importing And Exporting .reg Files

PowerShell has no native cmdlet for `.reg` files; shell out to `reg.exe`:

```powershell
# Export
reg.exe export 'HKCU\Software\MyApp' 'C:\backup\myapp.reg' /y

# Import
reg.exe import 'C:\backup\myapp.reg'
```

### Common Pitfalls

- **Item vs. ItemProperty.** `Get-Item` returns the key. `Get-ItemProperty` returns the *values* under the key. They're not interchangeable.
- **Default value.** The unnamed "(Default)" value is accessed with `-Name '(Default)'`.
- **64-bit vs. 32-bit views.** A 32-bit PowerShell process sees `HKLM:\Software\WOW6432Node\...` under `HKLM:\Software\...` due to registry reflection. To force a view, use the [.NET `RegistryKey.OpenBaseKey`](https://learn.microsoft.com/en-us/dotnet/api/microsoft.win32.registrykey.openbasekey) APIs.
- **HKLM writes require elevation.** Without an elevated session, `New-ItemProperty` under `HKLM:` errors with access denied.

## Related

- [`PowerShell Environment Variables`](/powershell/syntax/environment-variables/)
- [`PowerShell System Inspection Patterns`](/powershell/querying/system-inspection-patterns/)
- [`PowerShell Path Manipulation`](/powershell/filesystem/path-manipulation/)
