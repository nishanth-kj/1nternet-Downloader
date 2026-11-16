# agents/ — project knowledge for coding agents and contributors

Read these documents before you change the code. They are written for AI coding agents
(Claude Code, Codex, Cursor, …) and for human contributors who are new to the code.
The root `AGENTS.md` and `CLAUDE.md` files stay short and link here, so these
documents only use context when they are relevant.

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
CLAUDE.md              @AGENTS.md + Claude Code specifics
<component>/AGENTS.md  component rules, loaded only when an agent works in that folder
<component>/CLAUDE.md  @AGENTS.md (same content for Claude Code)
agents/*.md            this directory: spec, design, conventions, workflows, roadmap
docs/                  user and release docs (install guide, correctness checklist)
```

Claude Code reads `CLAUDE.md` (which imports `AGENTS.md`). Other agents read
`AGENTS.md` directly. Nested files in `core/`, `android/`, `browser/` and `site/`
load only when an agent opens files in that folder. Keep each instruction file
under ~200 lines, and put long material in this directory.

## Keeping this accurate

- Update the matching document in the same change as the code it describes.
- If a document contradicts the code, trust the code, then fix the document.
- Claude Code users can run `/doctor prompt-audit` to find instructions that are stale
  or contradict each other.
