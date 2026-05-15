---
title: PowerShell JSON And XML Handling
slug: /powershell/syntax/json-and-xml-handling/
summary: Reference for parsing, building, querying, and serializing JSON and XML in PowerShell using ConvertTo/From-Json, the [xml] type accelerator, Select-Xml with XPath, and XmlDocument.
topic: powershell/syntax
type: reference
tags: [powershell, json, xml, convertto-json, convertfrom-json, select-xml, xpath]
aliases: [powershell parse json, powershell convertto-json depth, powershell xml type accelerator, powershell xpath, powershell read json file, powershell select-xml]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/networking/rest-and-web-requests/
  - /powershell/syntax/strings-and-regex/
  - /powershell/syntax/credentials-and-secure-strings/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell speaks JSON natively (`ConvertTo-Json` / `ConvertFrom-Json`, depth caveats apply) and treats XML as a first-class object graph via the `[xml]` type accelerator. Each format has one well-known pitfall: JSON's default `-Depth` of 2 truncates anything deeper, and XML's case-sensitive element access trips up people coming from JSON.

## Syntax

```powershell
$obj  = '<json-string>' | ConvertFrom-Json
$json = $obj | ConvertTo-Json -Depth 10

[xml]$doc = Get-Content file.xml -Raw
$doc.root.element.attribute

Select-Xml -Xml $doc -XPath '//item[@id="42"]'
```

## Parameters/Flags

`ConvertTo-Json`:
- `-Depth N` — how many levels of nested objects to serialize (**default 2** in Windows PowerShell, 100 in PS7 since `-Depth` was raised; but `ConvertFrom-Json` still has a default of 1024)
- `-Compress` — strip whitespace
- `-AsArray` — wrap a single object as a one-element array

`ConvertFrom-Json`:
- `-Depth N` — maximum nesting accepted (default 1024 in PS7)
- `-AsHashtable` — return hashtables instead of `PSCustomObject` (PS6+) — usually what you want for mutation

`Select-Xml`:
- `-XPath '<expr>'` — XPath 1.0 query
- `-Namespace @{ ns = 'uri' }` — declare namespaces for prefixed XPath

## Examples

### JSON: Parse And Access

```powershell
$json = '{"name":"alice","tags":["admin","ops"],"meta":{"age":30}}'
$obj  = $json | ConvertFrom-Json

$obj.name              # alice
$obj.tags              # admin, ops
$obj.tags[0]           # admin
$obj.meta.age          # 30
```

The return type is `PSCustomObject`. Properties are accessed with `.` and are case-insensitive in PowerShell, even though the underlying JSON keys are case-sensitive.

### JSON: Read From File

```powershell
$config = Get-Content 'config.json' -Raw | ConvertFrom-Json
$config.database.host
```

`-Raw` reads the file as a single string; without it, `Get-Content` produces an array of lines that `ConvertFrom-Json` can still parse but slightly more slowly.

### JSON: Build And Write

```powershell
$obj = [PSCustomObject]@{
    name    = 'alice'
    tags    = @('admin', 'ops')
    meta    = [PSCustomObject]@{ age = 30; active = $true }
    created = (Get-Date).ToString('o')
}

$obj | ConvertTo-Json -Depth 10 | Set-Content 'out.json'
```

A hashtable serializes too:

```powershell
@{ name = 'bob'; roles = @('user') } | ConvertTo-Json
```

### The -Depth Trap

`ConvertTo-Json` silently truncates beyond `-Depth`:

```powershell
$o = @{ a = @{ b = @{ c = @{ d = 'deep' } } } }
$o | ConvertTo-Json                         # truncates at depth 2
$o | ConvertTo-Json -Depth 10               # full structure
```

A safe default: `-Depth 10` (or `-Depth 100`) on every call where you don't know the input shape.

### JSON: Mutate Nested Structures

`PSCustomObject` properties returned from `ConvertFrom-Json` are read-only-ish — you can reassign them but can't add new ones easily. For free-form editing, use `-AsHashtable` (PS6+):

```powershell
$config = Get-Content config.json -Raw | ConvertFrom-Json -AsHashtable
$config['database']['port'] = 5433
$config['features'] += 'new-feature'
$config | ConvertTo-Json -Depth 10 | Set-Content config.json
```

