param([Parameter(Mandatory)][string]$Path)
$ErrorActionPreference = 'Stop'
# Called by Tauri BEFORE packaging, including its generated NSIS uninstaller.
Import-Module ArtifactSigning -RequiredVersion 0.1.20
Invoke-ArtifactSigning -Endpoint $env:AZURE_ENDPOINT `
  -CodeSigningAccountName $env:AZURE_TRUSTED_SIGNING_ACCOUNT_NAME `
  -CertificateProfileName $env:AZURE_CERTIFICATE_PROFILE_NAME `
  -Files (Resolve-Path -LiteralPath $Path).Path `
  -FileDigest SHA256 -TimestampRfc3161 'http://timestamp.acs.microsoft.com' -TimestampDigest SHA256
& "$PSScriptRoot/windows-verify.ps1" -Path $Path
