---
title: PowerShell REST APIs And Web Requests
slug: /powershell/networking/rest-and-web-requests/
summary: Reference for calling REST APIs and downloading web content with Invoke-RestMethod and Invoke-WebRequest, including headers, bearer tokens, JSON bodies, retries, pagination, and PS5.1 versus PS7 differences.
topic: powershell/networking
type: reference
tags: [powershell, rest, http, invoke-restmethod, invoke-webrequest, api, json, tls]
aliases: [powershell invoke-restmethod, powershell bearer token, powershell post json, powershell api pagination, powershell invoke-webrequest tls, powershell rest retry]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/syntax/json-and-xml-handling/
  - /powershell/syntax/credentials-and-secure-strings/
  - /powershell/syntax/error-handling/
  - /powershell/networking/dns-and-network-monitoring/
status: published
updated: 2026-05-14
---

## Synopsis

`Invoke-RestMethod` parses the response body automatically (JSON → object, XML → `[xml]`); `Invoke-WebRequest` returns the full HTTP response so you can inspect headers and status codes. Use the first for APIs you trust, the second when you need the full picture (status codes, redirect chains, cookies, response headers).

## Syntax

```powershell
Invoke-RestMethod -Uri <url> [-Method GET|POST|PUT|DELETE|PATCH] `
                  [-Headers @{}] [-Body <obj-or-string>] `
                  [-ContentType 'application/json'] `
                  [-Authentication Bearer|Basic|OAuth] -Token <SecureString>

Invoke-WebRequest -Uri <url> -OutFile <path>             # download
$response = Invoke-WebRequest -Uri <url>
$response.StatusCode
$response.Content
$response.Headers
```

## Parameters/Flags

Common to both:
- `-Method` — HTTP verb (default `GET`)
- `-Headers @{ Key = 'value' }` — request headers
- `-Body` — request body; hashtable becomes form data, string is sent as-is
- `-ContentType` — `application/json`, `application/x-www-form-urlencoded`, etc.
- `-Credential` — `PSCredential` for basic auth
- `-UserAgent` — override the default UA
- `-MaximumRedirection 0` — disable redirects
- `-TimeoutSec N` — request timeout
- `-Proxy <url>` — proxy server

PS7-only:
- `-Authentication Bearer -Token <SecureString>` — clean bearer auth
- `-SkipCertificateCheck` — bypass TLS validation (dev only)
- `-SkipHttpErrorCheck` — don't throw on 4xx/5xx
- `-StatusCodeVariable <name>` — capture status code into a variable
- `-RetryIntervalSec` / `-MaximumRetryCount` — built-in retry on 5xx
- `-ResponseHeadersVariable <name>` — capture response headers separately

## Examples

### Basic GET

```powershell
$data = Invoke-RestMethod -Uri 'https://api.github.com/repos/anthropics/anthropic-sdk-python'
$data.full_name        # anthropics/anthropic-sdk-python
$data.stargazers_count
```

GitHub's API returns JSON; `Invoke-RestMethod` deserializes it into a `PSCustomObject` automatically.

### Headers And Bearer Token

```powershell
$headers = @{
    Accept        = 'application/vnd.github+json'
    Authorization = "Bearer $env:GITHUB_TOKEN"
    'X-Custom-Id' = 'abc123'
}

Invoke-RestMethod -Uri 'https://api.github.com/user' -Headers $headers
```

PS7 has a typed alternative that avoids putting the token in a header string:

```powershell
$token = ConvertTo-SecureString $env:GITHUB_TOKEN -AsPlainText -Force
Invoke-RestMethod -Uri 'https://api.github.com/user' `
                  -Authentication Bearer -Token $token
```

### Basic Auth

```powershell
$cred = Get-Credential
Invoke-RestMethod -Uri 'https://example.com/api' -Credential $cred -Authentication Basic
```

In PS5.1 (no `-Authentication`), build the header by hand:

```powershell
$pair    = "$user`:$pass"
$bytes   = [Text.Encoding]::ASCII.GetBytes($pair)
$encoded = [Convert]::ToBase64String($bytes)
$headers = @{ Authorization = "Basic $encoded" }
```

### POST With A JSON Body

```powershell
$payload = @{
    title  = 'Bug report'
    body   = 'Steps to reproduce: ...'
    labels = @('bug', 'triage')
}

Invoke-RestMethod -Uri 'https://api.github.com/repos/foo/bar/issues' `
                  -Method POST `
                  -Headers @{ Authorization = "Bearer $env:GITHUB_TOKEN" } `
                  -ContentType 'application/json' `
                  -Body ($payload | ConvertTo-Json -Depth 10)
```

Two key things:
- Set `-ContentType 'application/json'` explicitly. Without it, a hashtable `-Body` is sent as form-encoded.
- `ConvertTo-Json -Depth 10` defends against the depth-2 truncation gotcha (see [JSON handling](/powershell/syntax/json-and-xml-handling/)).

### Form-Encoded POST

```powershell
Invoke-RestMethod -Uri 'https://httpbin.org/post' `
                  -Method POST `
                  -Body @{ user = 'alice'; role = 'admin' }
