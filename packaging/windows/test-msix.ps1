# Checks the Microsoft Store package (build-msix.ps1) on a clean runner (no GStreamer):
#
#   packaging/windows/test-msix.ps1 -Msix frename.msix -Report wack-report.xml
#
# 1. refuses to run where a GStreamer is installed, so a pass means "works on a clean Windows";
# 2. signs a copy with a throwaway certificate named like the package's publisher and trusts it
#    on this machine (Windows installs only signed packages; the Store signs the real one);
# 3. installs it, checks that every DLL in the package is bundled or part of Windows, and runs
#    frename's self-test on tests/media inside the package, through its `frename.exe` alias;
# 4. uninstalls it and runs the Windows App Certification Kit on it, writing its report to
#    -Report; a FAIL fails the script. Where the kit is not installed, says so and skips it.
# Run from the repository root (for tests/media), as administrator. Exits non-zero on the first
# failure.
param(
    [Parameter(Mandatory)] [string]$Msix,
    [Parameter(Mandatory)] [string]$Report
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
. (Join-Path $PSScriptRoot "windows-sdk.ps1")

$clips = (Resolve-Path "tests\media").Path
$work = Join-Path ([System.IO.Path]::GetTempPath()) "frename-msix-test"
if (Test-Path $work) { Remove-Item $work -Recurse -Force }
New-Item -ItemType Directory -Path $work | Out-Null

Write-Host "== 1. No GStreamer on this machine"
$systemGst = Get-Command gst-launch-1.0 -ErrorAction SilentlyContinue
if ($env:GSTREAMER_1_0_ROOT_MSVC_X86_64 -or $systemGst) {
    throw "This runner has a GStreamer; the test needs a machine without one."
}

Write-Host "== 2. Sign a copy for this machine"
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path $Msix).Path)
try {
    $entry = $zip.Entries | Where-Object { $_.FullName -eq "AppxManifest.xml" }
    $reader = New-Object System.IO.StreamReader($entry.Open())
    [xml]$manifest = $reader.ReadToEnd()
    $reader.Close()
} finally {
    $zip.Dispose()
}
$identity = $manifest.Package.Identity
Write-Host "Package: $($identity.Name) $($identity.Version), publisher $($identity.Publisher)"

$signed = Join-Path $work "frename-signed.msix"
Copy-Item $Msix $signed
# A code-signing certificate whose subject is the package's publisher, as Windows requires.
$cert = New-SelfSignedCertificate -Type Custom -Subject $identity.Publisher `
    -KeyUsage DigitalSignature -FriendlyName "frename CI test signing" `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
$password = ConvertTo-SecureString -String ([guid]::NewGuid().ToString()) -Force -AsPlainText
$pfx = Join-Path $work "test-signing.pfx"
Export-PfxCertificate -Cert $cert -FilePath $pfx -Password $password | Out-Null
$cer = Join-Path $work "test-signing.cer"
Export-Certificate -Cert $cert -FilePath $cer | Out-Null
Import-Certificate -FilePath $cer -CertStoreLocation "Cert:\LocalMachine\TrustedPeople" | Out-Null

$signtool = Get-SdkTool "signtool.exe"
$plain = [System.Net.NetworkCredential]::new("", $password).Password
& $signtool sign /fd SHA256 /f $pfx /p $plain $signed
if ($LASTEXITCODE -ne 0) { throw "signtool sign failed" }

Write-Host "== 3. Install and self-test"
Add-AppxPackage -Path $signed
$installed = Get-AppxPackage -Name $identity.Name
if (!$installed) { throw "Not installed: $($identity.Name)" }
Write-Host "Installed to $($installed.InstallLocation)"
& (Join-Path $PSScriptRoot "check-bundle.ps1") -Dir $installed.InstallLocation
if ($LASTEXITCODE -ne 0) { throw "The installed package is missing DLLs" }
if (Test-Path (Join-Path $installed.InstallLocation "Update.exe")) {
    throw "The Store package holds Velopack's Update.exe"
}

# Through the alias, so frename runs inside its package, as a Store install does. It is a GUI
# exe, so the process is waited for explicitly.
$alias = Join-Path $env:LOCALAPPDATA "Microsoft\WindowsApps\frename.exe"
if (!(Test-Path $alias)) { throw "The frename.exe alias was not registered: $alias" }
$process = Start-Process -FilePath $alias -ArgumentList @("--self-test", "`"$clips`"") -Wait -PassThru
# The log is in %LocalAppData%\frename, which Windows redirects into the package's own storage.
$logs = @(
    (Join-Path $env:LOCALAPPDATA "Packages\$($installed.PackageFamilyName)\LocalCache\Local\frename\frename_debug.log"),
    (Join-Path $env:LOCALAPPDATA "frename\frename_debug.log")
)
$log = $logs | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($log) {
    Write-Host "Log: $log"
    Get-Content $log
} else {
    Write-Host "(no log at $($logs -join ' or '))"
}
if ($process.ExitCode -ne 0) { throw "Self-test in the package failed: exit code $($process.ExitCode)" }
if ($log -and !(Select-String -Path $log -Pattern "Microsoft Store" -Quiet)) {
    throw "The packaged frename is not the Store build (its log does not say 'Microsoft Store')"
}
Remove-AppxPackage -Package $installed.PackageFullName

Write-Host "== 4. Windows App Certification Kit"
$appcert = Get-AppCert
if (!$appcert) {
    Write-Host "::warning::The Windows App Certification Kit is not installed on this runner; skipped."
    if ($env:GITHUB_STEP_SUMMARY) {
        Add-Content $env:GITHUB_STEP_SUMMARY "Store package: installed and self-tested; Windows App Certification Kit not available on the runner, skipped."
    }
    exit 0
}
$reportPath = [System.IO.Path]::GetFullPath($Report)
if (Test-Path $reportPath) { Remove-Item $reportPath }
& $appcert reset | Out-Null
& $appcert test -appxpackagepath (Resolve-Path $signed).Path -reportoutputpath $reportPath
if (!(Test-Path $reportPath)) { throw "appcert wrote no report (exit code $LASTEXITCODE)" }
[xml]$result = Get-Content $reportPath
$overall = $result.REPORT.OVERALL_RESULT
$tests = $result.SelectNodes("//TEST")
foreach ($test in $tests) {
    # The result is a child element in the kit's reports; an attribute is read too, to be safe.
    $node = $test.SelectSingleNode("RESULT")
    $outcome = if ($node) { $node.InnerText.Trim() } else { $test.GetAttribute("RESULT") }
    Write-Host ("{0,-8} {1}" -f $outcome, $test.GetAttribute("NAME"))
    if ($outcome -ne "PASS") {
        $test.SelectNodes(".//MESSAGE") | ForEach-Object { Write-Host "         $($_.InnerText)" }
    }
}
Write-Host "Windows App Certification Kit: $overall"
if ($env:GITHUB_STEP_SUMMARY) {
    Add-Content $env:GITHUB_STEP_SUMMARY "Store package: installed and self-tested; Windows App Certification Kit: **$overall**."
}
if ($overall -eq "FAIL") { throw "The Windows App Certification Kit failed the package" }
Write-Host "All Store package tests passed."
