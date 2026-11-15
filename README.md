# dewz

**Kontoauszug auswerten – 100 % offline.**

dewz is a desktop app for Windows, macOS and Linux for analysing your own bank account exports. Import the Excel or CSV export from your
bank, let the bookings be categorised automatically, and see where your money goes – per category, per month and
over time. Nothing ever leaves your computer: the app has no network access at all.

The app is available in German and English. It starts in your system language; you can switch any time with the
language button in the top right corner.

## Download

Download the installer for your system from the [releases page](https://github.com/binoema/dewz/releases):

| System | File |
|---|---|
| Windows | `dewz_<version>_x64-setup.exe` (installer) or `dewz_<version>_x64-portable.zip` (no installation: unzip and start `dewz.exe`, e.g. from a USB stick) |
| macOS (Apple Silicon, M1 and newer) | `dewz_<version>_aarch64.dmg` |
| macOS (Intel) | `dewz_<version>_x64.dmg` |
| Linux | `.AppImage` (runs on any distribution), `.deb` (Debian/Ubuntu) or `.rpm` (Fedora/openSUSE) |

The installers are not signed, so your system will warn you the first time:

- **Windows:** SmartScreen shows "Windows protected your PC" – click **More info**, then **Run anyway**.
- **macOS:** right-click the app in the Applications folder and choose **Open**. If macOS says the app is damaged,
  run `xattr -cr /Applications/dewz.app` in the Terminal once.

See [Is dewz safe?](#is-dewz-safe) for what the warning means and how to check your download.

## Getting started

1. Start dewz and click **Choose data folder**. Pick any folder – e.g. in your documents, a cloud folder or a
   USB stick. dewz saves your data there and opens the folder automatically on the next start.
2. Export your transactions from your online banking as Excel (`.xlsx`) or CSV file.
3. Click **Import bank export** (or drag the files onto the window) and check the suggested column mapping.
4. Adjust the categories and rules under **Categories & rules** if needed.

Want to try it first? Download one of the two invented sample exports from this repository:
[Kontoauszug_Beispiel.xlsx](Kontoauszug_Beispiel.xlsx) (one month) or
[Kontoauszug_Halbjahr.xlsx](Kontoauszug_Halbjahr.xlsx) (six months).

## Features

- **Import** Excel (`.xlsx`) and CSV files from any bank. dewz suggests which column is the date, the amount, the
  recipient and so on, and remembers your choice for the next import. Several files can be imported at once;
  transactions you already imported are recognised and skipped. The old Excel format `.xls` is not supported –
  save it as `.xlsx` or CSV first.
- **Automatic categories** based on keywords in the recipient or purpose, e.g. "rewe" → Groceries. You can change the
  category of any transaction by hand, and turn such a correction into a rule with **Create rule …** so that similar
  transactions are sorted automatically from then on.
- **Sub-categories:** split a category into finer ones, e.g. "Mobility" into "Car" and "Public transport". The
  overviews add sub-categories up under their main category and show them with ▸ when you want the details.
- **Reserves:** mark categories such as "Savings" as transfers between your own accounts. They then don't count as
  income or expenses and are shown separately.
- **Views:** expenses by category, monthly overview, categories month by month (with unusual months highlighted),
  small expenses and the full list of transactions – all with filters for text, category, month, date range and amount.
- **Export** transactions and the summary as CSV for Excel, and your category rules to back them up or move them
  to another computer.
- **Light and dark theme:** dewz follows your system setting; switch any time with the ☾ / ☀ button in the top right
  corner.

## Your data

- Everything is stored in a single file, `dewz-daten.json`, in the folder you chose. Every change is saved
  automatically. To back up your data or move it to another computer, copy this file.
- You can protect the file with a **password** (in the **Data folder** dialog). It is encrypted and the password is
  asked for at every start. **If you forget the password, the data cannot be recovered.**
- dewz never connects to the internet. Your bank data stays on your computer.

## Is dewz safe?

The warning on first start only means that the installers are not signed with a paid certificate – it says nothing
about what the app does. Instead of a signature, you can check dewz yourself:

- **Open source:** the complete source code is in this repository. Anyone can read it or build the app themselves
  (see [DEVELOPMENT.md](DEVELOPMENT.md)).
- **Built in public:** the installers are not built on a private computer but by GitHub from this repository, using
  the [release workflow](.github/workflows/release.yml). Every build log is visible under
  [Actions](https://github.com/binoema/dewz/actions).
- **No network access:** the app's security policy blocks all connections to the internet. There is no telemetry,
  no account and no update check. Your data is only written to the folder you chose.

**Check your download (optional).** Each release contains `SHA256SUMS.txt` with a checksum for every file. Compare it
with the checksum of your download:

- Windows (PowerShell): `Get-FileHash .\dewz_<version>_x64-setup.exe`
- macOS / Linux: `shasum -a 256 dewz_<version>_aarch64.dmg`

With the [GitHub CLI](https://cli.github.com/) you can also verify that a file was built by this repository's workflow
and not changed afterwards:

```
gh attestation verify dewz_<version>_x64-setup.exe --repo binoema/dewz
```

## License

Copyright (C) 2026 binoema

dewz is free software under the [GNU General Public License v3](LICENSE) or later: you may use, modify and share it.
If you distribute a modified version, you must publish its source code under the same license. dewz comes without
any warranty.

Building from source: see [DEVELOPMENT.md](DEVELOPMENT.md).
