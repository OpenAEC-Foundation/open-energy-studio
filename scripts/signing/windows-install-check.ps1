param([Parameter(Mandatory)][string]$Directory)
$ErrorActionPreference = 'Stop'
$Directory = (Resolve-Path $Directory).Path
$installers = @(Get-ChildItem $Directory -File | Where-Object { $_.Extension -in '.exe','.msi' })
if (@($installers | Where-Object Extension -eq '.exe').Count -ne 1 -or @($installers | Where-Object Extension -eq '.msi').Count -ne 1) { throw 'Expected exactly one NSIS and one MSI installer.' }
& "$PSScriptRoot/windows-verify.ps1" -Path $installers.FullName
foreach ($installer in $installers) {
  $destination = Join-Path $env:RUNNER_TEMP ('installed-' + $installer.Extension.TrimStart('.'))
  if ($installer.Extension -eq '.msi') {
    $process = Start-Process msiexec.exe -ArgumentList @('/i', ('"' + $installer.FullName + '"'), '/qn', '/norestart', ('INSTALLDIR="' + $destination + '"')) -Wait -PassThru
  } else {
    # NSIS /D must be the last argument and must not be quoted.
    $process = Start-Process $installer.FullName -ArgumentList "/S /D=$destination" -Wait -PassThru
  }
  if ($process.ExitCode -notin 0,3010) { throw "Installation failed: $($process.ExitCode)" }
  $main = Join-Path $destination 'open-energy-studio.exe'
  if (!(Test-Path -LiteralPath $main)) { throw "Installed application missing: $main" }
  # Check all shipped EXE/DLL files; this also catches an unsigned NSIS uninstaller.
  $files = @(Get-ChildItem $destination -Recurse -File | Where-Object { $_.Extension -in '.exe','.dll' })
  & "$PSScriptRoot/windows-verify.ps1" -Path $files.FullName
  if ($installer.Extension -eq '.msi') {
    $cleanup = Start-Process msiexec.exe -ArgumentList @('/x', ('"' + $installer.FullName + '"'), '/qn', '/norestart') -Wait -PassThru
  } else {
    $uninstaller = @(Get-ChildItem $destination -Filter '*uninstall*.exe')
    if ($uninstaller.Count -ne 1) { throw 'Expected one installed NSIS uninstaller.' }
    $cleanup = Start-Process $uninstaller[0].FullName -ArgumentList '/S' -Wait -PassThru
  }
  if ($cleanup.ExitCode -notin 0,3010) { throw 'Uninstall failed.' }
}
