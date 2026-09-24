import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const sharp = require('sharp');

/**
 * mkrp Asset Optimization Toolkit
 * Downscales and compresses PC Ren'Py high-res image assets (1080p/1440p)
 * to handheld-friendly formats (720p/480p) to slash RAM usage and eliminate I/O lag.
 */

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    inputDir: '',
    outputDir: '',
    maxWidth: 1280,
    maxHeight: 720,
    quality: 82,
    format: 'webp', // 'webp', 'png', 'keep'
    overwrite: false,
    help: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--help' || arg === '-h') {
      options.help = true;
    } else if (arg === '--input' || arg === '-i') {
      options.inputDir = args[++i];
    } else if (arg === '--output' || arg === '-o') {
      options.outputDir = args[++i];
    } else if (arg === '--max-width') {
      options.maxWidth = parseInt(args[++i], 10);
    } else if (arg === '--max-height') {
      options.maxHeight = parseInt(args[++i], 10);
    } else if (arg === '--quality' || arg === '-q') {
      options.quality = parseInt(args[++i], 10);
    } else if (arg === '--format' || arg === '-f') {
      options.format = args[++i].toLowerCase();
    } else if (arg === '--overwrite') {
      options.overwrite = true;
    } else if (!options.inputDir) {
      options.inputDir = arg;
    }
  }

  return options;
}

function printHelp() {
  console.log(`
Usage: node scripts/optimize-assets.mjs [options] <inputDir>

Options:
  -i, --input <dir>       Path to source directory containing images
  -o, --output <dir>      Path to output directory (defaults to in-place or ./optimized)
  --max-width <pixels>    Maximum width for downscaling (default: 1280)
  --max-height <pixels>   Maximum height for downscaling (default: 720)
  -q, --quality <1-100>   Compression quality (default: 82)
  -f, --format <format>   Output image format: 'webp', 'png', or 'keep' (default: 'webp')
  --overwrite             Allow overwriting existing files in place
  -h, --help              Show this help message

Examples:
  pnpm run optimize --input /path/to/game/images --max-width 1280
  pnpm run optimize -i ./raw_cg -o ./compressed_cg -f webp -q 80
`);
}

function findImages(dir, fileList = []) {
  const supportedExts = new Set(['.png', '.jpg', '.jpeg', '.webp', '.avif', '.bmp']);
  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      findImages(fullPath, fileList);
    } else if (entry.isFile()) {
      const ext = path.extname(entry.name).toLowerCase();
      if (supportedExts.has(ext)) {
        fileList.push(fullPath);
      }
    }
  }
  return fileList;
}

async function optimize() {
  const options = parseArgs();

  if (options.help || !options.inputDir) {
    printHelp();
    return;
  }

  const inputDir = path.resolve(options.inputDir);
  if (!fs.existsSync(inputDir)) {
    console.error(`Error: Input directory does not exist: ${inputDir}`);
    process.exit(1);
  }

  const outputDir = options.outputDir
    ? path.resolve(options.outputDir)
    : options.overwrite
    ? inputDir
    : path.resolve('dist/optimized_assets');

  console.log('========================================================');
  console.log(`[mkrp Asset Optimizer]`);
  console.log(`Input Directory:  ${inputDir}`);
  console.log(`Output Directory: ${outputDir}`);
  console.log(`Target Bounds:    Max ${options.maxWidth}x${options.maxHeight}`);
  console.log(`Target Format:    ${options.format.toUpperCase()} (Quality: ${options.quality})`);
  console.log('========================================================');

  const imageFiles = findImages(inputDir);
  console.log(`Found ${imageFiles.length} image files to process.\n`);

  if (imageFiles.length === 0) {
    console.log('No supported image files found.');
    return;
  }

  let totalOriginalSize = 0;
  let totalOptimizedSize = 0;
  let processedCount = 0;

  for (const file of imageFiles) {
    const relPath = path.relative(inputDir, file);
    const origExt = path.extname(file).toLowerCase();
    const targetExt = options.format === 'keep' ? origExt : `.${options.format}`;
    const outFileName = relPath.slice(0, -origExt.length) + targetExt;
    const destPath = path.join(outputDir, outFileName);

    fs.mkdirSync(path.dirname(destPath), { recursive: true });

    try {
      const origStat = fs.statSync(file);
      totalOriginalSize += origStat.size;

      const pipeline = sharp(file);
      const meta = await pipeline.metadata();

      // Only downscale if original is larger than max bounds
      if (meta.width > options.maxWidth || meta.height > options.maxHeight) {
        pipeline.resize({
          width: options.maxWidth,
          height: options.maxHeight,
          fit: 'inside',
          withoutEnlargement: true,
        });
      }

      if (options.format === 'webp') {
        pipeline.webp({ quality: options.quality, effort: 4 });
      } else if (options.format === 'png') {
        pipeline.png({ compressionLevel: 8 });
      }

      await pipeline.toFile(destPath);
      const optStat = fs.statSync(destPath);
      totalOptimizedSize += optStat.size;

      processedCount++;
      if (processedCount % 10 === 0 || processedCount === imageFiles.length) {
        process.stdout.write(`\r[mkrp] Progress: ${processedCount}/${imageFiles.length} images processed...`);
      }
    } catch (err) {
      console.error(`\nFailed to process ${relPath}:`, err.message);
    }
  }

  console.log('\n\n========================================================');
  console.log(`Optimization Complete!`);
  console.log(`Processed:     ${processedCount} images`);
  console.log(`Original Size: ${(totalOriginalSize / (1024 * 1024)).toFixed(2)} MB`);
  console.log(`New Size:      ${(totalOptimizedSize / (1024 * 1024)).toFixed(2)} MB`);
  const savings = totalOriginalSize > 0 ? ((1 - totalOptimizedSize / totalOriginalSize) * 100).toFixed(1) : 0;
  console.log(`Total Savings: ${savings}%`);
  console.log('========================================================');
}

optimize().catch(err => {
  console.error('[mkrp] Optimizer error:', err);
  process.exit(1);
});
