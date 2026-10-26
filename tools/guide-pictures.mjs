// The user guide's pictures, taken from the real program: a fresh program folder with the built
// `Packing.exe` is started with WebView2's DevTools port open, driven through a batch of the
// made-up `testdata/` labels, and each screen is captured into `docs/user-guide/images/`.
//
//   bash tools/make-portable.sh && node tools/guide-pictures.mjs
//
// Windows only (WebView2). Nothing of the owner's is touched: the program folder is a new one in
// the system's temporary folder, removed at the end.

import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const ROOT = path.resolve(import.meta.dirname, "..");
const IMAGES = path.join(ROOT, "docs", "user-guide", "images");
const PORT = 9222;
const LABELS = ["labels-repeat-1.pdf", "labels-repeat-2.pdf"].map((name) => path.join(ROOT, "testdata", name));

const program = fs.mkdtempSync(path.join(os.tmpdir(), "packing-pictures-"));
for (const file of ["Packing.exe", "pdfium.dll"]) {
  fs.copyFileSync(path.join(ROOT, "target", "portable", "Packing", file), path.join(program, file));
}
const app = spawn(path.join(program, "Packing.exe"), [], {
  env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` },
  stdio: "ignore",
});

const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function connect() {
  for (let tries = 0; tries < 60; tries += 1) {
    try {
      const targets = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json();
      const target = targets.find((item) => item.type === "page");
      if (target) return new WebSocket(target.webSocketDebuggerUrl);
    } catch {
      // The window is not up yet.
    }
    await wait(500);
  }
  throw new Error("the program's window did not open a DevTools port");
}

const socket = await connect();
await new Promise((resolve, reject) => {
  socket.onopen = resolve;
  socket.onerror = reject;
});
let next = 0;
const replies = new Map();
socket.onmessage = (event) => {
  const message = JSON.parse(event.data);
  replies.get(message.id)?.(message);
  replies.delete(message.id);
};
const send = (method, params = {}) =>
  new Promise((resolve) => {
    next += 1;
    replies.set(next, resolve);
    socket.send(JSON.stringify({ id: next, method, params }));
  });

async function run(expression) {
  const reply = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  if (reply.result?.exceptionDetails) throw new Error(JSON.stringify(reply.result.exceptionDetails));
  return reply.result?.result?.value;
}

const shows = (text) => `document.body.innerText.includes(${JSON.stringify(text)})`;

async function until(expression) {
  for (let tries = 0; tries < 240; tries += 1) {
    if (await run(expression)) return;
    await wait(250);
  }
  throw new Error(`timed out waiting for ${expression}`);
}

// Click the first enabled element matching `selector` whose text starts with `text`.
async function click(text, selector = "button") {
  const clicked = await run(`(() => {
    const found = [...document.querySelectorAll(${JSON.stringify(selector)})]
      .find((item) => item.textContent.trim().startsWith(${JSON.stringify(text)}) && !item.disabled);
    found?.click();
    return Boolean(found);
  })()`);
  if (!clicked) throw new Error(`nothing to click: ${text}`);
  await wait(400);
}

async function picture(name) {
  const reply = await send("Page.captureScreenshot", { format: "png" });
  fs.writeFileSync(path.join(IMAGES, name), Buffer.from(reply.result.data, "base64"));
  console.log(`wrote docs/user-guide/images/${name}`);
}

try {
  await send("Runtime.enable");
  await until(shows("Where should the data be kept?"));
  await picture("01-start.png");
  await click("Use the folder next to the program");
  await until(shows("Name the first namespace"));
  await run(`(() => {
    const input = document.querySelector("input");
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set.call(input, "Toko Contoh");
    input.dispatchEvent(new Event("input", { bubbles: true }));
  })()`);
  await click("Use this name");
  await until(shows("Nothing saved on this day yet."));

  await click("+ New batch");
  // A drop on the window, as Explorer sends it.
  await run(`window.__TAURI_INTERNALS__.invoke("plugin:event|emit", {
    event: "tauri://drag-drop",
    payload: { type: "drop", paths: ${JSON.stringify(LABELS)}, position: { x: 10, y: 10 } },
  })`);
  await until(shows("labels-repeat-2.pdf"));
  await picture("02-new-batch.png");
  await click("Read labels");
  await until(shows("Repeated orders in this batch"));
  await click("▸");
  await picture("03-repeats.png");
  await click("Continue to the plan");
  await until(shows("Save batch"));
  await click("▸");
  await picture("04-plan.png");
  await click("Save batch");
  await until(shows("Batch 1 saved"));
  await click("Left out: ");
  await picture("05-day.png");

  await click("Search");
  await until(shows("This day"));
  await click("Search", ".search .controls button");
  await until(shows("orders in 1 batch"));
  await run(`document.querySelector(".search tbody input[type=checkbox]").click()`);
  await run(`document.querySelectorAll(".search td.tools button")[1].click()`);
  await wait(400);
  await picture("06-search.png");
  await click("Close");

  await click("Sort rules");
  await until(shows("Order matters"));
  await picture("07-sort-rules.png");
} finally {
  socket.close();
  app.kill();
  await wait(1000);
  fs.rmSync(program, { recursive: true, force: true });
}
