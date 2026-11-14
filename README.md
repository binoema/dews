# dewz

**Kontoauszug auswerten – 100 % offline.**

dewz is a Windows desktop app for analysing your own bank account exports. Import the Excel or CSV export from your
bank, let the bookings be categorised automatically by keyword rules, and see where your money goes – per category,
per month and over time. Nothing ever leaves your computer: the app has no network access at all.

The user interface is in German.

## Features

- **Import** bank exports as Excel (`.xlsx`, `.xlsm`) or CSV/TXT (UTF-8, UTF-16 or Windows-1252, delimiter detected
  automatically). A mapping dialog suggests the columns (date, signed amount or separate debit/credit columns,
  recipient, purpose, booking text, category) and remembers the mapping for the next import. Several files can be imported at once, also via drag & drop; duplicate bookings are detected and skipped.
  The old Excel format `.xls` is not supported – save it as `.xlsx` or CSV first.
- **Automatic categorisation** by keywords in recipient, purpose or booking text. The longest matching keyword wins;
  `"word"` matches whole words only, `!keyword` overrides a category supplied by the import file. Bookings can be
  reassigned by hand, and "Regel erstellen …" turns such a correction into a rule.
- **Rücklagen (transfers)**: categories marked as transfers to your own accounts (e.g. savings) are excluded from
  income and expenses and reported separately.
- **Views**: expenses by category, monthly overview, categories compared month by month, small expenses (configurable
  threshold) and the full booking list – all with filters for text, category, type, month, date range and amount.
- **Export** bookings and the evaluation as CSV (Excel-compatible, protected against formula injection) and the
  category rules as JSON.
- **Data file** in a folder of your choice, saved automatically after every change – optionally **encrypted with a
  password** (AES-256-GCM, key derived with PBKDF2-SHA-256, 600,000 iterations).

## Using the app

1. Start dewz and click **Datenordner wählen**. Pick any folder – e.g. in your documents, a cloud folder or a
   USB stick. dewz creates `dewz-daten.json` there and reopens the folder automatically on the next start.
2. Click **Kontoexport importieren** (or drop files onto the window) and confirm the column mapping.
3. Adjust categories and rules under **Kategorien & Regeln**.
4. Optional: set a password in the **Datenordner** dialog. It is asked for at every start.
   **If you forget it, the data cannot be recovered.**

Two invented sample exports for trying the app are included: `Kontoauszug_Beispiel.xlsx` and
`Kontoauszug_Halbjahr.xlsx`.

### Where data is stored

| What | Where |
|---|---|
| Bookings, categories, rules | `dewz-daten.json` in the folder you chose (plain JSON or encrypted) |
| Remembered data folder | `%APPDATA%\app.dewz.desktop\einstellungen.json` |
| UI settings (last view, column mappings, small-expense threshold, a copy of the categories) | WebView2 profile in `%LOCALAPPDATA%\app.dewz.desktop` |

Data files from the app's former name (`finanz-buddy-daten.json`) are read and migrated to `dewz-daten.json`;
the old file is left untouched.

## Security and privacy

dewz handles personal financial data, so it is built to have as little attack surface as possible:

- **No network.** A strict Content Security Policy (`default-src 'none'`) blocks every connection, external script,
  font and image. The only allowed channel is the IPC bridge to the app's own Rust backend. Inline scripts and styles
  are only executed with the hash/nonce Tauri adds at build time; `eval` is blocked.
- **Narrow file access.** The web UI has no general file system access. The Rust backend
  (`src-tauri/src/main.rs`) exposes only a few commands: it reads only folders the user picked in the Windows dialog,
  writes only `dewz-daten.json` into the active folder, and writes exports only to the path chosen in the
  "Save as" dialog.
- **Atomic saves.** The data file is written to a temporary file first and then renamed, so a crash never leaves a
  half-written file.
- **Encryption stays in the UI.** Password and key never leave the web view; the backend only receives finished
  (encrypted) text.
- **Minimal permissions.** The Tauri capability (`src-tauri/capabilities/default.json`) only allows listening to
  window events and closing the window.

## Development

The app is a single self-contained HTML file (`src/index.html`, no build step, no frameworks) running in WebView2,
hosted by a small [Tauri 2](https://tauri.app) backend in Rust.

### Prerequisites (Windows)

- [Node.js](https://nodejs.org) 20 or newer (only for the Tauri CLI)
- [Rust](https://rustup.rs) (stable, MSVC toolchain) – `winget install Rustlang.Rustup`
- Microsoft C++ Build Tools with the "Desktop development with C++" workload –
  `winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"`
- WebView2 Runtime (preinstalled on Windows 10/11)

### Commands

```sh
npm install        # installs the Tauri CLI
npm run dev        # starts the app in development mode (rebuilds on Rust changes)
npm run build      # release build plus installers
```

`npm run build` creates the installers in `src-tauri/target/release/bundle/`:

- `nsis/dewz_<version>_x64-setup.exe` – setup program (recommended)
- `msi/dewz_<version>_x64_en-US.msi` – MSI package, e.g. for managed installations

The installers are not code-signed, so Windows SmartScreen shows a warning on first install.

### Project structure

```
src/index.html                  the whole UI: markup, styles and script (German)
src-tauri/src/main.rs           Rust backend: folder dialog, reading/writing the data file, export
src-tauri/tauri.conf.json       window, bundle and Content Security Policy configuration
src-tauri/capabilities/         permissions of the web view
src-tauri/icons/                app icons, generated from dewz-logo.svg (npx tauri icon dewz-logo.svg -o src-tauri/icons)
tools/screenshot.mjs            screenshots of the app with sample data and an empty, temporary profile
dewz-logo.svg                   logo
Kontoauszug_*.xlsx              invented sample bank exports
```

### Notes for changes

- **CSP:** maintained only in `src-tauri/tauri.conf.json`. Keep the script and styles inline in `src/index.html`;
  Tauri hashes the `<script>` and adds a nonce to the `<style>` automatically. Tauri also allows `'self'`, so any
  additional `.js`/`.css` file placed in `src/` would be loadable – add files there only deliberately.
  `npm run dev` serves the page from Tauri's dev server **without** the CSP header; test CSP-related changes with
  `npm run build` or `tools/screenshot.mjs`, which both embed the page like the release.
- **Screenshots:** `node --experimental-websocket tools/screenshot.mjs --out shot.png --import Kontoauszug_Halbjahr.xlsx`
  (options: `--dark`, `--width`, `--height`, `--full`, `--eval "<js>"`). It builds its own debug exe in
  `src-tauri/target/screenshot` and starts it with a temporary profile, so your real data folder is never opened.
  The debug-only environment variable `DEWZ_SETTINGS_DIR` that makes this possible does not exist in release builds.
- **Never commit personal data.** `.gitignore` excludes data files, rule exports, CSV exports, bank exports and
  archives; only the two invented sample files are tracked. Check `git status` before every commit.

## Contributing

This is a personal project, and I don't accept pull requests. You're welcome to fork it and adapt it to your own
needs within the terms of the license.

## License

Copyright (C) 2026 binoema

dewz is free software: you can redistribute it and/or modify it under the terms of the
[GNU General Public License](LICENSE) as published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

dewz is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

In short: you may use, modify and share dewz, but if you distribute a modified version, you must publish its
source code under the same license.
