import sharp from "sharp";
import { mkdir, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join } from "node:path";

const ROOT = process.cwd();
const PUBLIC = join(ROOT, "public");
const ICONS = join(ROOT, "src-tauri", "icons");
const IOS = join(ROOT, "ios", "Runner", "Assets.xcassets", "AppIcon.appiconset");
const ANDROID_RES = join(ROOT, "android", "app", "src", "main", "res");

const SVG = join(PUBLIC, "logo.svg");
const CREAM = "#f3efe6";

async function ensureDir(d) {
  if (!existsSync(d)) await mkdir(d, { recursive: true });
}

const svg = await import("node:fs/promises").then((m) => m.readFile(SVG));

async function makePng(size, out, opts = {}) {
  const density = size >= 1024 ? 96 : size >= 512 ? 192 : 300;
  await sharp(svg, { density })
    .resize(size, size, { fit: "contain", background: CREAM })
    .png()
    .toFile(out);
  console.log(`  ${size}x${size} → ${out.replace(ROOT, ".")}`);
}

console.log("Web favicons:");
await makePng(16, join(PUBLIC, "favicon-16.png"));
await makePng(32, join(PUBLIC, "favicon-32.png"));
await makePng(180, join(PUBLIC, "apple-touch-icon.png"));
await makePng(192, join(PUBLIC, "icon-192.png"));
await makePng(512, join(PUBLIC, "icon-512.png"));

console.log("Tauri icon (1024 source for tauri-cli):");
const tauriSource = join(ROOT, "app-icon.png");
await makePng(1024, tauriSource);

console.log("iOS AppIcon 1024:");
await ensureDir(IOS);
await makePng(1024, join(IOS, "AppIcon-1024.png"));

console.log("Android launcher icons:");
const dirs = ["mipmap-mdpi", "mipmap-hdpi", "mipmap-xhdpi", "mipmap-xxhdpi", "mipmap-xxxhdpi"];
const androidSizes = {
  "mipmap-mdpi": 48,
  "mipmap-hdpi": 72,
  "mipmap-xhdpi": 96,
  "mipmap-xxhdpi": 144,
  "mipmap-xxxhdpi": 192,
};
for (const d of dirs) {
  const p = join(ANDROID_RES, d);
  if (!existsSync(p)) continue;
  const s = androidSizes[d];
  await makePng(s, join(p, "ic_launcher.png"));
  await makePng(s, join(p, "ic_launcher_round.png"));
  await makePng(s * 2, join(p, "ic_launcher_foreground.png"));
}

console.log("PWA manifest:");
const manifest = {
  name: "CalcTimers",
  short_name: "CalcTimers",
  description: "Calculator and multi-timer utility",
  start_url: "/",
  display: "standalone",
  background_color: CREAM,
  theme_color: CREAM,
  icons: [
    { src: "/icon-192.png", sizes: "192x192", type: "image/png" },
    { src: "/icon-512.png", sizes: "512x512", type: "image/png" },
  ],
};
await writeFile(join(PUBLIC, "site.webmanifest"), JSON.stringify(manifest, null, 2));
console.log("  manifest → public/site.webmanifest");

console.log("Done. Now run: pnpm tauri icon app-icon.png -o src-tauri/icons");
