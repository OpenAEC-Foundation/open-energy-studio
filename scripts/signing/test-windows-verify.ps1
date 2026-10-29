$ErrorActionPreference = 'Stop'
# Provider-independent failure tests. These mocks are NOT real signature evidence.
$env:WINDOWS_SIGNING_SUBJECT = 'CN=Expected Publisher'
$global:OesTestStatus = 'Valid'
$global:OesTestPublisher = $env:WINDOWS_SIGNING_SUBJECT
$global:OesTestTimestamp = [pscustomobject]@{Subject='CN=Timestamp'}
function Get-AuthenticodeSignature {
  param($LiteralPath)
  [pscustomobject]@{Status=$global:OesTestStatus; SignerCertificate=[pscustomobject]@{Subject=$global:OesTestPublisher}; TimeStamperCertificate=$global:OesTestTimestamp}
}
function Expect-Rejected {
  $rejected = $false
  try { & "$PSScriptRoot/windows-verify.ps1" -Path 'fixture.exe' } catch { $rejected = $true }
  if (!$rejected) { throw 'Verification incorrectly accepted a failing fixture.' }
}
& "$PSScriptRoot/windows-verify.ps1" -Path 'fixture.exe'
$global:OesTestStatus = 'NotSigned'; Expect-Rejected
$global:OesTestStatus = 'HashMismatch'; Expect-Rejected
$global:OesTestStatus = 'Valid'; $global:OesTestPublisher = 'CN=Other Publisher'; Expect-Rejected
$global:OesTestPublisher = $env:WINDOWS_SIGNING_SUBJECT; $global:OesTestTimestamp = $null; Expect-Rejected
$global:OesTestTimestamp = [pscustomobject]@{Subject='CN=Timestamp'}; $env:WINDOWS_SIGNING_SUBJECT = ''; Expect-Rejected
Get-ChildItem $PSScriptRoot -Filter '*.ps1' | ForEach-Object {
  $parseErrors = $null
  [void][System.Management.Automation.Language.Parser]::ParseFile($_.FullName, [ref]$null, [ref]$parseErrors)
  if ($parseErrors.Count) { throw ($parseErrors | Out-String) }
}
Write-Output 'Signature policy: valid fixture accepted; five invalid fixtures rejected; all PowerShell scripts parse.'
