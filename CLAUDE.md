# CLAUDE.md

@AGENTS.md

## Claude Code specifics

- Component rules load automatically from `core/`, `android/`, `browser/` and `site/`
  (each has its own `CLAUDE.md`). Open the deeper `agents/*.md` docs when the task
  needs them; they are linked from AGENTS.md rather than imported, to keep context small.
- Use PowerShell for `core/scripts/*.ps1`. The Bash tool here is Git Bash, so use
  `cygpath -w` when you hand paths to Windows programs.
- When you run the desktop app for verification, point `downloadDir` in
  `%APPDATA%\InternetDownloader\settings.json` at a scratch folder first. Write that
  JSON with Node rather than hand-escaping backslashes. Kill the app
  (`taskkill //IM InternetDownloader.exe //F`) and delete test data afterwards.
- Run `/doctor prompt-audit` after editing instruction files, to catch stale or
  conflicting rules.
