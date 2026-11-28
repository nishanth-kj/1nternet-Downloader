# Documentation

Start with the overview, then open the folder for the part you're working on.

| Document | What it covers |
| --- | --- |
| [overview.md](overview.md) | The four products, how they connect, platform support |
| [roadmap.md](roadmap.md) | What works, known bugs, planned stages |

## Desktop app (`core/`)

| Document | What it covers |
| --- | --- |
| [desktop/building.md](desktop/building.md) | Build, run, test, and try changes safely |
| [desktop/architecture.md](desktop/architecture.md) | Source layout, threading, download engine, storage, IPC |
| [desktop/ui.md](desktop/ui.md) | Window layout, dialogs, themes, saved preferences |
| [desktop/torrents.md](desktop/torrents.md) | libtorrent bridge (C++ ↔ Rust) and its limits |

## Other products

| Document | What it covers |
| --- | --- |
| [browser/extension.md](browser/extension.md) | Browser extension: handoff, video detection, keeping six copies in sync |
| [android/README.md](android/README.md) | Android app: build, layout, rules |

## Contributing

| Document | What it covers |
| --- | --- |
| [contributing/conventions.md](contributing/conventions.md) | Rules that must not be broken, and code style |
| [contributing/testing.md](contributing/testing.md) | How to verify changes, and the release checklist |
| [contributing/workflow.md](contributing/workflow.md) | Branches, versions, CI, releases |

If a document disagrees with the code, trust the code and fix the document in the same change.
