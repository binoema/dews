// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 binoema

// Screenshot der Desktop-App (Debug-Build) – mit leerem Profil, nie mit den echten Daten.
//
//   node --experimental-websocket tools/screenshot.mjs --out shot.png [Optionen]
//
// Baut vorher selbst (cargo build nach src-tauri/target/screenshot; beim ersten Mal ca. 2 Minuten).
//
// Optionen:
//   --out <datei.png>        Ziel (Pflicht)
//   --import <datei>         Kontoexport importieren und Spaltenzuordnung bestätigen (mehrfach möglich)
//   --eval <js>              JavaScript in der App ausführen, bevor fotografiert wird (z. B. Reiter anklicken)
//   --dark                   dunkles Farbschema
//   --width <px> --height <px>  Fenstergröße (Standard 1400 × 1000)
//   --full                   ganze Seite statt nur sichtbarem Bereich
//
// Die App startet mit eigenem WebView2-Profil und eigenem Einstellungsordner in einem Temp-Verzeichnis
// (DEWZ_SETTINGS_DIR wirkt nur im Debug-Build). Dadurch wird weder der gemerkte Datenordner geöffnet noch
// etwas am echten Profil verändert. Die DevTools-Schnittstelle lauscht nur auf 127.0.0.1.
import { spawn, spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir, homedir } from 'node:os';
import { join, resolve, dirname, delimiter } from 'node:path';
import { fileURLToPath } from 'node:url';

if (typeof WebSocket === 'undefined') throw new Error('Bitte mit "node --experimental-websocket" starten (ab Node 22 nicht mehr nötig).');

const args = process.argv.slice(2), opt = { import: [], eval: [] };
for (let i = 0; i < args.length; i++) {
  const k = args[i].replace(/^--/, '');
  if (k === 'dark' || k === 'full') opt[k] = true;
  else if (Array.isArray(opt[k])) opt[k].push(args[++i]);
  else opt[k] = args[++i];
}
if (!opt.out) throw new Error('--out <datei.png> fehlt.');
const width = Number(opt.width) || 1400, height = Number(opt.height) || 1000;

// Eigener Build in target/screenshot: enthält src/index.html fest eingebaut (mit CSP wie im Release) und
// wird nicht von "npm run dev" überschrieben, dessen Exe die Seite vom Dev-Server ohne CSP lädt.
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const env = Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith('TAURI_')));
env.PATH = join(homedir(), '.cargo', 'bin') + delimiter + (env.PATH || env.Path || '');
delete env.Path;
const build = spawnSync('cargo', ['build', '--quiet', '--target-dir', join('target', 'screenshot')],
  { cwd: join(root, 'src-tauri'), env, stdio: 'inherit', shell: true });
if (build.status !== 0) throw new Error('cargo build ist fehlgeschlagen.');
const exe = join(root, 'src-tauri', 'target', 'screenshot', 'debug', 'dewz.exe');
const port = 9300 + Math.floor(Math.random() * 500);
const tmp = mkdtempSync(join(tmpdir(), 'dewz-shot-'));
const app = spawn(exe, [], { stdio: 'ignore', env: { ...env,
  DEWZ_SETTINGS_DIR: join(tmp, 'settings'),
  WEBVIEW2_USER_DATA_FOLDER: join(tmp, 'webview'),
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port} --remote-debugging-address=127.0.0.1` } });
const sleep = ms => new Promise(r => setTimeout(r, ms));

try {
  let page;
  for (let i = 0; i < 60 && !page; i++) {
    await sleep(250);
    try { page = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find(t => t.type === 'page' && t.title === 'dewz'); } catch {}
  }
  if (!page) throw new Error('App hat keine DevTools-Schnittstelle geöffnet.');
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
  let n = 0; const waiting = new Map();
  ws.onmessage = m => { const d = JSON.parse(m.data); if (d.id) waiting.get(d.id)(d); };
  const send = (method, params = {}) => new Promise((r, j) => {
    const id = ++n; waiting.set(id, d => d.error ? j(new Error(`${method}: ${d.error.message}`)) : r(d.result));
    ws.send(JSON.stringify({ id, method, params }));
  });
  const run = async expr => {
    const r = await send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true });
    if (r.exceptionDetails) throw new Error('JS-Fehler: ' + (r.exceptionDetails.exception?.description || r.exceptionDetails.text));
    return r.result.value;
  };
  const until = async (expr, what) => {
    for (let i = 0; i < 80; i++) { if (await run(expr)) return; await sleep(100); }
    throw new Error('Zeitüberschreitung: ' + what);
  };

  await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
  await send('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: opt.dark ? 'dark' : 'light' }] });
  await until(`document.readyState === 'complete' && !!document.querySelector('#btnData')`, 'App geladen');

  for (const f of opt.import) {
    const { root: doc } = await send('DOM.getDocument');
    const { nodeId } = await send('DOM.querySelector', { nodeId: doc.nodeId, selector: '#file' });
    await send('DOM.setFileInputFiles', { nodeId, files: [resolve(f)] });
    await until(`document.querySelector('#dlgMap').open`, `Spaltenzuordnung für ${f}`);
    await run(`document.querySelector('#mapOk').click()`);
    await until(`!document.querySelector('#dlgMap').open`, `Import von ${f}`);
  }
  for (const js of opt.eval) await run(js);
  await sleep(400);   // Layout und Übergänge abwarten

  const shot = await send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: !!opt.full,
    ...(opt.full ? { clip: { x: 0, y: 0, width, height: await run('document.documentElement.scrollHeight'), scale: 1 } } : {}) });
  writeFileSync(opt.out, Buffer.from(shot.data, 'base64'));
  console.log('Gespeichert:', resolve(opt.out));
  ws.close();
} finally {
  app.kill();
  await sleep(500);
  try { rmSync(tmp, { recursive: true, force: true }); } catch {}
}
