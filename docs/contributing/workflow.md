# Workflow

## Versions

The root `.version` file is the single source of truth (`VERSION=<major>.<minor>.<patch>`).
The major number is also the sprint number: `0.1.0` is sprint 0. The desktop build reads
it in `core/build.rs` and warns if `core/Cargo.toml` disagrees.

## Branches

- Active work happens on `sprint_<version>`, for example `sprint_0.1.0`.
- Branch features and fixes off the sprint branch, and open PRs against it, not `main`.
- At release, the sprint branch is merged into `main` and tagged `v<version>`.

## CI (`.github/workflows/`)

| Workflow | What it does |
| --- | --- |
| `build-core.yml` | Checks all six extensions (syntax and drift), then builds the Windows app |
| `build-android.yml` | Cross-compiles curl and builds the APK |
| `site-ci.yml`, `deploy-site.yml` | Website lint and build, then deploy to GitHub Pages |

Path filters keep, for example, a website-only PR from building the apps.

`build-core.yml` still builds `core/` with CMake from the old C++ app. It needs to move
to `cargo build --release` (see the [roadmap](../roadmap.md)).

## Releases

1. Go through the [release checklist](testing.md#release-checklist).
2. Build the release configurations. On Windows, ship `1idm.exe` together with
   `torrent-rasterbar.dll`, `libssl-3-x64.dll` and `libcrypto-3-x64.dll`.
3. Publish a GitHub release with the artifacts, SHA-256 checksums, notes and known limits.
4. Update the website's download links only after the files are published.

There's no Windows installer yet.
