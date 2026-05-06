---
title: PowerShell Useful Query Patterns
slug: /powershell/querying/useful-query-patterns/
summary: Practical pipeline patterns for filtering, selecting, sorting, grouping, formatting, and exporting PowerShell objects, plus reusable templates for services, processes, files, network connections, and installed packages.
topic: powershell/querying
type: reference
tags: [powershell, pipeline, where-object, select-object, sort-object, group-object, format-table, export-csv, calculated-properties]
aliases: [powershell pipeline reference, where-object cheatsheet, select-object expand property, calculated property template, top processes by memory, listening ports powershell, group-object by property, useful query patterns]
platforms: [windows, powershell]
related:
  - /powershell/querying/system-inspection-patterns/
  - /powershell/syntax/cmdlet-patterns-and-filtering/
  - /powershell/querying/service-and-system-admin/
status: published
updated: 2026-05-06
---

## Synopsis

A practical reference for querying objects in PowerShell using common pipeline patterns. Covers the pipeline operator, object inspection, `Where-Object` filtering, comparison operators, wildcard and regex matching, property selection and expansion, sorting, grouping, output formatting, CSV/JSON export, calculated properties, multi-condition logic, top-N queries, and reusable templates for services, processes, files, network connections, and installed packages.

## Syntax

```powershell
Get-Service | Select-Object Name, Status
Get-Service | Where-Object { $_.Status -eq 'Running' }
Get-Process | Sort-Object CPU -Descending | Select-Object -First 10
Get-Service | Group-Object Status
Command | Where-Object Property -like 'pattern' | Select-Object Property1, Property2 | Sort-Object Property1
```

## Parameters/Flags

- `|`: pipeline operator that sends output of one command into the next
- `$_`: the current object in the pipeline
- `Get-Member` / `gm`: inspect the properties and methods of objects
- `Where-Object`: filter objects by a condition (long form `{ ... }`, short form `Property -op Value`)
- `Select-Object`: choose properties, take `-First` / `-Last` / `-Skip`, or use `-ExpandProperty` to flatten
- `Sort-Object`: order results, optionally `-Descending`
- `Group-Object`: bucket objects by property value with counts
- `Format-Table` / `Format-List` / `Format-Wide`: presentation cmdlets (always last in a pipeline)
- `Export-Csv`: serialize objects to CSV (use `-NoTypeInformation`)
- `ConvertTo-Json`: serialize objects to JSON (use `-Depth` for nested data)
- Comparison operators: `-eq`, `-ne`, `-gt`, `-ge`, `-lt`, `-le`, `-like`, `-notlike`, `-match`, `-notmatch`, `-in`, `-notin`, `-contains`, `-notcontains`
- Logical operators: `-and`, `-or`, `-not`

## Examples

### The Pipeline

The pipeline operator `|` sends the output of one command into the next.

```powershell
Get-Service | Select-Object Name, Status
```

Steps:

1. Get all services.
2. Send them to `Select-Object`.
3. Display only the `Name` and `Status` properties.

Multi-line formatting is often easier to read:

```powershell
Get-Service |
    Select-Object Name, Status
```

### Inspect Object Properties

Before filtering or selecting, inspect what properties are available.

```powershell
Get-Service | Get-Member
```

Shorter alias:

```powershell
Get-Service | gm
```

Example with processes:

```powershell
Get-Process | Get-Member
```

To see all properties for one object:

```powershell
Get-Service |
    Select-Object -First 1 *
```

Or format the first object as a list:

```powershell
Get-Service |
    Select-Object -First 1 |
    Format-List *
```

### Filtering with Where-Object

Use `Where-Object` to keep only objects that match a condition.

Long form:

```powershell
Get-Service |
    Where-Object { $_.Status -eq 'Running' }
```

Short form:

```powershell
Get-Service |
    Where-Object Status -eq 'Running'
```

`$_` means "the current object in the pipeline."

### Common Comparison Operators