```

A hashtable body without `-ContentType` is automatically form-encoded as `user=alice&role=admin`.

### Downloading A File

```powershell
Invoke-WebRequest -Uri 'https://example.com/big.zip' -OutFile 'C:\downloads\big.zip'
```

For large downloads on PS5.1, suppress the progress bar — it drastically slows transfers:

```powershell
$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest -Uri 'https://example.com/big.zip' -OutFile 'C:\downloads\big.zip'
$ProgressPreference = 'Continue'
```

### Inspecting The Full Response

When you need headers or status codes, use `Invoke-WebRequest`:

```powershell
$response = Invoke-WebRequest -Uri 'https://api.example.com/data'
$response.StatusCode                   # 200
$response.Headers['Content-Type']
$response.Content                      # raw body as string
$response.Content | ConvertFrom-Json   # parse if you know it's JSON
```

In PS7, `Invoke-RestMethod` also exposes them through `-ResponseHeadersVariable` and `-StatusCodeVariable`:

```powershell
Invoke-RestMethod -Uri 'https://api.example.com/data' `
                  -ResponseHeadersVariable headers `
                  -StatusCodeVariable      status

$status              # 200
$headers['Content-Type']
```

### Error Handling

By default, 4xx/5xx responses throw a terminating error in `try`/`catch`:

```powershell
try {
    Invoke-RestMethod -Uri 'https://httpbin.org/status/500'
}
catch {
    $resp = $_.Exception.Response
    if ($resp) {
        "Status: $($resp.StatusCode.value__) $($resp.ReasonPhrase)"
    }
    Write-Warning $_.Exception.Message
}
```

PS7 makes this easier with `-SkipHttpErrorCheck`:

```powershell
$result = Invoke-RestMethod -Uri 'https://httpbin.org/status/500' `
                            -SkipHttpErrorCheck `
                            -StatusCodeVariable status
if ($status -ge 400) { 'failed' } else { 'ok' }
```

See [`PowerShell Error Handling`](/powershell/syntax/error-handling/) for the broader patterns around terminating-vs-non-terminating errors.

### Retries With Backoff

PS7 has retries built in:

```powershell
Invoke-RestMethod -Uri 'https://api.example.com/data' `
                  -MaximumRetryCount 5 `
                  -RetryIntervalSec  2
```

PS5.1 — roll your own:

```powershell
function Invoke-WithRetry {
    param([scriptblock]$Action, [int]$Max = 5)
    for ($i = 1; $i -le $Max; $i++) {
        try { return & $Action }
        catch {
            if ($i -eq $Max) { throw }
            Start-Sleep -Seconds ([math]::Pow(2, $i))   # 2, 4, 8, 16, 32
        }
    }
}

Invoke-WithRetry { Invoke-RestMethod -Uri 'https://api.example.com/data' }
```

### Pagination

**Cursor-based (offset/limit):**

```powershell
$results = @()
$page    = 1
do {
    $batch = Invoke-RestMethod -Uri "https://api.example.com/items?page=$page&per_page=100"
    $results += $batch.items
    $page++
} while ($batch.items.Count -eq 100)
```

**Link-header-based (GitHub-style):**

```powershell
$url = 'https://api.github.com/repos/anthropics/anthropic-sdk-python/issues?per_page=100'
$all = @()

while ($url) {
    $resp = Invoke-WebRequest -Uri $url -Headers @{ Authorization = "Bearer $env:GITHUB_TOKEN" }
    $all += ($resp.Content | ConvertFrom-Json)

    # Parse the Link header for rel="next"
    $link = $resp.Headers['Link']
    if ($link -match '<([^>]+)>;\s*rel="next"') { $url = $Matches[1] } else { $url = $null }
}
```

### Multipart File Upload (PS7)

```powershell
$form = @{
    file        = Get-Item 'C:\path\report.pdf'
    description = 'Q2 report'
}
Invoke-RestMethod -Uri 'https://api.example.com/upload' -Method POST -Form $form
```

`-Form` is PS7-only. On PS5.1 you build the multipart body manually or use `System.Net.Http.HttpClient`.

### The PS5.1 TLS Gotcha

Windows PowerShell 5.1 defaults to SSL 3.0 / TLS 1.0 on older machines. Many APIs require TLS 1.2+. Set it for the session before making the call:

```powershell
[Net.ServicePointManager]::SecurityProtocol =
    [Net.SecurityProtocolType]::Tls12 -bor `
    [Net.SecurityProtocolType]::Tls13
```

PowerShell 7 picks up the OS's modern defaults automatically — this only matters on PS5.1.

### Skipping Certificate Validation (Dev Only)

PS7:

```powershell
Invoke-RestMethod -Uri 'https://self-signed.example/' -SkipCertificateCheck
```

PS5.1 — set a callback for the session (revert when done):

```powershell
[System.Net.ServicePointManager]::ServerCertificateValidationCallback = { $true }
```

Don't ship this. It is for poking at local dev hosts only.

### Proxies

```powershell
Invoke-RestMethod -Uri 'https://api.example.com' `
                  -Proxy 'http://corp-proxy:8080' `
                  -ProxyUseDefaultCredentials
```

## Related

- [`PowerShell JSON And XML Handling`](/powershell/syntax/json-and-xml-handling/)
- [`PowerShell Credentials And Secure Strings`](/powershell/syntax/credentials-and-secure-strings/)
- [`PowerShell Error Handling`](/powershell/syntax/error-handling/)
- [`DNS And Network Monitoring`](/powershell/networking/dns-and-network-monitoring/)
