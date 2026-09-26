# Lays out frename with its own GStreamer in one folder, ready for `vpk pack`:
#
#   packaging/windows/bundle.ps1 -GstRoot C:\gstreamer\1.0\msvc_x86_64 `
#       -Exe target\release\frename.exe -Out dist\frename
#
#   <Out>\frename.exe
#   <Out>\gst-plugin-scanner.exe
#   <Out>\*.dll                      GStreamer, GLib, FFmpeg and C runtime DLLs they import
#   <Out>\lib\gstreamer-1.0\*.dll    the plugins in gstreamer-plugins.txt
#   <Out>\licenses\                  licenses of the bundled parts, with source links
#
# DLLs are not hand-listed: every DLL imported, directly or not, by frename.exe, the scanner or a
# plugin is copied from GStreamer's bin\ or from Visual Studio's C runtime folder. The rest are
# Windows' own. Prints the bundle's size.
param(
    [Parameter(Mandatory)] [string]$GstRoot,
    [Parameter(Mandatory)] [string]$Exe,
    [Parameter(Mandatory)] [string]$Out
)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "vs-tools.ps1")

$dumpbin = Get-Dumpbin
$crtDir = Get-CrtRedistDir
$gstBin = Join-Path $GstRoot "bin"
$gstPlugins = Join-Path $GstRoot "lib\gstreamer-1.0"
Write-Host "GStreamer: $GstRoot"
Write-Host "C runtime: $crtDir"

if (Test-Path $Out) { Remove-Item $Out -Recurse -Force }
$pluginsOut = Join-Path $Out "lib\gstreamer-1.0"
New-Item -ItemType Directory -Path $pluginsOut -Force | Out-Null

Copy-Item $Exe (Join-Path $Out "frename.exe")
Copy-Item (Join-Path $GstRoot "libexec\gstreamer-1.0\gst-plugin-scanner.exe") $Out

$pluginNames = Get-Content (Join-Path $PSScriptRoot "gstreamer-plugins.txt") |
    ForEach-Object { $_.Trim() } | Where-Object { $_ -and !$_.StartsWith("#") }
foreach ($name in $pluginNames) {
    $plugin = Join-Path $gstPlugins "gst$name.dll"
    if (!(Test-Path $plugin)) { throw "Plugin not in this GStreamer: $plugin" }
    Copy-Item $plugin $pluginsOut
}

# Walk the imports from every executable file in the bundle.
$queue = New-Object System.Collections.Generic.Queue[string]
Get-ChildItem $Out -Recurse -Include *.exe, *.dll | ForEach-Object { $queue.Enqueue($_.FullName) }
$seen = @{}
while ($queue.Count -gt 0) {
    $file = $queue.Dequeue()
    foreach ($dll in Get-DllImports $dumpbin $file) {
        if ($seen.ContainsKey($dll)) { continue }
        $seen[$dll] = $true
        $fromGst = Join-Path $gstBin $dll
        $fromCrt = Join-Path $crtDir $dll
        if (Test-Path $fromGst) {
            Copy-Item $fromGst $Out
        } elseif (Test-Path $fromCrt) {
            Copy-Item $fromCrt $Out
        } elseif (Test-CrtDll $dll) {
            throw "$file imports $dll, which is neither in GStreamer nor in $crtDir"
        } else {
            continue  # a Windows DLL
        }
        $queue.Enqueue((Join-Path $Out $dll))
    }
}

# Licenses: GStreamer's package keeps one folder per component.
$licenses = Join-Path $Out "licenses"
Copy-Item (Join-Path $GstRoot "share\licenses") $licenses -Recurse
$gstVersion = (Get-Item (Join-Path $gstBin "gstreamer-1.0-0.dll")).VersionInfo.ProductVersion
$readme = @(
    "frename bundles GStreamer $gstVersion (the official MSVC x86_64 build) with GLib and FFmpeg,",
    "linked dynamically. GStreamer and FFmpeg are LGPL. Their sources:",
    "  https://gstreamer.freedesktop.org/src/",
    "  https://gitlab.freedesktop.org/gstreamer/cerbero (how the official build is made)",
    "  https://ffmpeg.org/download.html",
    "Each folder here holds the licenses of one component of that build."
)
Set-Content -Path (Join-Path $licenses "README.txt") -Value $readme -Encoding utf8

$files = Get-ChildItem $Out -Recurse -File
$mb = [math]::Round(($files | Measure-Object Length -Sum).Sum / 1MB, 1)
$dlls = (Get-ChildItem $Out -Filter *.dll).Count
Write-Host "Bundle: $($pluginNames.Count) plugins, $dlls DLLs next to the exe, $mb MB in $($files.Count) files"
