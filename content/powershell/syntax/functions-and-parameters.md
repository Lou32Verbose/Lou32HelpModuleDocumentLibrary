---
title: PowerShell Functions And Parameters
slug: /powershell/syntax/functions-and-parameters/
summary: Reference for defining functions, declaring typed parameters, attaching validation attributes, building advanced functions with CmdletBinding, parameter sets, and pipeline-aware begin/process/end blocks.
topic: powershell/syntax
type: reference
tags: [powershell, functions, parameters, cmdletbinding, validation, pipeline, advanced-function]
aliases: [powershell param block, powershell cmdletbinding, powershell validateset, powershell mandatory parameter, powershell pipeline input, powershell advanced function]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/syntax/error-handling/
  - /powershell/syntax/output-streams-and-redirection/
  - /powershell/syntax/comparison-and-logical-operators/
status: published
updated: 2026-05-14
---

## Synopsis

A bare PowerShell function is just a named script block. Adding `[CmdletBinding()]` and typed parameters turns it into an **advanced function** that behaves like a real cmdlet — common parameters (`-Verbose`, `-ErrorAction`), parameter validation, pipeline binding, and parameter sets all come for free.

## Syntax

```powershell
function <Verb-Noun> {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$Name,

        [Parameter(ValueFromPipeline)]
        [int]$Count = 1
    )

    begin   { <# one-time setup #> }
    process { <# runs once per pipeline item #> }
    end     { <# one-time cleanup / final output #> }
}
```

## Parameters/Flags

`Parameter` attribute options:
- `Mandatory` — required; PowerShell prompts if missing
- `Position = N` — accepted positionally without naming
- `ValueFromPipeline` — bind whole pipeline object
- `ValueFromPipelineByPropertyName` — bind matching property
- `ParameterSetName = '<name>'` — group into a parameter set
- `HelpMessage = '<text>'` — shown on mandatory prompt

Validation attributes:
- `[ValidateNotNull()]`, `[ValidateNotNullOrEmpty()]`
- `[ValidateSet('a','b','c')]`
- `[ValidateRange(1, 100)]`
- `[ValidateLength(3, 20)]`
- `[ValidatePattern('^\w+$')]`
- `[ValidateScript({ $_ -gt 0 })]`
- `[ValidateCount(1, 5)]`
- `[AllowNull()]`, `[AllowEmptyString()]`, `[AllowEmptyCollection()]`

## Examples

### Minimal Function

```powershell
function Get-Square {
    param([int]$n)
    $n * $n
}

Get-Square 5            # 25
Get-Square -n 5         # 25
```

The last expression's value is the return value. `return` exits early but isn't required to produce output.

### Advanced Function With CmdletBinding

```powershell
function Get-Greeting {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Name,

        [string]$Greeting = 'Hello'
    )

    Write-Verbose "Greeting '$Name' with '$Greeting'"
    "$Greeting, $Name!"
}

Get-Greeting -Name 'Sam' -Verbose
# VERBOSE: Greeting 'Sam' with 'Hello'
# Hello, Sam!
```

`[CmdletBinding()]` adds `-Verbose`, `-Debug`, `-ErrorAction`, `-ErrorVariable`, `-WarningAction`, `-WarningVariable`, `-OutVariable`, `-OutBuffer`, `-PipelineVariable`, `-InformationAction`, `-InformationVariable` automatically.

### Validation Attributes

```powershell
function Set-Volume {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [ValidateRange(0, 100)]
        [int]$Level,

        [ValidateSet('low', 'medium', 'high')]
        [string]$Profile = 'medium',

        [ValidatePattern('^[A-Z]{2,4}$')]
        [string]$Channel,

        [ValidateScript({
            if (Test-Path $_) { $true }
            else { throw "File not found: $_" }
        })]
        [string]$ConfigFile
    )
    # ...
}
```

`[ValidateSet]` also drives tab completion — typing `Set-Volume -Profile <Tab>` cycles through `low`, `medium`, `high`.

### Default Values And Optional Parameters

```powershell
function New-Report {
    param(
        [string]$Title       = 'Untitled',
        [datetime]$Generated = (Get-Date),
        [string[]]$Sections  = @('summary')
    )
    # ...
}
```

Defaults can be any expression, evaluated at call time.

### Parameter Sets

Mutually exclusive parameter combinations:

