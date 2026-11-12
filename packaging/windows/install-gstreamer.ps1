# Installs the official GStreamer MSVC x86_64 package (runtime and development files) into one
# folder, for building frename and bundling it. Since 1.28 the package is an Inno Setup .exe;
# its portable mode writes no environment variables and no registry keys, so it is also safe on
# a developer machine that has another GStreamer.
#
#   packaging/windows/install-gstreamer.ps1 -Version 1.28.7 -Sha256 <sha256> `
#       -Dir C:\gstreamer\1.28 [-Download C:\gst-download]
#
# On GitHub Actions it also points the build at it (PKG_CONFIG_PATH, PATH, ...).
param(
    [Parameter(Mandatory)] [string]$Version,
    [Parameter(Mandatory)] [string]$Sha256,
    [Parameter(Mandatory)] [string]$Dir,
    # Where the installer is downloaded, and found again when cached.
    [string]$Download = (Join-Path $env:TEMP "gstreamer-download")
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"  # Invoke-WebRequest is many times slower with it

$name = "gstreamer-1.0-msvc-x86_64-$Version.exe"
$installer = Join-Path $Download $name
New-Item -ItemType Directory -Path $Download -Force | Out-Null
# A cached installer with the right checksum is used as it is. Otherwise it is downloaded; a
# download that is cut off or comes out wrong is tried again (3 attempts, 10 s apart), so one
# network hiccup does not fail a build or a release (#402).
function Test-Installer {
    (Test-Path $installer) -and ((Get-FileHash $installer -Algorithm SHA256).Hash -eq $Sha256)
}
if (!(Test-Installer)) {
    $url = "https://gstreamer.freedesktop.org/data/pkg/windows/$Version/msvc/$name"
    $attempts = 3
    for ($attempt = 1; $attempt -le $attempts; $attempt++) {
        Remove-Item $installer -ErrorAction SilentlyContinue
        Write-Host "Downloading $url"
        try {
            Invoke-WebRequest -Uri $url -OutFile $installer
            if (Test-Installer) { break }
            $actual = (Get-FileHash $installer -Algorithm SHA256).Hash
            $problem = "checksum mismatch: expected $Sha256, got $actual"
        } catch {
            $problem = $_.Exception.Message
        }
        if ($attempt -eq $attempts) {
            Remove-Item $installer -ErrorAction SilentlyContinue
            throw "Downloading $name failed $attempts times: $problem"
        }
        Write-Host "Download attempt $attempt failed ($problem); trying again in 10 s"
        Start-Sleep -Seconds 10
    }
}

$log = Join-Path $Download "install-$Version.log"
$arguments = @(
    "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CURRENTUSER",
    "/portable=1", "/TYPE=devel", "/DIR=$Dir", "/LOG=$log"
)
$process = Start-Process -FilePath $installer -ArgumentList $arguments -Wait -PassThru
if ($process.ExitCode -ne 0) {
    Get-Content $log -Tail 30
    throw "The GStreamer installer failed with exit code $($process.ExitCode)"
}
foreach ($expected in @("bin\gstreamer-1.0-0.dll", "bin\pkg-config.exe",
        "lib\pkgconfig\gstreamer-1.0.pc", "libexec\gstreamer-1.0\gst-plugin-scanner.exe")) {
    if (!(Test-Path (Join-Path $Dir $expected))) { throw "Not installed: $expected" }
}
Write-Host "GStreamer $Version is in $Dir"

if ($env:GITHUB_ENV) {
    Add-Content $env:GITHUB_ENV "GSTREAMER_ROOT=$Dir"
    Add-Content $env:GITHUB_ENV "PKG_CONFIG=$Dir\bin\pkg-config.exe"
    Add-Content $env:GITHUB_ENV "PKG_CONFIG_PATH=$Dir\lib\pkgconfig"
    Add-Content $env:GITHUB_PATH "$Dir\bin"
}
