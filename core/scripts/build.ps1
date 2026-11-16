param(
    [ValidateSet("default", "release")]
    [string] $Preset = "default",
    [switch] $ConfigureOnly,
    [switch] $ForceConfigure,
    [switch] $Clean
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptDir = $PSScriptRoot
$CoreDir = Split-Path $ScriptDir -Parent
$OutDir = Join-Path $CoreDir "out"
$Jobs = [Environment]::ProcessorCount

function Find-VsInstallPath {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) {
        return $null
    }
    $path = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($path) { return $path.Trim() }
    return $null
}

function Import-MsvcEnvironment([string] $VsPath) {
    $vcvars = Join-Path $VsPath "VC\Auxiliary\Build\vcvars64.bat"
    if (-not (Test-Path $vcvars)) {
        Write-Error "vcvars64.bat not found under $VsPath"
        exit 1
    }

    Write-Host "Loading MSVC environment..." -ForegroundColor Cyan
    cmd.exe /c "`"$vcvars`" >NUL && set" | ForEach-Object {
        if ($_ -match "^([^=]+)=(.*)$") {
            [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
        }
    }
}

function Add-NinjaToPath([string] $VsPath) {
    if (Get-Command ninja -ErrorAction SilentlyContinue) {
        return
    }

    $ninja = Join-Path $VsPath "Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe"
    if (Test-Path $ninja) {
        $env:PATH = "$(Split-Path $ninja -Parent);$env:PATH"
        return
    }

    Write-Error "ninja.exe not found. Install the CMake tools workload in Visual Studio."
    exit 1
}

function Resolve-VcpkgRoot([string] $VsPath) {
    $candidates = @()
    if ($env:VCPKG_ROOT) { $candidates += $env:VCPKG_ROOT }
    if ($VsPath) { $candidates += (Join-Path $VsPath "VC\vcpkg") }
    $candidates += "C:\vcpkg"

    foreach ($root in $candidates) {
        $toolchain = Join-Path $root "scripts\buildsystems\vcpkg.cmake"
        if (Test-Path $toolchain) {
            return $root
        }
    }

    Write-Error "vcpkg toolchain not found. Set VCPKG_ROOT or install Visual Studio's vcpkg component."
    exit 1
}

$vsPath = Find-VsInstallPath
if (-not $vsPath) {
    Write-Error "Visual Studio C++ tools not found (vswhere)."
    exit 1
}

Import-MsvcEnvironment $vsPath
Add-NinjaToPath $vsPath

$vcpkgRoot = Resolve-VcpkgRoot $vsPath
$env:VCPKG_ROOT = $vcpkgRoot
$env:VCPKG_MAX_CONCURRENCY = "$Jobs"
$env:CMAKE_BUILD_PARALLEL_LEVEL = "$Jobs"

$binaryCache = Join-Path $env:LOCALAPPDATA "vcpkg\archives"
if (-not $env:VCPKG_BINARY_SOURCES) {
    New-Item -ItemType Directory -Force -Path $binaryCache | Out-Null
    $env:VCPKG_BINARY_SOURCES = "clear;files,$binaryCache,readwrite"
}

Write-Host "VCPKG : $env:VCPKG_ROOT" -ForegroundColor Cyan
Write-Host "Ninja : $((Get-Command ninja).Source)" -ForegroundColor Cyan
Write-Host "Jobs  : $Jobs" -ForegroundColor Cyan
Write-Host "Preset: $Preset"

if ($Clean -and (Test-Path $OutDir)) {
    Write-Host "Cleaning $OutDir ..." -ForegroundColor Yellow
    Remove-Item $OutDir -Recurse -Force
}

function Test-ConfigureNeeded {
    if ($ForceConfigure -or $ConfigureOnly) { return $true }
    $cacheFile = Join-Path $OutDir "CMakeCache.txt"
    if (-not (Test-Path $cacheFile)) { return $true }

    $genLine = Select-String -Path $cacheFile -Pattern '^CMAKE_GENERATOR:INTERNAL=' | Select-Object -First 1
    if ($genLine -and $genLine.Line -notmatch 'Ninja') {
        Write-Host "Build directory uses $($genLine.Line.Split('=')[1]); recreating for Ninja." -ForegroundColor Yellow
        Remove-Item $OutDir -Recurse -Force
        return $true
    }

    $cacheTime = (Get-Item $cacheFile).LastWriteTimeUtc
    $inputs = @(
        (Join-Path $CoreDir "CMakeLists.txt"),
        (Join-Path $CoreDir "vcpkg.json"),
        (Join-Path $CoreDir "CMakePresets.json")
    )
    foreach ($file in $inputs) {
        if ((Test-Path $file) -and (Get-Item $file).LastWriteTimeUtc -gt $cacheTime) {
            return $true
        }
    }
    return $false
}

$needConfigure = Test-ConfigureNeeded

Push-Location $CoreDir
try {
    if ($needConfigure) {
        Write-Host "`n-- Configure --" -ForegroundColor Cyan
        cmake --preset $Preset
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    else {
        Write-Host "`n-- Configure -- skipped (cache up to date; pass -ForceConfigure to rerun)" -ForegroundColor DarkGray
    }

    if ($ConfigureOnly) { exit 0 }

    $config = if ($Preset -eq "release") { "Release" } else { "Debug" }

    Write-Host "`n-- Build --" -ForegroundColor Cyan
    cmake --build $OutDir --config $config --parallel $Jobs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $exe = Join-Path $OutDir "$config\InternetDownloader.exe"
    Write-Host "`nSuccess: $exe" -ForegroundColor Green
}
catch {
    Write-Error $_
    exit 1
}
finally {
    Pop-Location
}
