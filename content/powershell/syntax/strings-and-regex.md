---
title: PowerShell Strings, Regex, And Format Operators
slug: /powershell/syntax/strings-and-regex/
summary: Reference for string quoting, interpolation, here-strings, regex matching with -match and -replace, capture groups, and the -f format operator.
topic: powershell/syntax
type: reference
tags: [powershell, strings, regex, here-string, format-operator, replace, split, match]
aliases: [powershell here string, powershell -replace regex, powershell capture group, powershell format operator, powershell string interpolation, powershell -f operator]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/syntax/comparison-and-logical-operators/
  - /powershell/syntax/json-and-xml-handling/
  - /powershell/syntax/cmdlet-patterns-and-filtering/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell has two distinct string-processing systems wedged into the same operators: wildcard matching for shell-style globbing (`-like`) and regex matching for everything else (`-match`, `-replace`, `-split`). The `-f` format operator borrows .NET composite formatting and is the right tool any time you need padded, aligned, or numerically formatted output.

## Syntax

```powershell
'literal — no interpolation'
"interpolated — $var and $($expr)"
@"
multi-line here-string
with $variable interpolation
"@
@'
multi-line here-string
NO interpolation
'@

<string> -replace <regex>, <replacement>
<string> -split   <regex>
<string> -match   <regex>
'{0,10:N2}' -f 1234.5
```

## Parameters/Flags

