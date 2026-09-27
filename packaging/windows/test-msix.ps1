# Checks the Microsoft Store package (build-msix.ps1) on a clean runner (no GStreamer):
#
#   packaging/windows/test-msix.ps1 -Msix frename.msix -Report wack-report.xml
#
# 1. refuses to run where a GStreamer is installed, so a pass means "works on a clean Windows";
# 2. signs a copy with a throwaway certificate named like the package's publisher and trusts it
#    on this machine (Windows installs only signed packages; the Store signs the real one);
# 3. installs it, checks that every DLL in the package is bundled or part of Windows, and runs
#    frename's self-test on tests/media inside the package, through its `frename.exe` alias,
#    which must also copy the installed version's settings (a database made for the test);
# 4. uninstalls it and runs the Windows App Certification Kit on it, writing its report to
#    -Report; anything but PASS or WARNING fails the script. Where the kit is not installed,
#    says so and skips it;
# 5. removes the package and the throwaway certificate again, also after a failure.
# Run from the repository root (for tests/media), as administrator, in Windows PowerShell
# (powershell.exe, not pwsh). Exits non-zero on the first failure.
param(
    [Parameter(Mandatory)] [string]$Msix,
    [Parameter(Mandatory)] [string]$Report
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
# The Appx cmdlets (Add-AppxPackage, Get-AppxPackage) do not load in PowerShell 7 on every Windows,
# Windows Server 2022 among them ("Operation is not supported on this platform").
if ($PSVersionTable.PSEdition -eq "Core") {
    throw "Run this script with Windows PowerShell (powershell.exe), not PowerShell 7 (pwsh)"
}
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
# The script uninstalls the package at the end, with its settings: never one that was here before.
if (Get-AppxPackage -Name $identity.Name) {
    throw "$($identity.Name) is already installed here; run this where frename from the Store is not"
}

$signed = Join-Path $work "frename-signed.msix"
Copy-Item $Msix $signed
# A code-signing certificate whose subject is the package's publisher, as Windows requires.
$cert = New-SelfSignedCertificate -Type Custom -Subject $identity.Publisher `
    -KeyUsage DigitalSignature -FriendlyName "frename CI test signing" `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
# The throwaway certificate must not stay trusted on the machine, whatever happens below.
$overall = $null
$seededDir = $null
$seededDb = $null
$pfx = Join-Path $work "test-signing.pfx"
$cer = Join-Path $work "test-signing.cer"
try {
    $password = ConvertTo-SecureString -String ([guid]::NewGuid().ToString()) -Force -AsPlainText
    Export-PfxCertificate -Cert $cert -FilePath $pfx -Password $password | Out-Null
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
    # Needs Visual Studio's dumpbin: always there on CI, maybe not where the owner tries it.
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if ($env:GITHUB_ACTIONS -or (Test-Path $vswhere)) {
        & (Join-Path $PSScriptRoot "check-bundle.ps1") -Dir $installed.InstallLocation
        if ($LASTEXITCODE -ne 0) { throw "The installed package is missing DLLs" }
    } else {
        Write-Host "No Visual Studio here: the check of the package's DLLs is skipped."
    }

    # The installed (Velopack) version's database, which the Store build copies on its first start,
    # read from outside its package. Made here if this machine has none (CI runners never do).
    $installedDir = Join-Path $env:LOCALAPPDATA "frename"
    $installedDb = Join-Path $installedDir "frename.db"
    if (!(Test-Path $installedDb)) {
        $python = Get-Command python -ErrorAction SilentlyContinue
        if ($python) {
            # Only what this script makes is removed afterwards: the folder if it was not there
            # (an installed frename that never started has it without a database).
            if (!(Test-Path $installedDir)) { $seededDir = $installedDir }
            New-Item -ItemType Directory -Path $installedDir -Force | Out-Null
            $seededDb = $installedDb
            $seed = Join-Path $work "seed.py"
            Set-Content $seed @'
import sqlite3, sys
db = sqlite3.connect(sys.argv[1])
db.execute("PRAGMA journal_mode=WAL")
db.execute("CREATE TABLE ci_seed (v TEXT)")
db.execute("INSERT INTO ci_seed VALUES ('from the installed version')")
db.commit()
db.close()
'@
            & $python.Source $seed $installedDb
            if ($LASTEXITCODE -ne 0) { throw "Could not create $installedDb" }
        } elseif ($env:GITHUB_ACTIONS) {
            throw "No python on this runner to make an installed version's database"
        } else {
            Write-Host "No python: the first-start import of the installed version's settings is not tested."
        }
    }

    # Through the alias, so frename runs inside its package, as a Store install does. It is a GUI
    # exe, so the process is waited for explicitly.
    $alias = Join-Path $env:LOCALAPPDATA "Microsoft\WindowsApps\frename.exe"
    if (!(Test-Path $alias)) { throw "The frename.exe alias was not registered: $alias" }
    $process = Start-Process -FilePath $alias -ArgumentList @("--self-test", "`"$clips`"") -Wait -PassThru
    # The log is in %LocalAppData%\frename-store, which Windows redirects into the package's own
    # storage (a new folder of a packaged app).
    $logs = @(
        (Join-Path $env:LOCALAPPDATA "Packages\$($installed.PackageFamilyName)\LocalCache\Local\frename-store\frename_debug.log"),
        (Join-Path $env:LOCALAPPDATA "frename-store\frename_debug.log")
    )
    $log = $logs | Where-Object { Test-Path $_ } | Select-Object -First 1
    if ($log) {
        Write-Host "Log: $log"
        Get-Content $log
    }
    if ($process.ExitCode -ne 0) { throw "Self-test in the package failed: exit code $($process.ExitCode)" }
    if (!$log) { throw "No log at $($logs -join ' or '): the packaged frename is not the Store build" }
    if (!(Select-String -Path $log -Pattern "Microsoft Store" -Quiet)) {
        throw "The packaged frename is not the Store build (its log does not say 'Microsoft Store')"
    }
    # "Open log" hands this path to Explorer, which runs outside the package: it must be the real
    # file, the one found above in the package's storage.
    if (!(Select-String -Path $log -Pattern ("log for other apps: " + $log) -SimpleMatch -Quiet)) {
        throw "The Store build would open its log at another path than $log"
    }
    if ((Test-Path $installedDb) -and !(Select-String -Path $log -Pattern "settings imported from" -Quiet)) {
        throw "The Store build did not import the installed version's settings from $installedDb"
    }
    Remove-AppxPackage -Package $installed.PackageFullName

    Write-Host "== 4. Windows App Certification Kit"
    $appcert = Get-AppCert
    if (!$appcert) {
        Write-Host "::warning::The Windows App Certification Kit is not installed on this runner; skipped."
        if ($env:GITHUB_STEP_SUMMARY) {
            Add-Content $env:GITHUB_STEP_SUMMARY "Store package: installed and self-tested; Windows App Certification Kit not available on the runner, skipped."
        }
    } else {
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
                # A message's text is its TEXT attribute or its content, depending on the kit.
                $test.SelectNodes(".//MESSAGE") | ForEach-Object {
                    $message = if ($_.GetAttribute("TEXT")) { $_.GetAttribute("TEXT") } else { $_.InnerText.Trim() }
                    if ($message) { Write-Host "         $message" }
                }
            }
        }
        Write-Host "Windows App Certification Kit: $overall"
        if ($env:GITHUB_STEP_SUMMARY) {
            Add-Content $env:GITHUB_STEP_SUMMARY "Store package: installed and self-tested; Windows App Certification Kit: **$overall**."
        }
    }
} finally {
    Get-AppxPackage -Name $identity.Name | Remove-AppxPackage -ErrorAction SilentlyContinue
    Remove-Item "Cert:\LocalMachine\TrustedPeople\$($cert.Thumbprint)" -ErrorAction SilentlyContinue
    Remove-Item "Cert:\CurrentUser\My\$($cert.Thumbprint)" -DeleteKey -ErrorAction SilentlyContinue
    Remove-Item $pfx, $cer -ErrorAction SilentlyContinue
    if ($seededDir) {
        Remove-Item $seededDir -Recurse -Force -ErrorAction SilentlyContinue
    } elseif ($seededDb) {
        foreach ($suffix in @("", "-wal", "-shm")) {
            Remove-Item "$seededDb$suffix" -Force -ErrorAction SilentlyContinue
        }
    }
}
# Only a report that says it passed counts: no result at all (a crashed kit) fails too.
if ($appcert -and $overall -notin @("PASS", "WARNING")) {
    throw "The Windows App Certification Kit did not pass the package (overall result: '$overall')"
}
Write-Host "All Store package tests passed."
