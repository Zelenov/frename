# Finds Visual Studio's C++ tools: dumpbin.exe and the folder of the redistributable C runtime
# DLLs. Dot-source it:  . packaging/windows/vs-tools.ps1
# They are not on PATH on GitHub runners or outside a developer prompt.

function Get-VsInstallPath {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (!(Test-Path $vswhere)) { throw "vswhere.exe not found: is Visual Studio installed?" }
    $path = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$path) { throw "No Visual Studio with the C++ x64 tools found" }
    return $path
}

function Get-Dumpbin {
    $vs = Get-VsInstallPath
    $dumpbin = Get-ChildItem (Join-Path $vs "VC\Tools\MSVC\*\bin\Hostx64\x64\dumpbin.exe") |
        Sort-Object FullName -Descending | Select-Object -First 1
    if (!$dumpbin) { throw "dumpbin.exe not found under $vs" }
    return $dumpbin.FullName
}

function Get-CrtRedistDir {
    $vs = Get-VsInstallPath
    $crt = Get-ChildItem (Join-Path $vs "VC\Redist\MSVC\*\x64\Microsoft.VC*.CRT") -Directory |
        Where-Object { Test-Path (Join-Path $_.FullName "vcruntime140.dll") } |
        Sort-Object FullName -Descending | Select-Object -First 1
    if (!$crt) { throw "The VC++ runtime redistributable folder was not found under $vs" }
    return $crt.FullName
}

# The DLL names a file imports, lower-case.
function Get-DllImports([string]$Dumpbin, [string]$File) {
    $out = & $Dumpbin /nologo /dependents $File
    if ($LASTEXITCODE -ne 0) { throw "dumpbin failed on $File" }
    return $out | ForEach-Object { $_.Trim() } |
        Where-Object { $_ -match '^[\w.+-]+\.dll$' } |
        ForEach-Object { $_.ToLowerInvariant() }
}

# The C/C++ runtime DLLs, which must ship next to frename.exe (app-local).
function Test-CrtDll([string]$Name) {
    return $Name -match '^(vcruntime|msvcp|concrt|vccorlib)\d'
}
