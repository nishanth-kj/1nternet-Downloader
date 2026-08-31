param(
    [string] $Preset = "default",
    [switch] $ConfigureOnly,
    [switch] $Clean
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Paths
$ScriptDir = $PSScriptRoot
$CoreDir = Split-Path $ScriptDir -Parent
$EnvFile = Join-Path $CoreDir ".env"

# Global vcpkg
$VcpkgRoot = "C:\vcpkg"

if (-not (Test-Path (Join-Path $VcpkgRoot "vcpkg.exe"))) {
    Write-Error "vcpkg not found at $VcpkgRoot"
    exit 1
}

$env:VCPKG_ROOT = $VcpkgRoot

Write-Host "VCPKG : $env:VCPKG_ROOT" -ForegroundColor Cyan
Write-Host "Preset: $Preset"

# Load MSVC Environment
$vswhere = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"

if (Test-Path $vswhere) {
    $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath

    if ($vsPath) {
        $vcvars = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"

        if (Test-Path $vcvars) {
            Write-Host "Loading MSVC Environment..." -ForegroundColor Cyan

            cmd.exe /c "`"$vcvars`" >NUL && set" | ForEach-Object {
                if ($_ -match "^([^=]+)=(.*)$") {
                    [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
                }
            }
        }
    }
}

# Keep global vcpkg after loading MSVC environment
$env:VCPKG_ROOT = $VcpkgRoot

# Clean
$BuildDir = Join-Path $CoreDir "build"

if ($Clean -and (Test-Path $BuildDir)) {
    Write-Host "Cleaning build directory..." -ForegroundColor Yellow
    Remove-Item $BuildDir -Recurse -Force
}

# CMake
Push-Location $CoreDir

try {
    Write-Host "`n-- Configure --" -ForegroundColor Cyan

    cmake --preset $Preset

    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    if ($ConfigureOnly) {
        exit 0
    }

    Write-Host "`n-- Build --" -ForegroundColor Cyan

    cmake --build --preset $Preset

    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    Write-Host "`nSuccess!" -ForegroundColor Green
}
catch {
    Write-Error $_
    exit 1
}
finally {
    Pop-Location
}