---
title: PowerShell Credentials And Secure Strings
slug: /powershell/syntax/credentials-and-secure-strings/
summary: Reference for handling passwords and credentials in PowerShell using Get-Credential, PSCredential, SecureString, DPAPI-backed serialization, and the SecretManagement module.
topic: powershell/syntax
type: reference
tags: [powershell, credentials, securestring, pscredential, dpapi, secrets, secretmanagement]
aliases: [powershell get-credential, powershell securestring from plaintext, powershell convertto-securestring, powershell store password, powershell pscredential, powershell secretmanagement]
platforms: [windows, powershell, linux, macos]
related:
  - /powershell/networking/rest-and-web-requests/
  - /powershell/syntax/environment-variables/
  - /powershell/syntax/error-handling/
status: published
updated: 2026-05-14
---

## Synopsis

PowerShell wraps passwords in a `SecureString` (encrypted in memory) and pairs them with a username inside a `PSCredential`. Cmdlets that need authentication (`Invoke-RestMethod`, `Connect-PSSession`, `New-PSDrive`, etc.) accept a `-Credential` parameter. For anything beyond throwaway scripts, store secrets through `Microsoft.PowerShell.SecretManagement` instead of rolling your own file format.

## Syntax

```powershell
Get-Credential                                # interactive prompt
$secure = Read-Host -AsSecureString
$secure = ConvertTo-SecureString '<plain>' -AsPlainText -Force
$cred   = [PSCredential]::new('user', $secure)

ConvertFrom-SecureString $secure              # DPAPI-encrypted hex string
ConvertTo-SecureString   $hex                 # back to SecureString (same user/machine)

$cred.GetNetworkCredential().Password         # extract plaintext (use sparingly)
```

## Parameters/Flags

`PSCredential` constructor: `[PSCredential]::new(<username>, <SecureString>)`.

`ConvertTo-SecureString` options:
- `-AsPlainText -Force` — wrap a plaintext string (no encryption, just hides it from logs)
- `-Key <byte[]>` — symmetric key for round-tripping across machines/users
- (default) — input is a DPAPI hex string produced by `ConvertFrom-SecureString`

DPAPI scope on Windows:
- DPAPI-protected blobs are encrypted with the **current user's profile key on the current machine**.
- The same blob cannot be decrypted by another user, or on a different machine, unless you supply a `-Key`.
- On Linux/macOS, `ConvertFrom-SecureString` uses AES with a user-specific key file under `$HOME/.powershell`.

## Examples

### Interactive Prompt

```powershell
$cred = Get-Credential -Message 'Enter API credentials' -UserName 'apiuser'
Invoke-RestMethod -Uri 'https://api.example.com/data' -Credential $cred
```

The GUI prompt returns a `PSCredential`. On headless hosts, PowerShell falls back to a console prompt unless `$env:PSReadLineProfile` settings interfere.

### Build A PSCredential From Code

When the prompt isn't available (CI, scripts), construct one manually:

```powershell
$user   = 'apiuser'
$plain  = $env:API_PASSWORD
$secure = ConvertTo-SecureString $plain -AsPlainText -Force
$cred   = [PSCredential]::new($user, $secure)
```

`-AsPlainText -Force` does **not** encrypt anything — it just wraps the string in a `SecureString` so cmdlets that demand one will accept it. The plaintext was already exposed in `$plain`. Use this only when the secret entered the script from an equally exposed source (env var, vault response).

### Extract The Password

Sometimes you need the plaintext (e.g., to put it in a header):

```powershell
$plaintext = $cred.GetNetworkCredential().Password
```

This is the right way to pull plaintext out of a `SecureString`. Don't traffic it any further than necessary.

### Persisting A Secret With DPAPI

For per-user local storage on Windows, `ConvertFrom-SecureString` produces a string that's safe to write to disk — only the same user on the same machine can decrypt it.

```powershell
# Save:
$secure = Read-Host -AsSecureString -Prompt 'Enter token'
$secure | ConvertFrom-SecureString | Set-Content 'C:\Users\louis\token.dat'

# Load (later session, same user/machine):
$secure = Get-Content 'C:\Users\louis\token.dat' | ConvertTo-SecureString
$plain  = ([PSCredential]::new('x', $secure)).GetNetworkCredential().Password
```

