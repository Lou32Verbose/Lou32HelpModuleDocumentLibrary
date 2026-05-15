---
title: PowerShell Modules And PSGallery
slug: /powershell/syntax/modules-and-psgallery/
summary: Reference for finding, installing, importing, updating, and authoring PowerShell modules from PSGallery and local sources, including PSResourceGet, manifest files, and PSModulePath.
topic: powershell/syntax
type: reference
tags: [powershell, modules, psgallery, install-module, import-module, psresourceget, psmodulepath]
aliases: [powershell install module, powershell psgallery, powershell import-module, powershell module manifest, powershell psresourceget, powershell psmodulepath]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/profiles/console-and-profile-customization/
  - /powershell/syntax/environment-variables/
  - /powershell/syntax/functions-and-parameters/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell modules are reusable bundles of functions, cmdlets, variables, and other resources. PSGallery is the public registry. The older `PowerShellGet` (`Install-Module`) is being replaced by `Microsoft.PowerShell.PSResourceGet` (`Install-PSResource`) — both are supported in PowerShell 7, and you'll see scripts using either.

## Syntax

```powershell
Find-Module    <name>                          # search PSGallery
Install-Module <name> -Scope CurrentUser       # install
Import-Module  <name>                          # load into current session
Update-Module  <name>                          # upgrade installed copy
Uninstall-Module <name>                        # remove
Get-Module     -ListAvailable                  # what's installed
Get-Module                                     # what's loaded

# PSResourceGet (newer, faster, recommended in PS7+):
Find-PSResource    <name>
Install-PSResource <name> -Scope CurrentUser
```

## Parameters/Flags

- `-Scope CurrentUser` — install to `~/Documents/PowerShell/Modules` (no admin needed)
- `-Scope AllUsers` — install to `$env:ProgramFiles\PowerShell\Modules` (admin required)
- `-Force` — overwrite existing version
- `-AllowPrerelease` — include prerelease versions
- `-RequiredVersion` / `-MinimumVersion` / `-MaximumVersion` — pin or constrain
- `-Repository` — install from a non-default repo (e.g., private feed)

## Examples

### Find And Install From PSGallery

```powershell
Find-Module -Name PSScriptAnalyzer
Install-Module -Name PSScriptAnalyzer -Scope CurrentUser
```

The first time you hit PSGallery, you'll see a trust prompt. Suppress it by trusting the repo:

```powershell
Set-PSRepository -Name PSGallery -InstallationPolicy Trusted
```

(With PSResourceGet, the equivalent is `Set-PSResourceRepository -Name PSGallery -Trusted`.)

### Searching

```powershell
Find-Module -Name *azure*
Find-Module -Tag aws -Repository PSGallery
Find-Module -Name PSReadLine -AllVersions
```

### Loading A Module

Most modules auto-load when you call one of their commands, but you can load explicitly:

```powershell
Import-Module PSScriptAnalyzer
Get-Command -Module PSScriptAnalyzer            # what did it bring in?
```

Force a reload after editing module source:

```powershell
Import-Module ./MyModule -Force
```

### Updating And Removing

```powershell
Update-Module -Name PSScriptAnalyzer
Uninstall-Module -Name PSScriptAnalyzer

# Keep only the latest version, remove old ones:
$name = 'PSScriptAnalyzer'
Get-InstalledModule $name -AllVersions |
    Sort-Object Version -Descending |
    Select-Object -Skip 1 |
    Uninstall-Module
```

### Pinning A Version

```powershell
Install-Module -Name Pester -RequiredVersion 5.5.0 -Force
Import-Module Pester -RequiredVersion 5.5.0
```

`-Force` here permits installing a version when others are already present.

### Where Modules Live: $env:PSModulePath

PowerShell searches `$env:PSModulePath` for modules in order. Inspect it:

```powershell
$env:PSModulePath -split [IO.Path]::PathSeparator
```

