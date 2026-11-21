// Screenshot any local page at an exact CSS viewport over CDP.
// Usage: node page-shot.mjs <url> <width> <height> <out.png> [dpr]
import { writeFileSync } from "node:fs";

const [url, widthArg, heightArg, out, dprArg] = process.argv.slice(2);
const width = Number(widthArg);
const height = Number(heightArg);
const dpr = Number(dprArg ?? 1);
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const created = await fetch(
  `http://127.0.0.1:9222/json/new?${encodeURIComponent("about:blank")}`,
  { method: "PUT" },
).then((response) => response.json());

const ws = new WebSocket(created.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  ws.addEventListener("open", resolve, { once: true });
  ws.addEventListener("error", reject, { once: true });
});

let id = 0;
const pending = new Map();
ws.addEventListener("message", (event) => {
  const message = JSON.parse(event.data);
  if (message.id === undefined) return;
  const slot = pending.get(message.id);
  if (!slot) return;
  pending.delete(message.id);
  message.error ? slot.reject(new Error(JSON.stringify(message.error))) : slot.resolve(message.result);
});

function send(method, params = {}) {
  const call = ++id;
  ws.send(JSON.stringify({ id: call, method, params }));
  return new Promise((resolve, reject) => pending.set(call, { resolve, reject }));
}

await send("Page.enable");
await send("Emulation.setDeviceMetricsOverride", {
  width,
  height,
  deviceScaleFactor: dpr,
  // Desktop viewports only.
  screenWidth: width,
  screenHeight: height,
});
await send("Page.navigate", { url });
await sleep(3500);
const { data } = await send("Page.captureScreenshot", { format: "png" });
writeFileSync(out, Buffer.from(data, "base64"));
const expression = process.argv[7];
if (expression) {
  await send("Runtime.enable");
  const result = await send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  console.log(JSON.stringify(result.result?.value ?? result.exceptionDetails ?? null, null, 2));
}
console.log(JSON.stringify({ url, width, height, out }));
ws.close();
