import { createServer } from "node:http";
import { existsSync, readFileSync, statSync } from "node:fs";
import { extname, join, normalize, resolve } from "node:path";
import { createRequire } from "node:module";
import { chromium } from "playwright-core";

const require = createRequire(import.meta.url);
const staticDirectory = resolve(import.meta.dirname, "../../backend/apps/api-gateway/static");
const pages = [
  "home.html",
  "login.html",
  "setup.html",
  "reset-password.html",
  "index.html",
  "customer.html",
  "mobile-admin.html",
  "mobile-courier.html",
  "mobile-customer.html",
  "warehouse.html",
  "field-service.html",
];

const contentTypes = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".webmanifest": "application/manifest+json; charset=utf-8",
};

function browserExecutable() {
  const candidates = [
    process.env.QERVON_CHROME_BIN,
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
  ].filter(Boolean);
  const executable = candidates.find(existsSync);
  if (!executable) throw new Error("Chrome/Chromium executable was not found");
  return executable;
}

const server = createServer((request, response) => {
  const requestPath = request.url === "/" ? "/home.html" : request.url.split("?", 1)[0];
  const relativePath = normalize(decodeURIComponent(requestPath)).replace(/^[/\\]+/, "");
  const filePath = resolve(join(staticDirectory, relativePath));
  if (!filePath.startsWith(`${staticDirectory}/`) || !existsSync(filePath) || !statSync(filePath).isFile()) {
    response.writeHead(404).end("Not found");
    return;
  }
  response.writeHead(200, { "content-type": contentTypes[extname(filePath)] ?? "application/octet-stream" });
  response.end(readFileSync(filePath));
});

await new Promise((resolveListening) => server.listen(0, "127.0.0.1", resolveListening));
const address = server.address();
const baseURL = `http://127.0.0.1:${address.port}`;
const browser = await chromium.launch({ executablePath: browserExecutable(), headless: true });
let failures = 0;

try {
  for (const pageName of pages) {
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    await page.goto(`${baseURL}/${pageName}`, { waitUntil: "domcontentloaded" });
    await page.addScriptTag({ path: require.resolve("axe-core/axe.min.js") });
    const results = await page.evaluate(async () => globalThis.axe.run(document, {
      runOnly: {
        type: "tag",
        values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"],
      },
    }));
    const blocking = results.violations.filter((violation) =>
      violation.impact === "critical" || violation.impact === "serious"
    );
    if (blocking.length > 0) {
      failures += blocking.length;
      console.error(`${pageName}: ${blocking.length} blocking accessibility violation(s)`);
      for (const violation of blocking) {
        console.error(`  ${violation.id}: ${violation.help} (${violation.nodes.length} node(s))`);
        for (const node of violation.nodes) {
          console.error(`    ${node.target.join(" ")}: ${node.failureSummary ?? node.html}`);
        }
      }
    } else {
      console.log(`${pageName}: WCAG A/AA serious-critical gate passed`);
    }
    await page.close();
  }
} finally {
  await browser.close();
  await new Promise((resolveClosed) => server.close(resolveClosed));
}

if (failures > 0) process.exitCode = 1;