### JSON: Round-Trip Without Re-formatting

`ConvertTo-Json | ConvertFrom-Json` may lose key ordering and change boolean/number formatting. If you need byte-stable round-tripping, treat the file as raw text and edit surgically with `-replace` instead.

### XML: Parse And Access With [xml]

The `[xml]` cast turns a string into an `XmlDocument`:

```powershell
[xml]$doc = @'
<settings>
  <database host="db.example.com" port="5432" />
  <features>
    <feature name="auth" enabled="true" />
    <feature name="cache" enabled="false" />
  </features>
</settings>
'@

$doc.settings.database.host        # db.example.com
$doc.settings.database.port        # 5432 (as string)
$doc.settings.features.feature     # array of two <feature> nodes
$doc.settings.features.feature[0].name      # auth
```

XML access is **case-sensitive** for element and attribute names — `$doc.settings.Database` returns `$null` even though `<database>` exists.

### XML: Read From File

```powershell
[xml]$doc = Get-Content 'config.xml' -Raw
```

Or use `Load`:

```powershell
$doc = New-Object System.Xml.XmlDocument
$doc.Load('C:\path\config.xml')
```

### XML: Build And Save

```powershell
$doc  = New-Object System.Xml.XmlDocument
$root = $doc.AppendChild($doc.CreateElement('items'))

foreach ($name in 'alpha','beta','gamma') {
    $item = $doc.CreateElement('item')
    $item.SetAttribute('name', $name)
    $root.AppendChild($item) | Out-Null
}

$doc.Save('C:\out\items.xml')
```

### XML: Modify An Existing Document

```powershell
[xml]$doc = Get-Content config.xml -Raw

$doc.settings.database.SetAttribute('port', '5433')

$new = $doc.CreateElement('feature')
$new.SetAttribute('name', 'new-thing')
$new.SetAttribute('enabled', 'true')
$doc.settings.features.AppendChild($new) | Out-Null

$doc.Save((Resolve-Path config.xml))
```

### XPath With Select-Xml

`Select-Xml` runs XPath queries — much cleaner than dotted navigation for filtering:

```powershell
[xml]$doc = Get-Content config.xml -Raw

Select-Xml -Xml $doc -XPath '//feature[@enabled="true"]' |
    ForEach-Object { $_.Node.name }
# auth
```

Each match is wrapped — get the underlying element with `.Node`.

XPath against namespaced XML requires explicit prefixes:

```powershell
[xml]$doc = Get-Content rss.xml -Raw
$ns = @{ atom = 'http://www.w3.org/2005/Atom' }

Select-Xml -Xml $doc -XPath '//atom:entry/atom:title' -Namespace $ns |
    ForEach-Object { $_.Node.InnerText }
```

### Pretty-Printing XML

`XmlDocument.Save` doesn't indent. Use an `XmlWriter`:

```powershell
$settings = New-Object System.Xml.XmlWriterSettings
$settings.Indent      = $true
$settings.IndentChars = '  '

$writer = [System.Xml.XmlWriter]::Create('out.xml', $settings)
$doc.Save($writer)
$writer.Close()
```

### Common Pitfalls

- **JSON `-Depth 2` default** in Windows PowerShell quietly drops nested data. Always specify `-Depth`.
- **`ConvertTo-Json` on a hashtable** with `null` values: the keys are kept; on a `PSCustomObject`, they may be too — but type coercion differs subtly. Test round-trips.
- **XML case sensitivity**: `<Foo>` and `<foo>` are different elements; PowerShell dotted access reflects that.
- **`Select-Xml` returns `SelectXmlInfo`** wrappers, not the matched nodes — drill in via `.Node`.
- **Newlines in JSON strings** are preserved; if you write them and read them back in another tool, `\n` may render as a literal backslash-n vs an actual newline depending on the consumer.

## Related

- [`PowerShell REST APIs And Web Requests`](/powershell/networking/rest-and-web-requests/)
- [`PowerShell Strings, Regex, And Format Operators`](/powershell/syntax/strings-and-regex/)
- [`PowerShell Credentials And Secure Strings`](/powershell/syntax/credentials-and-secure-strings/)