| Operator | Meaning |
| --- | --- |
| `-eq` | Equals |
| `-ne` | Not equals |
| `-gt` | Greater than |
| `-ge` | Greater than or equal |
| `-lt` | Less than |
| `-le` | Less than or equal |
| `-like` | Wildcard match |
| `-notlike` | Wildcard does not match |
| `-match` | Regex match |
| `-notmatch` | Regex does not match |
| `-in` | Value is in a collection |
| `-notin` | Value is not in a collection |
| `-contains` | Collection contains value |
| `-notcontains` | Collection does not contain value |

Example:

```powershell
Get-Process |
    Where-Object CPU -gt 10
```

### Wildcard Filtering with -like

Use `-like` when you want simple wildcard matching.

| Pattern | Meaning |
| --- | --- |
| `abc*` | Starts with `abc` |
| `*abc` | Ends with `abc` |
| `*abc*` | Contains `abc` |
| `a?c` | Matches `abc`, `axc`, etc. |

Services starting with `Win`:

```powershell
Get-Service |
    Where-Object Name -like 'Win*'
```

Packages with IDs starting with `MSIX/`:

```powershell
Get-WingetPackage |
    Where-Object Id -like 'MSIX/*'
```

Processes containing `edge`:

```powershell
Get-Process |
    Where-Object ProcessName -like '*edge*'
```

### Regex Filtering with -match

Use `-match` for regular expressions.

Names starting with `Win`:

```powershell
Get-Service |
    Where-Object Name -match '^Win'
```

Names ending with `Svc`:

```powershell
Get-Service |
    Where-Object Name -match 'Svc$'
```

Package IDs starting with `MSIX/`:

```powershell
Get-WingetPackage |
    Where-Object Id -match '^MSIX/'
```

Regex anchors:

| Regex | Meaning |
| --- | --- |
| `^` | Start of string |
| `$` | End of string |
| `.` | Any single character |
| `.*` | Any number of characters |
| `\d` | Digit |
| `\w` | Word character |

### Selecting Properties with Select-Object

Use `Select-Object` to choose which properties to display.

```powershell
Get-Service |
    Select-Object Name, DisplayName, Status
```

Process selection:

```powershell
Get-Process |
    Select-Object Name, Id, CPU, WorkingSet64
```

Select the first 10 objects:

```powershell
Get-Process |
    Select-Object -First 10
```

Select the last 10 objects:

```powershell
Get-Process |
    Select-Object -Last 10
```

Skip the first 5 objects:

```powershell
Get-Process |
    Select-Object -Skip 5
```

### Expanding a Single Property

By default, `Select-Object Name` returns objects with a `Name` property.

```powershell
Get-Service |
    Select-Object Name
```

Output shape:

```text
Name
----
AarSvc
AdobeARMservice
AppIDSvc
```

For raw string values, use `-ExpandProperty`:

```powershell
Get-Service |
    Select-Object -ExpandProperty Name
```

Output shape:

```text
AarSvc
AdobeARMservice
AppIDSvc
```

This is useful when passing property values to another command:

```powershell
Get-WingetPackage |
    Where-Object Id -like 'MSIX/*' |
    Select-Object -ExpandProperty Id
```

### Sorting with Sort-Object

Sort objects by a property:

```powershell
Get-Service |
    Sort-Object Name
```

Sort descending:

```powershell
Get-Process |
    Sort-Object CPU -Descending
```

Top 10 processes by CPU:

```powershell
Get-Process |
    Sort-Object CPU -Descending |
    Select-Object -First 10 Name, Id, CPU
```

Sort by multiple properties:

```powershell
Get-Service |
    Sort-Object Status, Name
```

### Grouping with Group-Object

Group services by status:

```powershell
Get-Service |
    Group-Object Status
```

Example output:

```text
Count Name                      Group
----- ----                      -----
  142 Running                   {...}
  120 Stopped                   {...}
```

Group processes by company:

```powershell
Get-Process |
    Group-Object Company
```

Group installed packages by source:

```powershell
Get-WingetPackage |
    Group-Object Source
```

Sort groups by count:

```powershell
Get-Service |
    Group-Object Status |
    Sort-Object Count -Descending
```

