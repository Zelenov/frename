# Checks frename's Windows packages on a clean runner (no GStreamer, never had one):
#
#   packaging/windows/test-package.ps1 -Setup frename-win-Setup.exe -Portable frename-portable.zip
#
# 1. refuses to run where a GStreamer is installed, so a pass means "works on a clean Windows";
# 2. installs Setup.exe silently, checks that every DLL the installed files import is bundled or
#    part of Windows, and runs the installed frename's self-test on tests/media;
# 3. installs an older official GStreamer system-wide, puts it on PATH and GST_PLUGIN_PATH, and
#    runs the self-test again: a system GStreamer must not get in the way;
# 4. unzips the portable package and runs its self-test.
# Run from the repository root (for tests/media). Exits non-zero on the first failure.
param(
    [Parameter(Mandatory)] [string]$Setup,
    [Parameter(Mandatory)] [string]$Portable
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# An older GStreamer for step 3: still an MSI, installed like users installed it for frename 0.66.
$OldGstVersion = "1.26.10"
$OldGstSha256 = "a863bf3faa49e9f33bd3cc42967b473482d4dc98655ed95cba1ac59f26fb0cfb"

$clips = (Resolve-Path "tests\media").Path

# Runs a frename exe's self-test and prints its log (from the data folder); release builds are
# GUI exes, so the process is waited for explicitly.
function Invoke-SelfTest([string]$Exe, [string]$DataDir, [string]$What) {
    Write-Host "== Self-test: $What"
    $process = Start-Process -FilePath $Exe -ArgumentList @("--self-test", "`"$clips`"") -Wait -PassThru
    $log = Join-Path $DataDir "frename_debug.log"
    if (Test-Path $log) { Get-Content $log } else { Write-Host "(no log at $log)" }
    if ($process.ExitCode -ne 0) { throw "Self-test failed ($What): exit code $($process.ExitCode)" }
}

Write-Host "== 1. No GStreamer on this machine"
$systemGst = Get-Command gst-launch-1.0 -ErrorAction SilentlyContinue
if ($env:GSTREAMER_1_0_ROOT_MSVC_X86_64 -or $systemGst) {
    throw "This runner has a GStreamer; the test needs a machine without one."
}

Write-Host "== 2. Install with Setup.exe"
# Waiting on Setup itself, not on the app it may start (-Wait would wait for children too).
$setupProcess = Start-Process -FilePath (Resolve-Path $Setup) -ArgumentList "--silent" -PassThru
$setupProcess.WaitForExit()
if ($setupProcess.ExitCode -ne 0) { throw "Setup.exe failed: exit code $($setupProcess.ExitCode)" }
# Velopack's docs disagree on whether Setup starts the app after a silent install.
Start-Sleep -Seconds 5
Get-Process frename -ErrorAction SilentlyContinue | Stop-Process -Force
$root = Join-Path $env:LOCALAPPDATA "frename"
$current = Join-Path $root "current"
if (!(Test-Path (Join-Path $current "frename.exe"))) { throw "Not installed: $current\frename.exe" }
Get-ChildItem $root | Format-Table Name, Length | Out-String | Write-Host
& (Join-Path $PSScriptRoot "check-bundle.ps1") -Dir $current
if ($LASTEXITCODE -ne 0) { throw "The installed bundle is missing DLLs" }
# The exe in current\, not the root launcher, whose exit code is not documented.
Invoke-SelfTest (Join-Path $current "frename.exe") $root "installed"

Write-Host "== 3. Installed, with GStreamer $OldGstVersion on the system"
$msi = Join-Path $env:TEMP "gstreamer-$OldGstVersion.msi"
Invoke-WebRequest -Uri "https://gstreamer.freedesktop.org/data/pkg/windows/$OldGstVersion/msvc/gstreamer-1.0-msvc-x86_64-$OldGstVersion.msi" -OutFile $msi
if ((Get-FileHash $msi -Algorithm SHA256).Hash -ne $OldGstSha256) { throw "Checksum mismatch for $msi" }
# Every feature, so its plugins compete with the bundled ones for every format.
$msiProcess = Start-Process msiexec.exe -ArgumentList @("/i", "`"$msi`"", "/qn", "ADDLOCAL=ALL") -Wait -PassThru
if ($msiProcess.ExitCode -ne 0) { throw "Installing GStreamer $OldGstVersion failed: $($msiProcess.ExitCode)" }
# The MSI records where it went in a machine-wide variable, as users' installs did.
$oldRootDir = [Environment]::GetEnvironmentVariable("GSTREAMER_1_0_ROOT_MSVC_X86_64", "Machine")
if (!$oldRootDir -or !(Test-Path (Join-Path $oldRootDir "bin\gstreamer-1.0-0.dll"))) {
    throw "GStreamer $OldGstVersion was not found after installing it (root: '$oldRootDir')"
}
$env:GSTREAMER_1_0_ROOT_MSVC_X86_64 = $oldRootDir
$oldBin = Join-Path $oldRootDir "bin"
$env:PATH = "$oldBin;$env:PATH"
$env:GST_PLUGIN_PATH = Join-Path (Split-Path $oldBin) "lib\gstreamer-1.0"
Write-Host "PATH starts with $oldBin; GST_PLUGIN_PATH=$env:GST_PLUGIN_PATH"
Invoke-SelfTest (Join-Path $current "frename.exe") $root "installed, system GStreamer $OldGstVersion"

Write-Host "== 4. Portable"
$portableDir = Join-Path $env:TEMP "frename-portable"
Expand-Archive -Path $Portable -DestinationPath $portableDir -Force
if (!(Test-Path (Join-Path $portableDir ".portable"))) { throw "The portable zip has no .portable marker" }
Invoke-SelfTest (Join-Path $portableDir "current\frename.exe") $portableDir "portable"

Write-Host "All package tests passed."
