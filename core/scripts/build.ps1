param(
    [ValidateSet("default", "release")]
    [string] $Preset = "default",
    [switch] $ConfigureOnly,
    [switch] $Clean
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not $env:VCPKG_ROOT) {
    throw "VCPKG_ROOT is not set. Set it to your vcpkg installation directory."
}

$vcpkgToolchain = Join-Path $env:VCPKG_ROOT "scripts/buildsystems/vcpkg.cmake"
if (-not (Test-Path $vcpkgToolchain)) {
    throw "vcpkg toolchain not found: $vcpkgToolchain"
}

$CoreDir = Split-Path $PSScriptRoot -Parent
$BuildDir = Join-Path $CoreDir "build"

if ($Clean -and (Test-Path $BuildDir)) {
    Remove-Item $BuildDir -Recurse -Force
}

Push-Location $CoreDir
try {
    cmake --preset $Preset
    if ($LASTEXITCODE -ne 0) {
        throw "CMake configuration failed."
    }

    if (-not $ConfigureOnly) {
        cmake --build --preset $Preset --parallel
        if ($LASTEXITCODE -ne 0) {
            throw "Build failed."
        }
    }
}
finally {
    Pop-Location
}
