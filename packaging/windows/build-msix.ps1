# Packs a bundle made by bundle.ps1 (from the Store build, `cargo build --release --features store`)
# into the MSIX package for the Microsoft Store:
#
#   packaging/windows/build-msix.ps1 -Bundle dist\frename-store -Version 0.74.0.0 -Out frename.msix `
#       -IdentityName 12345Zelenov.frename -Publisher "CN=…" -PublisherDisplayName "Eugene Zelenov"
#
# The identity comes from Partner Center ("Product identity", docs/store-setup.md). Without it
# (a branch build, or before the name is reserved) a placeholder identity is used: the package
# still installs for testing, but the Store rejects it.
#
# The package is not signed: the Store signs what it publishes. test-msix.ps1 signs a copy with a
# throwaway certificate to install and test it.
param(
    [Parameter(Mandatory)] [string]$Bundle,
    [Parameter(Mandatory)] [string]$Version,
    [Parameter(Mandatory)] [string]$Out,
    [string]$IdentityName,
    [string]$Publisher,
    [string]$PublisherDisplayName
)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "windows-sdk.ps1")

if (!$IdentityName) { $IdentityName = "frename.dev" }
if (!$Publisher) { $Publisher = "CN=frename-dev" }
if (!$PublisherDisplayName) { $PublisherDisplayName = "frename (development build)" }

# The Store reserves the fourth part of the version: it must be 0.
if ($Version -notmatch '^\d+\.\d+\.\d+\.0$') {
    throw "MSIX version must be X.Y.Z.0, got '$Version'"
}
if (!(Test-Path (Join-Path $Bundle "frename.exe"))) { throw "No frename.exe in $Bundle" }
# The Store build has no updater; a Velopack Update.exe in it would mean the wrong bundle.
if (Test-Path (Join-Path $Bundle "Update.exe")) { throw "$Bundle holds Velopack's Update.exe" }

$staging = Join-Path ([System.IO.Path]::GetTempPath()) "frename-msix-$([guid]::NewGuid())"
Copy-Item $Bundle $staging -Recurse
Copy-Item (Join-Path $PSScriptRoot "msix\Assets") (Join-Path $staging "Assets") -Recurse

$escape = { param($value) [System.Security.SecurityElement]::Escape($value) }
$manifest = Get-Content (Join-Path $PSScriptRoot "msix\AppxManifest.xml") -Raw
$manifest = $manifest.Replace('$IDENTITY_NAME$', (& $escape $IdentityName)).
    Replace('$PUBLISHER$', (& $escape $Publisher)).
    Replace('$PUBLISHER_DISPLAY_NAME$', (& $escape $PublisherDisplayName)).
    Replace('$VERSION$', $Version)
if ($manifest -match '\$[A-Z_]+\$') { throw "Unfilled placeholder in the manifest: $($Matches[0])" }
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText((Join-Path $staging "AppxManifest.xml"), $manifest, $utf8NoBom)

# The resource index Windows reads the logos through (the certification kit checks it).
$makepri = Get-SdkTool "makepri.exe"
$priConfig = Join-Path ([System.IO.Path]::GetTempPath()) "frename-priconfig-$([guid]::NewGuid()).xml"
& $makepri createconfig /cf $priConfig /dq en-US /pv 10.0.0 /o
if ($LASTEXITCODE -ne 0) { throw "makepri createconfig failed" }
& $makepri new /pr $staging /cf $priConfig /mn (Join-Path $staging "AppxManifest.xml") /of (Join-Path $staging "resources.pri") /o
if ($LASTEXITCODE -ne 0) { throw "makepri new failed" }
Remove-Item $priConfig

$makeappx = Get-SdkTool "makeappx.exe"
$outDir = Split-Path $Out -Parent
if ($outDir) { New-Item -ItemType Directory -Path $outDir -Force | Out-Null }
& $makeappx pack /h SHA256 /d $staging /p $Out /o
if ($LASTEXITCODE -ne 0) { throw "makeappx pack failed" }
Remove-Item $staging -Recurse -Force

$size = [math]::Round((Get-Item $Out).Length / 1MB, 1)
Write-Host "$Out`: $IdentityName $Version, $size MB (unsigned)"
