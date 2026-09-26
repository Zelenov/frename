# Fails when an exe or DLL of a bundled frename imports a DLL that the bundle does not have and
# Windows does not provide. C runtime DLLs must be in the bundle even when System32 has them: a
# clean Windows may not, and a plugin that fails to load is dropped silently.
#
#   packaging/windows/check-bundle.ps1 -Dir $env:LOCALAPPDATA\frename\current
param([Parameter(Mandatory)] [string]$Dir)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "vs-tools.ps1")

$dumpbin = Get-Dumpbin
$system32 = Join-Path $env:SystemRoot "System32"
$bundled = @{}
Get-ChildItem $Dir -Filter *.dll | ForEach-Object { $bundled[$_.Name.ToLowerInvariant()] = $true }

$problems = @()
foreach ($file in Get-ChildItem $Dir -Recurse -Include *.exe, *.dll) {
    foreach ($dll in Get-DllImports $dumpbin $file.FullName) {
        if ($bundled.ContainsKey($dll)) { continue }
        # API sets (api-ms-win-*, ext-ms-*) are resolved by the loader, not found as files.
        if ($dll -match '^(api|ext)-ms-') { continue }
        if (Test-CrtDll $dll) {
            $problems += "$($file.Name) imports $dll, a C runtime DLL missing from the bundle"
        } elseif (!(Test-Path (Join-Path $system32 $dll))) {
            $problems += "$($file.Name) imports $dll, which is neither bundled nor part of Windows"
        }
    }
}
if ($problems) {
    $problems | ForEach-Object { Write-Host "::error::$_" }
    exit 1
}
Write-Host "Every import of every file in $Dir is bundled or part of Windows."
