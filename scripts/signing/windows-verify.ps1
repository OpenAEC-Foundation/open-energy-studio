param([Parameter(Mandatory)][string[]]$Path)
$ErrorActionPreference = 'Stop'
foreach ($file in $Path) {
  $signature = Get-AuthenticodeSignature -LiteralPath $file
  if ($signature.Status -ne 'Valid' -or !$signature.SignerCertificate -or !$signature.TimeStamperCertificate) {
    throw "Missing valid publisher signature or timestamp: $file ($($signature.Status))"
  }
  if ([string]::IsNullOrWhiteSpace($env:WINDOWS_SIGNING_SUBJECT)) { throw 'WINDOWS_SIGNING_SUBJECT must pin the expected publisher subject.' }
  if ($signature.SignerCertificate.Subject -ne $env:WINDOWS_SIGNING_SUBJECT) { throw "Unexpected publisher: $file" }
  Write-Output "Verified publisher and timestamp: $file"
}
