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

# Load .env
if (Test-Path $EnvFile) {
    Write-Host "Loading .env ..." -ForegroundColor Cyan
    Write-Host "Loading MSVC Environment..."
    $vswhere = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($vsPath) {
            $vcvars = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"
            if (Test-Path $vcvars) {
                $cmd = "`"$vcvars`" && set"
                cmd /c $cmd | ForEach-Object {
                    if ($_ -match "^(.*?)=(.*)$") {
                        [Environment]::SetEnvironmentVariable($matches[1], $matches[2])
                    }
                }
            }
        }
    }

    Get-Content $EnvFile | Where-Object { -not [string]::IsNullOrWhiteSpace($_) -and -not $_.TrimStart().StartsWith("#") } | ForEach-Object {
        $parts = $_.Trim() -split "=", 2
        if ($parts.Count -eq 2) {
            [Environment]::SetEnvironmentVariable($parts[0].Trim(), $parts[1].Trim(), "Process")
        }
    }
}


# Check VCPKG_ROOT
if (-not $env:VCPKG_ROOT) { Write-Error "VCPKG_ROOT not set"; exit 1 }
Write-Host "VCPKG : $env:VCPKG_ROOT"
Write-Host "Preset: $Preset"

# Load MSVC Env if needed
if (-not $env:VSCMD_ARG_TGT_ARCH) {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
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
}

# Clean
$BuildDir = Join-Path $CoreDir "out\build\$Preset"
if ($Clean -and (Test-Path $BuildDir)) { Remove-Item $BuildDir -Recurse -Force }

# CMake
Push-Location $CoreDir
try {
    Write-Host "`n-- Configure --" -ForegroundColor Cyan
    cmake --preset $Preset
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    if ($ConfigureOnly) { exit 0 }

    Write-Host "`n-- Build --" -ForegroundColor Cyan
    cmake --build --preset $Preset
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    Write-Host "`nSuccess!" -ForegroundColor Green
    Pop-Location
}
catch {
    Write-Error $_
    exit 1
}