- `"..."` — expandable string; `$var` and `$()` are interpolated.
- `'...'` — literal string; nothing is interpolated.
- `@" ... "@` / `@' ... '@` — here-strings; the closing terminator **must** be at the start of its own line with no leading whitespace.
- Escape character is **backtick** (`` ` ``), not backslash: `` `n `` newline, `` `t `` tab, `` `r `` carriage return, `` `0 `` null, `` `" `` literal quote.
- `-replace` / `-split` / `-match` use .NET regex syntax.
- Default is case-insensitive; prefix with `-c` for case-sensitive (`-creplace`, `-cmatch`, `-csplit`).

## Examples

### Quoting

```powershell
$name = 'world'
"Hello, $name"                    # Hello, world
'Hello, $name'                    # Hello, $name (literal)
"2 + 2 = $(2 + 2)"                # 2 + 2 = 4
"Object: $($proc.Name)"           # use $() for property access
```

Escape a literal `$` inside an expandable string with a backtick:

```powershell
"Price: `$5.00"                   # Price: $5.00
```

### Here-Strings

Useful for multi-line content where you don't want to fight escaping:

```powershell
$body = @"
Dear $name,

Your invoice for $($amount) is attached.
"@
```

Literal here-string (no interpolation), useful for JSON or scripts:

```powershell
$json = @'
{
  "name": "$user",
  "active": true
}
'@
```

### -replace With Regex

```powershell
'2026-05-14' -replace '-', '/'              # 2026/05/14
'  trim me  ' -replace '^\s+|\s+$', ''      # 'trim me'
'phone: 555-1234' -replace '\d', '*'        # phone: ***-****
```

Use capture groups in the replacement with `$1`, `$2`, etc:

```powershell
'John Smith' -replace '^(\w+)\s+(\w+)$', '$2, $1'
# Smith, John
```

A literal `$` in the replacement is `$$`:

```powershell
'100' -replace '^(\d+)$', '$$$1'            # $100
```

### -split With Regex

```powershell
'a,b;c d' -split '[,; ]'                    # 'a', 'b', 'c', 'd'
'one  two   three' -split '\s+'             # 'one', 'two', 'three'
```

Limit the number of splits:

```powershell
'a=b=c=d' -split '=', 2                     # 'a', 'b=c=d'
```

Use a script block for complex splitting:

```powershell
'abc123def456' -split { $_ -match '\d' }
```

### -match And $Matches

`-match` returns a boolean and, as a side effect, populates the automatic `$Matches` hashtable with named or numbered groups:

```powershell
if ('user42@example.com' -match '^(?<user>[^@]+)@(?<domain>.+)$') {
    $Matches.user                # user42
    $Matches.domain              # example.com
}
```

Numbered groups work the same way:

```powershell
'log-2026-05-14.txt' -match '(\d{4})-(\d{2})-(\d{2})'
$Matches[1], $Matches[2], $Matches[3]       # 2026, 05, 14
```

`$Matches` is overwritten by the next `-match` and is empty after a failed match.

### Array -match Behavior

When the left side is an array, `-match` returns the matching elements (like `-eq` does):

```powershell
@('apple','banana','grape') -match 'a.*e'   # apple, grape
```

This is a fast inline filter — but `$Matches` is **not** populated when matching against arrays.

### The [regex] Class

Use the static methods when you need multiple matches, named captures from an array, or compiled patterns:

```powershell
$matches = [regex]::Matches('a1 b22 c333', '\w(\d+)')
foreach ($m in $matches) {
    "{0} -> {1}" -f $m.Value, $m.Groups[1].Value
}
# a1 -> 1
# b22 -> 22
# c333 -> 333
```

Compile a pattern when reusing it:

```powershell
$re = [regex]'^\d{3}-\d{4}$'
$re.IsMatch('555-1234')                     # True
```

Escape a literal string for safe inclusion in a pattern:

```powershell
[regex]::Escape('C:\Program Files\*.exe')
# C:\\Program\ Files\\\*\.exe
```

### The -f Format Operator

Composite formatting from .NET — the right tool for padding, alignment, and number formatting:

```powershell
'{0} = {1}'        -f 'pi', 3.14159         # pi = 3.14159
'{0,10}'           -f 'hi'                  # right-aligned in 10 cols
'{0,-10}|'         -f 'hi'                  # 'hi        |' (left-aligned)
'{0:N2}'           -f 1234.5                # 1,234.50
'{0:P1}'           -f 0.4567                # 45.7%
'{0:X4}'           -f 255                   # 00FF
'{0:yyyy-MM-dd}'   -f (Get-Date)            # 2026-05-14
'{0,5:D3}'         -f 7                     # '  007' (padded then formatted)
```

Common format specifiers:

| Specifier | Meaning                | Example                       |
| --------- | ---------------------- | ----------------------------- |
| `N`       | Number with separators | `{0:N0}` → `1,234`            |
| `C`       | Currency               | `{0:C}` → `$1,234.56`         |
| `P`       | Percent                | `{0:P0}` → `46%`              |
| `X` / `x` | Hex (upper/lower)      | `{0:X}` → `FF`                |
| `D`       | Decimal with padding   | `{0:D5}` → `00042`            |
| `F`       | Fixed-point            | `{0:F3}` → `3.142`            |
| `E`       | Scientific             | `{0:E2}` → `1.23E+003`        |

### Case-Sensitive Variants

```powershell
'ABC' -match    'abc'                       # True (default insensitive)
'ABC' -cmatch   'abc'                       # False
'abc' -creplace 'a', 'X'                    # Xbc
'ABC' -creplace 'a', 'X'                    # ABC (no change)
```

### Useful String Methods

PowerShell strings are .NET `[string]` objects, so all the usual methods are available:

```powershell
'  trim me  '.Trim()                        # 'trim me'
'CamelCase'.ToLower()                       # 'camelcase'
'a,b,c'.Split(',')                          # 'a', 'b', 'c'  (not regex)
'hello'.PadLeft(10, '0')                    # '00000hello'
'banana'.Replace('na', 'NA')                # 'baNANA' (literal, not regex)
'abc'.StartsWith('a')                       # True
'abc'.IndexOf('b')                          # 1
```

`.Replace()` is literal; `-replace` is regex. Pick deliberately.

## Related

- [`PowerShell Comparison And Logical Operators`](/powershell/syntax/comparison-and-logical-operators/)
- [`PowerShell JSON And XML Handling`](/powershell/syntax/json-and-xml-handling/)
- [`PowerShell Cmdlet Patterns And Filtering`](/powershell/syntax/cmdlet-patterns-and-filtering/)