```powershell
function Get-Thing {
    [CmdletBinding(DefaultParameterSetName = 'ByName')]
    param(
        [Parameter(Mandatory, ParameterSetName = 'ByName')]
        [string]$Name,

        [Parameter(Mandatory, ParameterSetName = 'ById')]
        [int]$Id,

        [Parameter(ParameterSetName = 'ByName')]
        [Parameter(ParameterSetName = 'ById')]
        [switch]$Verbose2
    )

    "Active set: $($PSCmdlet.ParameterSetName)"
}

Get-Thing -Name 'foo'    # ByName
Get-Thing -Id 42         # ById
Get-Thing -Name 'foo' -Id 42   # ERROR: ambiguous
```

### Switch Parameters

`[switch]` parameters are flags — present or absent, no value:

```powershell
function Remove-Cache {
    param([switch]$Force)
    if ($Force) { 'removing without confirmation' }
    else        { 'skipping (use -Force to override)' }
}

Remove-Cache             # skipping
Remove-Cache -Force      # removing
```

To programmatically pass a switch value, splat it: `Remove-Cache @{ Force = $true }` or use `-Force:$bool`.

### Pipeline Input

Two ways a function can receive pipeline objects:

```powershell
function Get-Square {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromPipeline)]
        [int]$Number
    )
    process {
        $Number * $Number
    }
}

1..5 | Get-Square            # 1, 4, 9, 16, 25
```

The `process` block runs once per pipeline element. Without it, the function only sees the *last* piped value.

Bind by property name to consume objects fluently:

```powershell
function Show-FileSize {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromPipelineByPropertyName)]
        [string]$Name,

        [Parameter(ValueFromPipelineByPropertyName)]
        [long]$Length
    )
    process {
        "{0,-30} {1,10:N0}" -f $Name, $Length
    }
}

Get-ChildItem *.log | Show-FileSize
```

### begin / process / end Blocks

```powershell
function Group-Average {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromPipeline)]
        [double]$Value
    )

    begin {
        $sum   = 0
        $count = 0
    }
    process {
        $sum   += $Value
        $count += 1
    }
    end {
        if ($count -gt 0) { $sum / $count } else { $null }
    }
}

1..10 | Group-Average        # 5.5
```

`begin` runs once before pipeline processing, `process` per item, `end` once after all items.

### $PSBoundParameters

A hashtable of parameters explicitly passed by the caller (excludes defaults):

```powershell
function Invoke-Thing {
    [CmdletBinding()]
    param(
        [string]$Name,
        [int]$Count = 10
    )

    "Caller passed: $($PSBoundParameters.Keys -join ', ')"
    if ($PSBoundParameters.ContainsKey('Count')) {
        'count was explicitly set'
    }
}

Invoke-Thing -Name 'foo'
# Caller passed: Name
```

Useful for forwarding parameters to another cmdlet with splatting:

```powershell
function Wrap-Request {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [string]$Uri,
        [hashtable]$Headers,
        [string]$Method = 'GET'
    )

    Invoke-RestMethod @PSBoundParameters
}
```

### Returning Values

PowerShell functions emit anything that isn't assigned, captured, or piped to `Out-Null`:

```powershell
function Get-Two {
    'first'                     # emitted
    'second'                    # emitted
    return 'third'              # emitted, then function exits
    'never'                     # unreachable
}

Get-Two                         # 'first', 'second', 'third' (three items!)
```

To return only one value, suppress everything else:

```powershell
function Get-One {
    $null = Initialize-Stuff    # suppress
    Get-Computation             # the return value
}
```

### SupportsShouldProcess (-WhatIf / -Confirm)

```powershell
function Remove-Old {
    [CmdletBinding(SupportsShouldProcess, ConfirmImpact = 'High')]
    param([string]$Path)

    if ($PSCmdlet.ShouldProcess($Path, 'Delete')) {
        Remove-Item $Path
    }
}

Remove-Old -Path foo.txt -WhatIf
# What if: Performing the operation "Delete" on target "foo.txt".
```

## Related

- [`PowerShell Error Handling`](/powershell/syntax/error-handling/)
- [`PowerShell Output Streams And Redirection`](/powershell/syntax/output-streams-and-redirection/)
- [`PowerShell Comparison And Logical Operators`](/powershell/syntax/comparison-and-logical-operators/)