### Formatting Output

PowerShell has formatting commands such as:

| Command | Purpose |
| --- | --- |
| `Format-Table` | Display as a table |
| `Format-List` | Display as a list |
| `Format-Wide` | Display wide columns |

Format as table:

```powershell
Get-Service |
    Select-Object Name, Status |
    Format-Table -AutoSize
```

Format as list:

```powershell
Get-Service |
    Select-Object -First 5 Name, DisplayName, Status |
    Format-List
```

Formatting commands should usually be last. Good:

```powershell
Get-Service |
    Where-Object Status -eq 'Running' |
    Select-Object Name, Status |
    Format-Table -AutoSize
```

Avoid this:

```powershell
Get-Service |
    Format-Table |
    Where-Object Status -eq 'Running'
```

Once data is formatted, it is no longer as useful for further object-based processing.

### Exporting to CSV

Save object data with `Export-Csv`:

```powershell
Get-Service |
    Select-Object Name, DisplayName, Status |
    Export-Csv "$env:USERPROFILE\Desktop\Services.csv" -NoTypeInformation
```

Export running services:

```powershell
Get-Service |
    Where-Object Status -eq 'Running' |
    Select-Object Name, DisplayName, Status |
    Export-Csv "$env:USERPROFILE\Desktop\RunningServices.csv" -NoTypeInformation
```

Open the CSV afterward:

```powershell
Invoke-Item "$env:USERPROFILE\Desktop\RunningServices.csv"
```

### Exporting to JSON

Convert to JSON for structured data:

```powershell
Get-Service |
    Select-Object Name, DisplayName, Status |
    ConvertTo-Json
```

Save JSON to a file:

```powershell
Get-Service |
    Select-Object Name, DisplayName, Status |
    ConvertTo-Json |
    Set-Content "$env:USERPROFILE\Desktop\Services.json"
```

For nested objects, increase depth:

```powershell
Get-Process |
    Select-Object -First 5 * |
    ConvertTo-Json -Depth 5
```

### Calculated Properties

Calculated properties create custom output columns.

Show process memory in MB:

```powershell
Get-Process |
    Select-Object Name, Id,
        @{
            Name = 'MemoryMB'
            Expression = { [math]::Round($_.WorkingSet64 / 1MB, 2) }
        }
```

Disk size and free space in GB:

```powershell
Get-CimInstance Win32_LogicalDisk |
    Where-Object DriveType -eq 3 |
    Select-Object DeviceID, VolumeName,
        @{
            Name = 'SizeGB'
            Expression = { [math]::Round($_.Size / 1GB, 2) }
        },
        @{
            Name = 'FreeGB'
            Expression = { [math]::Round($_.FreeSpace / 1GB, 2) }
        }
```

Structure:

```powershell
@{
    Name = 'NewColumnName'
    Expression = { value calculation here }
}
```

### Combining Multiple Conditions

Use `-and`, `-or`, and `-not`.

Running services with names starting with `Win`:

```powershell
Get-Service |
    Where-Object {
        $_.Status -eq 'Running' -and
        $_.Name -like 'Win*'
    }
```

Stopped services whose names contain `update`:

```powershell
Get-Service |
    Where-Object {
        $_.Status -eq 'Stopped' -and
        $_.Name -like '*update*'
    }
```

Processes using high memory or high CPU:

```powershell
Get-Process |
    Where-Object {
        $_.WorkingSet64 -gt 500MB -or
        $_.CPU -gt 100
    }
```

Negation:

```powershell
Get-Service |
    Where-Object {
        $_.Status -ne 'Running'
    }
```

Or:

```powershell
Get-Service |
    Where-Object {
        -not ($_.Status -eq 'Running')
    }
```

### Finding Top Results

Top 10 processes by memory:

```powershell
Get-Process |
    Sort-Object WorkingSet64 -Descending |
    Select-Object -First 10 Name, Id,
        @{
            Name = 'MemoryMB'
            Expression = { [math]::Round($_.WorkingSet64 / 1MB, 2) }
        }
```

