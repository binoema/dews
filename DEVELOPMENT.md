# dewz – development

Technical notes for building and changing dewz. For using the app, see the [README](README.md).

## How it works

The app is a single self-contained HTML file (`src/index.html`, no build step, no frameworks) running in the system web view
(WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux), hosted by a small [Tauri 2](https://tauri.app) backend in Rust.
All UI text is kept in German and English in the `I18N` dictionary at the top of the script.

### Where data is stored

| What | Where |
|---|---|
| Bookings, categories, rules | `dewz-daten.json` in the folder the user chose (plain JSON or encrypted) |
| Remembered data folder | `einstellungen.json` in the app's config folder: `%APPDATA%\app.dewz.desktop` (Windows), `~/Library/Application Support/app.dewz.desktop` (macOS), `~/.config/app.dewz.desktop` (Linux) |
| UI settings (language, last view, column mappings, small-expense threshold, a copy of the categories) | the web view's local storage (on Windows the WebView2 profile in `%LOCALAPPDATA%\app.dewz.desktop`) |

Data files from the app's former name (`finanz-buddy-daten.json`) are read and migrated to `dewz-daten.json`;
the old file is left untouched.

### Security and privacy

dewz handles personal financial data, so it is built to have as little attack surface as possible:

- **No network.** A strict Content Security Policy (`default-src 'none'`) blocks every connection, external script,
  font and image. The only allowed channel is the IPC bridge to the app's own Rust backend. Inline scripts and styles
  are only executed with the hash/nonce Tauri adds at build time; `eval` is blocked.
- **Narrow file access.** The web UI has no general file system access. The Rust backend
  (`src-tauri/src/main.rs`) exposes only a few commands: it reads only folders the user picked in the system's folder dialog,
  writes only `dewz-daten.json` into the active folder, and writes exports only to the path chosen in the
  "Save as" dialog.
- **Atomic saves.** The data file is written to a temporary file first and then renamed, so a crash never leaves a
  half-written file.
- **Encryption stays in the UI.** Password and key never leave the web view; the backend only receives finished
  (encrypted) text. AES-256-GCM, key derived with PBKDF2-SHA-256, 600,000 iterations.
- **Minimal permissions.** The Tauri capability (`src-tauri/capabilities/default.json`) only allows listening to
  window events and closing the window.

## Building

### Prerequisites (Windows)

- [Node.js](https://nodejs.org) 20 or newer (only for the Tauri CLI)
- [Rust](https://rustup.rs) (stable, MSVC toolchain) – `winget install Rustlang.Rustup`
- Microsoft C++ Build Tools with the "Desktop development with C++" workload –
  `winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"`
- WebView2 Runtime (preinstalled on Windows 10/11)

### Prerequisites (macOS / Linux)

- Node.js 20 or newer and Rust (stable) as above
- macOS: Xcode Command Line Tools – `xcode-select --install`
- Linux (Debian/Ubuntu): `sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  – see the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for other distributions

### Commands

```sh
npm install        # installs the Tauri CLI
npm run dev        # starts the app in development mode (rebuilds on Rust changes)
npm run build      # release build plus installers
```

`npm run build` creates the installers for the current platform in `src-tauri/target/release/bundle/`:

- Windows: `nsis/dewz_<version>_x64-setup.exe` (setup program) and `msi/dewz_<version>_x64_en-US.msi`
  (e.g. for managed installations)
- macOS: `macos/dewz.app` and `dmg/dewz_<version>_<arch>.dmg`
- Linux: `appimage/`, `deb/` and `rpm/` packages

## Releases

`.github/workflows/release.yml` builds the installers for all platforms on GitHub Actions. To publish a release,
set the new version in `src-tauri/tauri.conf.json` (and `package.json`), then push a matching tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The workflow creates a draft release with all installers attached; review it on GitHub and click **Publish**.

## Project structure

```
src/index.html                  the whole UI: markup, styles, script and German/English texts
src-tauri/src/main.rs           Rust backend: folder dialog, reading/writing the data file, export
src-tauri/tauri.conf.json       window, bundle and Content Security Policy configuration
src-tauri/capabilities/         permissions of the web view
src-tauri/icons/                app icons, generated from dewz-logo.svg (npx tauri icon dewz-logo.svg -o src-tauri/icons)
tools/screenshot.mjs            screenshots of the app with sample data and an empty, temporary profile
dewz-logo.svg                   logo
Kontoauszug_*.xlsx              invented sample bank exports
```

## Notes for changes

- **CSP:** maintained only in `src-tauri/tauri.conf.json`. Keep the script and styles inline in `src/index.html`;
  Tauri hashes the `<script>` and adds a nonce to the `<style>` automatically. Tauri also allows `'self'`, so any
  additional `.js`/`.css` file placed in `src/` would be loadable – add files there only deliberately.
  `npm run dev` serves the page from Tauri's dev server **without** the CSP header; test CSP-related changes with
  `npm run build` or `tools/screenshot.mjs`, which both embed the page like the release.
- **Texts:** add every new UI text to both languages in `I18N` (`src/index.html`); static HTML uses `data-i18n`
  attributes, script code calls `T('key')`. Messages from the Rust backend use `tr(&lang, de, en)`.
- **Screenshots:** `node --experimental-websocket tools/screenshot.mjs --out shot.png --import Kontoauszug_Halbjahr.xlsx`
  (options: `--dark`, `--width`, `--height`, `--full`, `--eval "<js>"`). It builds its own debug exe in
  `src-tauri/target/screenshot` and starts it with a temporary profile, so your real data folder is never opened.
  The debug-only environment variable `DEWZ_SETTINGS_DIR` that makes this possible does not exist in release builds.
- **Never commit personal data.** `.gitignore` excludes data files, rule exports, CSV exports, bank exports and
  archives; only the two invented sample files are tracked. Check `git status` before every commit.

## Contributing

This is a personal project, and I don't accept pull requests. You're welcome to fork it and adapt it to your own
needs within the terms of the license.
