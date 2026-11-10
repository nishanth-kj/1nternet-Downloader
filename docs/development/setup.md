# Development setup

The build, run and verification commands for every product (desktop, Android,
browser extensions, website) are in **[agents/WORKFLOWS.md](../../agents/WORKFLOWS.md)**.
To set up the Windows toolchain from scratch, see
[docs/installation/INSTALL.md](../installation/INSTALL.md).

Quick start for the desktop app:

```powershell
cd core
./scripts/build.ps1          # finds vcpkg via VCPKG_ROOT, C:\vcpkg, or Visual Studio's bundled copy
./scripts/register_protocol.ps1
./build/Debug/InternetDownloader.exe
```

New `.cpp` files under `core/src/` are picked up automatically by
`file(GLOB_RECURSE)`. Re-run the configure step (`build.ps1` does this) after you
add or remove files.

Before you open a PR, read [agents/CONVENTIONS.md](../../agents/CONVENTIONS.md).
If you changed the download engine, run the end-to-end check in
[core/tests/e2e/README.md](../../core/tests/e2e/README.md).