Typical entries on Windows PowerShell 7:
- `$HOME\Documents\PowerShell\Modules` (CurrentUser)
- `$env:ProgramFiles\PowerShell\Modules` (AllUsers)
- `$PSHOME\Modules` (shipped with PowerShell)
- `$env:ProgramFiles\WindowsPowerShell\Modules` (Win PS compat)

Add a custom directory for local development:

```powershell
$env:PSModulePath = "C:\dev\my-modules;$env:PSModulePath"
```

To persist this change, set it in your profile or as a User environment variable (see [`Environment Variables`](/powershell/syntax/environment-variables/)).

### PSResourceGet (The Modern Replacement)

PSResourceGet replaces PowerShellGet with a faster, simpler API:

```powershell
Install-Module -Name Microsoft.PowerShell.PSResourceGet -Scope CurrentUser

Find-PSResource    PSScriptAnalyzer
Install-PSResource PSScriptAnalyzer -Scope CurrentUser
Update-PSResource  PSScriptAnalyzer
Uninstall-PSResource PSScriptAnalyzer
Get-InstalledPSResource
```

It also installs scripts and roles, not just modules:

```powershell
Install-PSResource -Name MyScript -Type Script
```

### Authoring A Quick Script Module

A script module is just a `.psm1` file. The directory name and `.psm1` name must match:

```text
MyModule/
  MyModule.psm1
```

Minimal `MyModule.psm1`:

```powershell
function Get-Square {
    param([int]$n)
    $n * $n
}

Export-ModuleMember -Function Get-Square
```

Load it:

```powershell
Import-Module .\MyModule
Get-Square 7        # 49
```

If `MyModule` is on `$env:PSModulePath`, you don't need the relative path — just `Import-Module MyModule`.

### Adding A Manifest (.psd1)

A manifest makes the module discoverable, versioned, and publishable. Generate one:

```powershell
New-ModuleManifest `
    -Path .\MyModule\MyModule.psd1 `
    -RootModule MyModule.psm1 `
    -ModuleVersion '0.1.0' `
    -Author 'Lou32' `
    -Description 'A small utility module' `
    -FunctionsToExport @('Get-Square') `
    -PowerShellVersion '5.1'
```

When a manifest is present, prefer listing exports there (`FunctionsToExport`, `CmdletsToExport`, `AliasesToExport`, `VariablesToExport`) instead of `Export-ModuleMember` — it's faster for auto-loading.

### Publishing To PSGallery

You need an API key from [https://www.powershellgallery.com/account/apikeys](https://www.powershellgallery.com/account/apikeys):

```powershell
Publish-Module -Path .\MyModule -NuGetApiKey '<key>'

# Or with PSResourceGet:
Publish-PSResource -Path .\MyModule -ApiKey '<key>'
```

### Trusted vs Untrusted Repositories

```powershell
Get-PSRepository
# Name        InstallationPolicy   SourceLocation
# ----        ------------------   --------------
# PSGallery   Untrusted            https://www.powershellgallery.com/api/v2

Set-PSRepository -Name PSGallery -InstallationPolicy Trusted
```

For private feeds (Azure Artifacts, ProGet, etc.):

```powershell
Register-PSRepository -Name MyFeed `
                     -SourceLocation 'https://nuget.example.com/v3/index.json' `
                     -InstallationPolicy Trusted
Install-Module -Name InternalTools -Repository MyFeed -Scope CurrentUser
```

### Listing Commands From A Module

```powershell
Get-Module Pester | Select-Object -ExpandProperty ExportedCommands
Get-Command -Module Pester
```

### Removing A Module From The Session (Without Uninstalling)

```powershell
Remove-Module MyModule
```

This unloads it from the current session; the files on disk are untouched.

## Related

- [`PowerShell Console And Profile Customization`](/powershell/profiles/console-and-profile-customization/)
- [`PowerShell Environment Variables`](/powershell/syntax/environment-variables/)
- [`PowerShell Functions And Parameters`](/powershell/syntax/functions-and-parameters/)
