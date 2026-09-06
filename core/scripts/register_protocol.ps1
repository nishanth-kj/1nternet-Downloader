# Registers (or removes) the "idr://" custom URI protocol handler for the current
# Windows user, so links like idr://https%3A%2F%2Fexample.com%2Ffile.zip open in
# Internet Downloader. This is what the browser extensions under browser/*/ hand off
# to when no native-messaging host is installed.
#
# Usage:
#   ./scripts/register_protocol.ps1 [-ExePath <path to InternetDownloader.exe>] [-Uninstall]
#
# Run from a normal (non-elevated) PowerShell — this only writes to HKCU, so no
# admin rights are required.

param(
    [string]$ExePath = (Join-Path $PSScriptRoot "..\out\build\x64-Debug\InternetDownloader.exe"),
    [switch]$Uninstall
)

$classKey = "HKCU:\Software\Classes\idr"

if ($Uninstall) {
    if (Test-Path $classKey) {
        Remove-Item -Path $classKey -Recurse -Force
        Write-Host "Removed idr:// protocol handler."
    } else {
        Write-Host "idr:// protocol handler was not registered."
    }
    exit 0
}

$resolvedExe = Resolve-Path -Path $ExePath -ErrorAction SilentlyContinue
if (-not $resolvedExe) {
    Write-Error "Could not find InternetDownloader.exe at '$ExePath'. Build the project first, or pass -ExePath explicitly."
    exit 1
}

New-Item -Path $classKey -Force | Out-Null
Set-ItemProperty -Path $classKey -Name "(default)" -Value "URL:Internet Downloader Protocol"
Set-ItemProperty -Path $classKey -Name "URL Protocol" -Value ""

New-Item -Path "$classKey\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path "$classKey\shell\open\command" -Name "(default)" -Value "`"$resolvedExe`" `"%1`""

Write-Host "Registered idr:// to launch: $resolvedExe"
