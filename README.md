# dewz

**Kontoauszug auswerten – 100 % offline.**

dewz is a desktop app for Windows, macOS and Linux for analysing your own bank account exports. Import the Excel or CSV export from your
bank, let the bookings be categorised automatically, and see where your money goes – per category, per month and
over time. Nothing ever leaves your computer: the app has no network access at all.

The app is available in German and English. It starts in your system language; you can switch any time with the
language button in the top right corner.

## Download

Download the installer for your system from the [releases page](https://github.com/binoema/dews/releases):

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
- **Reserves:** mark categories such as "Savings" as transfers between your own accounts. They then don't count as
  income or expenses and are shown separately.
- **Views:** expenses by category, monthly overview, categories month by month (with unusual months highlighted),
  small expenses and the full list of transactions – all with filters for text, category, month, date range and amount.
- **Export** transactions and the summary as CSV for Excel, and your category rules to back them up or move them
  to another computer.

## Your data

- Everything is stored in a single file, `dewz-daten.json`, in the folder you chose. Every change is saved
  automatically. To back up your data or move it to another computer, copy this file.
- You can protect the file with a **password** (in the **Data folder** dialog). It is encrypted and the password is
  asked for at every start. **If you forget the password, the data cannot be recovered.**
- dewz never connects to the internet. Your bank data stays on your computer.

## License

Copyright (C) 2026 binoema

dewz is free software under the [GNU General Public License v3](LICENSE) or later: you may use, modify and share it.
If you distribute a modified version, you must publish its source code under the same license. dewz comes without
any warranty.

Building from source: see [DEVELOPMENT.md](DEVELOPMENT.md).