Top 20 largest files in a folder:

```powershell
Get-ChildItem "C:\Temp" -File -Recurse |
    Sort-Object Length -Descending |
    Select-Object -First 20 FullName,
        @{
            Name = 'SizeMB'
            Expression = { [math]::Round($_.Length / 1MB, 2) }
        }
```

### Searching Files

Find files by extension:

```powershell
Get-ChildItem "C:\Temp" -Filter *.log -Recurse
```

Find large files:

```powershell
Get-ChildItem "C:\Temp" -File -Recurse |
    Where-Object Length -gt 100MB
```

Find recently modified files:

```powershell
Get-ChildItem "C:\Temp" -File -Recurse |
    Where-Object LastWriteTime -gt (Get-Date).AddDays(-7)
```

Find files containing text:

```powershell
Select-String -Path "C:\Temp\*.log" -Pattern "error"
```

Recursive text search:

```powershell
Get-ChildItem "C:\Temp" -Filter *.log -Recurse |
    Select-String -Pattern "error"
```

### Querying Services

All services:

```powershell
Get-Service
```

Running services:

```powershell
Get-Service |
    Where-Object Status -eq 'Running'
```

Stopped services:

```powershell
Get-Service |
    Where-Object Status -eq 'Stopped'
```

Services matching a name:

```powershell
Get-Service |
    Where-Object Name -like '*update*'
```

Services sorted by status then name:

```powershell
Get-Service |
    Sort-Object Status, Name |
    Select-Object Name, DisplayName, Status
```

Services with startup type:

```powershell
Get-CimInstance Win32_Service |
    Select-Object Name, DisplayName, State, StartMode
```

Automatic services that are not running:

```powershell
Get-CimInstance Win32_Service |
    Where-Object {
        $_.StartMode -eq 'Auto' -and
        $_.State -ne 'Running'
    } |
    Select-Object Name, DisplayName, State, StartMode
```

### Querying Processes

All processes:

```powershell
Get-Process
```

Processes matching name:

```powershell
Get-Process |
    Where-Object ProcessName -like '*edge*'
```

High CPU processes:

```powershell
Get-Process |
    Where-Object CPU -gt 10 |
    Sort-Object CPU -Descending |
    Select-Object Name, Id, CPU
```

High memory processes:

```powershell
Get-Process |
    Sort-Object WorkingSet64 -Descending |
    Select-Object -First 15 Name, Id,
        @{
            Name = 'MemoryMB'
            Expression = { [math]::Round($_.WorkingSet64 / 1MB, 2) }
        }
```

### Querying Network Connections

Established TCP connections:

```powershell
Get-NetTCPConnection |
    Where-Object State -eq 'Established'
```

Listening ports:

```powershell
Get-NetTCPConnection |
    Where-Object State -eq 'Listen' |
    Select-Object LocalAddress, LocalPort, OwningProcess
```

Listening ports with process name:

```powershell
Get-NetTCPConnection |
    Where-Object State -eq 'Listen' |
    Select-Object LocalAddress, LocalPort, OwningProcess,
        @{
            Name = 'ProcessName'
            Expression = {
                (Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue).Name
            }
        }
```

Connections to a specific remote port:

```powershell
Get-NetTCPConnection |
    Where-Object RemotePort -eq 443
```

### Querying Installed Packages

Winget packages:

```powershell
Get-WingetPackage
```

Packages with IDs starting with `MSIX/`:

```powershell
Get-WingetPackage |
    Where-Object Id -like 'MSIX/*'
```

Show selected package properties:

```powershell
Get-WingetPackage |
    Select-Object Name, Id, Version, Source
```

Sort by package ID:

```powershell
Get-WingetPackage |
    Select-Object Name, Id, Version, Source |
    Sort-Object Id
```

Group by source:

```powershell
Get-WingetPackage |
    Group-Object Source
```

Export package list:

```powershell
Get-WingetPackage |
    Select-Object Name, Id, Version, Source |
    Export-Csv "$env:USERPROFILE\Desktop\WingetPackages.csv" -NoTypeInformation
```

