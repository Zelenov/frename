# Finds the Windows SDK's tools (makeappx.exe, signtool.exe) and the Windows App Certification
# Kit. Dot-source it:  . packaging/windows/windows-sdk.ps1
# They are not on PATH on GitHub runners.

# The newest x64 copy of an SDK tool, e.g. "makeappx.exe".
function Get-SdkTool([string]$Name) {
    $bin = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
    $tool = Get-ChildItem $bin -Recurse -Filter $Name -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match '\\x64\\' } |
        Sort-Object FullName -Descending | Select-Object -First 1
    if (!$tool) { throw "$Name not found under ${bin}: is the Windows SDK installed?" }
    return $tool.FullName
}

# appcert.exe of the Windows App Certification Kit, or $null when the SDK was installed without it.
function Get-AppCert {
    $path = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\App Certification Kit\appcert.exe"
    if (Test-Path $path) { return $path }
    return $null
}
