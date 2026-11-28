# Desktop UI

Built with Iced 0.13 in `core/src/ui/`. The layout follows classic download managers:
menu bar, toolbar, category tree, sortable list, details pane and status bar.

## Files

| File | Contents |
| --- | --- |
| `app.rs` | `IcedApp` state, `Message`, `update`, subscriptions, keyboard shortcuts, title-bar theming, start-with-Windows |
| `view.rs` | Main window: menus, toolbar, sidebar, list, details pane, status bar, menu popups, shared button styles |
| `dialogs.rs` | Add URL, Batch, Options, Remove, Download window (properties), Download complete, About |
| `format.rs` | Sizes, speeds, times, file categories from extensions, link detection |
| `prefs.rs` | `UiPrefs` (saved to `ui.json`) and the list `Column`s |
| `theme.rs` | `ThemeMode` (System, Classic light, Dark), `Accent`, `Palette` |

## How state flows

- `update` handles one `Message` and returns a `Task`. The public `update` wraps
  `update_inner` so that any theme change also re-themes the native title bar.
- Per-row "Added" and "Last try" times are tracked in the UI (`RowMeta`) while the app
  runs; the engine snapshot doesn't carry them.
- A download appearing after startup opens the download window, if that option is on.
  A download finishing opens "Download complete", if that option is on.

## Look and colour

- Text is small (about 11–12 px) and rows are 22 px, close to classic Windows apps.
- Icons come from Windows' *Segoe MDL2 Assets* font (`view::icon`). No third-party
  icon images are used.
- Colour roles:
  - The **accent** (green by default, chosen in Options › Appearance) is used only for
    download progress: progress bars, the "Downloading" status and the transfer rate.
  - Selection, hover, buttons and icons use neutral greys (`Palette::neutrals`).
  - Finished downloads are grey.
- `Palette::iced_theme` builds Iced's theme from the palette, so checkboxes, dropdowns
  and scrollbars match.
- **System** theme reads Windows' `AppsUseLightTheme` every ~5 s. The title bar follows
  through `DwmSetWindowAttribute(DWMWA_USE_IMMERSIVE_DARK_MODE)`.

## Saved preferences (`ui.json`)

Theme, accent, visible columns, sort column and direction, category and details panes,
category subfolders, download window on start, download window details, "Download
complete" dialog, ask-before-remove, and clipboard monitoring. Start-with-Windows is a
value under `HKCU\…\Run`, not in `ui.json`.

## Shortcuts

| Keys | Action |
| --- | --- |
| Ctrl+N / Ctrl+B | Add URL / Batch |
| Ctrl+F | Search |
| Ctrl+A | Select all visible rows |
| Ctrl/Shift + click | Multi-select |
| Double-click, Enter | Download window |
| Ctrl+R / Ctrl+P | Resume / pause selected |
| Del | Remove selected |
| Ctrl+O | Options |
| Esc | Close menu or dialog |

## Adding UI

- New user-facing settings belong in **Options**, not in the View menu. The View menu
  only holds panes and columns.
- Use the style helpers in `view.rs` (`flat_button`, `primary_button`,
  `secondary_button`, `input_style`, `panel`) so new widgets follow the theme.
- Don't use the accent colour for anything other than download progress.
