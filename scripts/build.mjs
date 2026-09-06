import fs from 'node:fs';
import path from 'node:path';
import AdmZip from 'adm-zip';

const ROOT_DIR = process.cwd();
const pkg = JSON.parse(fs.readFileSync(path.join(ROOT_DIR, 'package.json'), 'utf-8'));
const DIST_DIR = path.join(ROOT_DIR, 'dist');
const TEMPLATE_DIR = path.join(ROOT_DIR, 'template');
const DIST_APP_DIR = path.join(DIST_DIR, 'mkrp');
const ZIP_NAME = `mkrp-v${pkg.version}.zip`;
const DIST_ZIP_PATH = path.join(DIST_DIR, ZIP_NAME);

async function build() {
  console.log('========================================================');
  console.log(`[mkrp] Building PortMaster Package for Ren'Py Runner v${pkg.version}`);
  console.log('========================================================');

  // 1. Prepare directories
  fs.rmSync(DIST_DIR, { recursive: true, force: true });
  fs.mkdirSync(DIST_APP_DIR, { recursive: true });

  // 2. Copy Linux aarch64 binary if built
  const possibleBinaryPaths = [
    path.join(ROOT_DIR, 'target', 'aarch64-unknown-linux-gnu', 'release', 'mkrp'),
    path.join(ROOT_DIR, 'target', 'aarch64-unknown-linux-gnu', 'debug', 'mkrp'),
  ];

  let binaryCopied = false;
  for (const binPath of possibleBinaryPaths) {
    if (fs.existsSync(binPath)) {
      fs.copyFileSync(binPath, path.join(DIST_APP_DIR, 'mkrp'));
      console.log(`[mkrp] Included Linux aarch64 binary from: ${binPath}`);
      binaryCopied = true;
      break;
    }
  }

  if (!binaryCopied) {
    console.log('[mkrp] Note: Native binary not yet built. Packaging templates & metadata.');
  }

  // 3. Copy template files
  const templateFiles = ['port.json', 'keymap.gptk'];
  for (const file of templateFiles) {
    const src = path.join(TEMPLATE_DIR, file);
    const dest = path.join(DIST_APP_DIR, file);
    if (fs.existsSync(src)) {
      fs.copyFileSync(src, dest);
    }
  }

  // Copy template directories
  const templateDirs = ['fonts', 'conf', 'lib'];
  for (const dirName of templateDirs) {
    const srcDir = path.join(TEMPLATE_DIR, dirName);
    const destDir = path.join(DIST_APP_DIR, dirName);
    if (fs.existsSync(srcDir)) {
      fs.cpSync(srcDir, destDir, { recursive: true });
    }
  }

  // Copy launcher to dist root with strict Unix LF line endings
  const launcherSrc = path.join(TEMPLATE_DIR, 'mkrp.sh');
  const launcherDest = path.join(DIST_DIR, 'mkrp.sh');
  let launcherContent = fs.readFileSync(launcherSrc, 'utf8');
  if (launcherContent.charCodeAt(0) === 0xFEFF) {
    launcherContent = launcherContent.slice(1);
  }
  launcherContent = launcherContent.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  fs.writeFileSync(launcherDest, launcherContent, { encoding: 'utf8', flag: 'w' });

  // Ensure game and saves directories with .gitkeep
  const gameDir = path.join(DIST_APP_DIR, 'game');
  const savesDir = path.join(DIST_APP_DIR, 'saves');
  fs.mkdirSync(gameDir, { recursive: true });
  fs.mkdirSync(savesDir, { recursive: true });
  fs.writeFileSync(path.join(gameDir, '.gitkeep'), '');
  fs.writeFileSync(path.join(savesDir, '.gitkeep'), '');

  // 4. Create PortMaster distribution zip
  console.log(`[mkrp] Packaging into dist/${ZIP_NAME}...`);
  const distZip = new AdmZip();
  distZip.addLocalFile(launcherDest);

  const howToUseSrc = path.join(ROOT_DIR, 'HOW_TO_USE.md');
  if (fs.existsSync(howToUseSrc)) {
    distZip.addLocalFile(howToUseSrc);
  }
  const licenseSrc = path.join(ROOT_DIR, 'LICENSE');
  if (fs.existsSync(licenseSrc)) {
    distZip.addLocalFile(licenseSrc);
  }

  distZip.addLocalFolder(DIST_APP_DIR, 'mkrp');
  distZip.writeZip(DIST_ZIP_PATH);

  const stat = fs.statSync(DIST_ZIP_PATH);
  console.log('========================================================');
  console.log(`[mkrp] Distribution package created successfully!`);
  console.log(`[mkrp] Output zip: ${DIST_ZIP_PATH}`);
  console.log(`[mkrp] Size: ${(stat.size / 1024).toFixed(2)} KB`);
  console.log('========================================================');
}

build().catch(err => {
  console.error('[mkrp] Build failed:', err);
  process.exit(1);
});