### The Most Reusable Pattern

Use this as a template:

```powershell
Command |
    Where-Object Property -like 'pattern' |
    Select-Object Property1, Property2 |
    Sort-Object Property1
```

Example:

```powershell
Get-WingetPackage |
    Where-Object Id -like 'MSIX/*' |
    Select-Object Name, Id, Version |
    Sort-Object Id
```

Another example:

```powershell
Get-Service |
    Where-Object Name -like 'Win*' |
    Select-Object Name, DisplayName, Status |
    Sort-Object Name
```

### Quick Reference Cheatsheet

Filter:

```powershell
Command |
    Where-Object Property -eq 'Value'
```

Wildcard filter:

```powershell
Command |
    Where-Object Property -like '*text*'
```

Regex filter:

```powershell
Command |
    Where-Object Property -match '^text'
```

Select columns:

```powershell
Command |
    Select-Object Property1, Property2
```

Expand one property:

```powershell
Command |
    Select-Object -ExpandProperty Property
```

Sort ascending:

```powershell
Command |
    Sort-Object Property
```

Sort descending:

```powershell
Command |
    Sort-Object Property -Descending
```

Take first results:

```powershell
Command |
    Select-Object -First 10
```

Group and count:

```powershell
Command |
    Group-Object Property
```

Export CSV:

```powershell
Command |
    Export-Csv "C:\Path\File.csv" -NoTypeInformation
```

Format table:

```powershell
Command |
    Format-Table -AutoSize
```

### Practical Troubleshooting Tips

If a property does not work, inspect the object:

```powershell
Command |
    Get-Member
```

If output is cut off, use `Format-List`:

```powershell
Command |
    Select-Object -First 1 |
    Format-List *
```

If you only need values, use `-ExpandProperty`:

```powershell
Command |
    Select-Object -ExpandProperty Name
```

Put formatting commands last. Good:

```powershell
Get-Service |
    Where-Object Status -eq 'Running' |
    Format-Table -AutoSize
```

Avoid:

```powershell
Get-Service |
    Format-Table -AutoSize |
    Where-Object Status -eq 'Running'
```

Prefer objects over text. PowerShell is strongest when you keep data as objects until the final output step. Good:

```powershell
Get-Service |
    Where-Object Status -eq 'Running' |
    Select-Object Name, Status
```

Less ideal:

```powershell
Get-Service |
    Out-String |
    Select-String "Running"
```

### Suggested Practice Commands

Try these and modify them:

```powershell
Get-Service |
    Where-Object Status -eq 'Running' |
    Select-Object Name, Status |
    Sort-Object Name
```

```powershell
Get-Process |
    Sort-Object CPU -Descending |
    Select-Object -First 10 Name, Id, CPU
```

```powershell
Get-NetTCPConnection |
    Where-Object State -eq 'Listen' |
    Select-Object LocalAddress, LocalPort, OwningProcess
```

```powershell
Get-WingetPackage |
    Where-Object Id -like 'MSIX/*' |
    Select-Object Name, Id, Version
```

```powershell
Get-ChildItem "$env:USERPROFILE\Downloads" -File |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 20 Name, LastWriteTime, Length
```

### Summary

Most PowerShell queries are built from a few core commands:

| Command | Purpose |
| --- | --- |
| `Where-Object` | Filter objects |
| `Select-Object` | Choose properties |
| `Sort-Object` | Sort results |
| `Group-Object` | Group and count |
| `Format-Table` | Display table output |
| `Format-List` | Display detailed output |
| `Export-Csv` | Save structured results |

Core pattern:

```powershell
Get-Something |
    Where-Object SomeProperty -like 'SomePattern' |
    Select-Object Property1, Property2 |
    Sort-Object Property1
```

## Related

- [`PowerShell System Inspection Patterns`](/powershell/querying/system-inspection-patterns/)
- [`PowerShell Cmdlet Patterns And Filtering`](/powershell/syntax/cmdlet-patterns-and-filtering/)
- [`PowerShell Service And System Administration`](/powershell/querying/service-and-system-admin/)
