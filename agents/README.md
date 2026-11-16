# agents/ — project knowledge for coding agents and contributors

Read these documents before you change the code. They are written for AI coding agents
(Claude Code, Codex, Cursor, …) and for human contributors who are new to the code.
The root `AGENTS.md` stays short and links here, so these documents only use
context when they are relevant.

| File | Read it when you… |
| --- | --- |
| [PROJECT_SPEC.md](PROJECT_SPEC.md) | need to know what the product must do, per platform, and what is out of scope |
| [DESIGN.md](DESIGN.md) | touch the engine, threading, persistence, IPC or the GUI ↔ engine boundary |
| [CONVENTIONS.md](CONVENTIONS.md) | write or review code (invariants, things not to change, style) |
| [WORKFLOWS.md](WORKFLOWS.md) | build, run, verify or release any of the four products |
| [ROADMAP.md](ROADMAP.md) | look for known gaps or the next useful work |

## How the instruction files fit together

```text
AGENTS.md              canonical cross-tool instructions (short; links into agents/)
<component>/AGENTS.md  component rules, loaded only when an agent works in that folder
agents/*.md            this directory: spec, design, conventions, workflows, roadmap
docs/                  user and release docs (architecture, setup, platforms, releases, checklist)
```

`AGENTS.md` is the only instruction file; the repository has no `CLAUDE.md`. Nested files in `core/`, `android/`, `browser/` and `site/`
load only when an agent opens files in that folder. Keep each instruction file
under ~200 lines, and put long material in this directory.

## Keeping this accurate

- Update the matching document in the same change as the code it describes.
- If a document contradicts the code, trust the code, then fix the document.