Trying to read this file as a different user, on a different box, or in a different OS profile gets you an exception. That's the feature, not a bug.

### Cross-Machine Or Cross-User: Use A Key

If multiple users/machines need to decrypt the same blob, you must share an AES key — which means **the key itself becomes the secret you have to protect**:

```powershell
# Generate once, share securely:
$key = [byte[]](1..32 | ForEach-Object { Get-Random -Maximum 256 })
$key | Set-Content -Encoding Byte 'C:\secrets\shared.key'

# Encrypt with the key:
$secure | ConvertFrom-SecureString -Key $key | Set-Content '.\token.dat'

# Decrypt with the same key:
$key    = Get-Content -Encoding Byte 'C:\secrets\shared.key'
$secure = Get-Content '.\token.dat' | ConvertTo-SecureString -Key $key
```

This is roll-your-own. Prefer SecretManagement (below) if at all possible.

### SecretManagement (Recommended)

`Microsoft.PowerShell.SecretManagement` provides a uniform API over vault backends (Windows Credential Manager, KeePass, 1Password, Azure Key Vault, HashiCorp Vault, etc.):

```powershell
Install-Module Microsoft.PowerShell.SecretManagement -Scope CurrentUser
Install-Module Microsoft.PowerShell.SecretStore       -Scope CurrentUser

Register-SecretVault -Name LocalVault `
                     -ModuleName Microsoft.PowerShell.SecretStore `
                     -DefaultVault

Set-Secret    -Name 'github-pat' -Secret 'ghp_xxxx...'
Get-Secret    -Name 'github-pat' -AsPlainText
Get-SecretInfo
Remove-Secret -Name 'github-pat'
```

You can store any of: `String`, `SecureString`, `PSCredential`, `Hashtable`, `byte[]`. `Get-Secret -Name x` returns the original type.

```powershell
$cred = Get-Secret -Name 'api-creds'        # if stored as PSCredential, you get one back
Invoke-RestMethod -Uri ... -Credential $cred
```

### Reading A Secret From Environment Variables

For CI pipelines, env vars are usually the right shape:

```powershell
if (-not $env:GITHUB_TOKEN) {
    throw 'GITHUB_TOKEN is not set'
}

$headers = @{ Authorization = "Bearer $env:GITHUB_TOKEN" }
Invoke-RestMethod -Uri 'https://api.github.com/user' -Headers $headers
```

Env vars are plaintext on disk if persisted (User/Machine scope). Set them in the CI runner's secret store; never persist a real token via `setx`.

### Common Pitfalls

- **`ConvertTo-SecureString -AsPlainText -Force` is not encryption.** It's a type conversion. If your plaintext was in source code or a log, the `SecureString` doesn't undo that.
- **DPAPI blobs aren't portable.** Encrypting in user A's session and decrypting in user B's (or on a different machine, or in a scheduled task running as SYSTEM) will fail.
- **`Read-Host -AsSecureString` is interactive only.** It won't work in a non-interactive PowerShell host.
- **PowerShell 7 on Linux/macOS** uses a different (AES-based) backing store for `SecureString` serialization — blobs are still per-user and not portable across machines.
- **Logging.** Avoid `Write-Verbose "Got password: $plain"`. Verbose output goes to transcripts.

### Quick Reference: From Plaintext To Authenticated Request

```powershell
$user   = 'apiuser'
$secret = $env:API_TOKEN                                 # from secret store
$secure = ConvertTo-SecureString $secret -AsPlainText -Force
$cred   = [PSCredential]::new($user, $secure)

Invoke-RestMethod -Uri 'https://api.example.com/me' -Credential $cred
```

## Related

- [`PowerShell REST APIs And Web Requests`](/powershell/networking/rest-and-web-requests/)
- [`PowerShell Environment Variables`](/powershell/syntax/environment-variables/)
- [`PowerShell Error Handling`](/powershell/syntax/error-handling/)
