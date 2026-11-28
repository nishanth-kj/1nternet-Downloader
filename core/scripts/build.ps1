# Builds the desktop app with cargo.
#
#   ./scripts/build.ps1                # debug   -> target\debug\1idm.exe
#   ./scripts/build.ps1 -Release       # release -> target\release\1idm.exe
#   ./scripts/build.ps1 -Test          # build, then run the tests
#   ./scripts/build.ps1 -InstallDeps   # install libtorrent with vcpkg first (once)
#
# Torrent support needs libtorrent in core\vcpkg_installed\x64-windows (see -InstallDeps).
param(
    [switch] $Release,
    [switch] $Test,
    [switch] $InstallDeps
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$CoreDir = Split-Path $PSScriptRoot -Parent
Push-Location $CoreDir
try {
    if ($InstallDeps) {
        $vcpkg = if ($env:VCPKG_ROOT) { Join-Path $env:VCPKG_ROOT "vcpkg.exe" } else { "vcpkg" }
        & $vcpkg install --x-manifest-root=. --x-install-root=vcpkg_installed --triplet x64-windows
        if ($LASTEXITCODE -ne 0) { throw "vcpkg install failed" }
    }
    $cargoArgs = @("build")
    if ($Release) { $cargoArgs += "--release" }
    cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
    if ($Test) {
        cargo test
        if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }
    }
} finally {
    Pop-Location
}
