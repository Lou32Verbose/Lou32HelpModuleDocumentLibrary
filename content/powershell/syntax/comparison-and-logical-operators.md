---
title: PowerShell Comparison And Logical Operators
slug: /powershell/syntax/comparison-and-logical-operators/
summary: Reference for equality, pattern, membership, type, and logical operators in PowerShell, including case-sensitive variants and the array filtering gotcha.
topic: powershell/syntax
type: reference
tags: [powershell, operators, comparison, where-object, regex, like, match]
aliases: [powershell -eq vs -like, powershell -contains vs -in, case sensitive comparison powershell, powershell array filter operator, powershell logical and or]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/syntax/strings-and-regex/
  - /powershell/querying/useful-query-patterns/
  - /powershell/syntax/cmdlet-patterns-and-filtering/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell comparison operators use word-style prefixes (`-eq`, `-like`, `-match`) instead of symbols. They behave differently when the left side is a scalar vs. an array — a single operator can be either a boolean test or a filter, and forgetting that distinction is the most common bug in this corner of the language.

## Syntax

```powershell
<value> -<operator> <value>
<array> -<operator> <value>   # returns matching elements, not a bool
-not <value>
<expr> -and <expr>
<expr> -or  <expr>
```

## Parameters/Flags

- All operators are case-**insensitive** by default.
- Prefix with `-c` for case-sensitive (`-ceq`, `-clike`, `-cmatch`).
- Prefix with `-i` to be explicit about insensitivity (`-ieq`).
- `-not` (alias `!`) negates a boolean.

## Examples

### Equality And Ordering

```powershell
1 -eq 1            # True
'A' -eq 'a'        # True (case-insensitive by default)
'A' -ceq 'a'       # False (case-sensitive)
5 -gt 3            # True
5 -ge 5            # True
5 -ne 3            # True
```

### Pattern Operators

`-like` uses shell wildcards (`*`, `?`). `-match` uses regex.

```powershell
'report.log' -like '*.log'        # True
'report.log' -like 'r?port*'      # True

'user42' -match '^user(\d+)$'     # True
$Matches[1]                       # '42' — capture group from last -match
```

`-notlike` and `-notmatch` invert the result.

### Membership Operators

`-contains` and `-in` are reversed sides of the same test:

```powershell
@('a','b','c') -contains 'b'      # True  — collection on left, value on right
'b' -in @('a','b','c')            # True  — value on left, collection on right
```

`-contains` does **not** do substring matching. Use `-like '*b*'` or `-match 'b'` for that.

### Type Operators

```powershell
42 -is [int]                      # True
42 -is [string]                   # False
'42' -as [int]                    # 42  — returns converted value or $null
'abc' -as [int]                   # $null — failed conversion
```

`-as` is useful for "try to convert, fall back if it fails":

```powershell
$n = '42x' -as [int]
if ($null -eq $n) { 'not a number' } else { "got $n" }
```

### Logical Operators

```powershell
$true -and $false                 # False
$true -or  $false                 # True
-not $true                        # False
!$true                            # False (shorthand)
```

Combine with parentheses when mixing:

```powershell
if (($age -ge 18) -and ($status -eq 'active')) {
    'eligible'
}
```

### The Array Filtering Gotcha

When the left side of a comparison is an array, PowerShell returns the **matching elements**, not `$true`/`$false`:

```powershell
@(1,2,3,4,5) -gt 2                # 3, 4, 5
@(1,2,3,4,5) -eq 3                # 3
@('apple','banana','avocado') -like 'a*'   # apple, avocado
```

This is a built-in `Where-Object` shortcut. It only matters in boolean contexts:

```powershell
if (@(1,2,3) -eq 99) { 'found' } else { 'missing' }
# 'missing' — empty array is falsy

if (@(1,2,3) -eq 2)  { 'found' } else { 'missing' }
# 'found' — non-empty array is truthy
```

To test "does this scalar equal anything in an array", flip the operands:

```powershell
2 -in @(1,2,3)                    # True (proper boolean test)
```

### Null Comparison

Always put `$null` on the **left** side:

```powershell
if ($null -eq $value) { 'is null' }       # correct
if ($value -eq $null) { 'is null' }       # wrong if $value is an array
```

If `$value` is an array, `$value -eq $null` returns the null elements of the array (per the gotcha above), not a boolean.

### Case-Sensitive Variants

| Default       | Case-sensitive | Explicit insensitive |
| ------------- | -------------- | -------------------- |
| `-eq`         | `-ceq`         | `-ieq`               |
| `-ne`         | `-cne`         | `-ine`               |
| `-like`       | `-clike`       | `-ilike`             |
| `-match`      | `-cmatch`      | `-imatch`            |
| `-replace`    | `-creplace`    | `-ireplace`          |
| `-contains`   | `-ccontains`   | `-icontains`         |
| `-in`         | `-cin`         | `-iin`               |

### Combining With Where-Object

All of these operators work in `Where-Object` filters:

```powershell
Get-Process | Where-Object { $_.WorkingSet -gt 100MB }
Get-ChildItem | Where-Object { $_.Name -like '*.log' -and $_.Length -gt 0 }
Get-Service | Where-Object { $_.Status -eq 'Running' -and $_.StartType -ne 'Manual' }
```

## Related

- [`PowerShell Strings, Regex, And Format Operators`](/powershell/syntax/strings-and-regex/)
- [`PowerShell Useful Query Patterns`](/powershell/querying/useful-query-patterns/)
- [`PowerShell Cmdlet Patterns And Filtering`](/powershell/syntax/cmdlet-patterns-and-filtering/)
