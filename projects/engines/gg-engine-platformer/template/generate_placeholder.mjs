import { writeFileSync, mkdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { deflateSync } from "zlib";

const __dirname = dirname(fileURLToPath(import.meta.url));

function createPng(width, height, r, g, b) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

  const ihdrData = Buffer.alloc(13);
  ihdrData.writeUInt32BE(width, 0);
  ihdrData.writeUInt32BE(height, 4);
  ihdrData[8] = 8;
  ihdrData[9] = 2;
  ihdrData[10] = 0;
  ihdrData[11] = 0;
  ihdrData[12] = 0;
  const ihdr = makeChunk("IHDR", ihdrData);

  const rawRow = Buffer.alloc(1 + width * 3);
  rawRow[0] = 0;
  for (let x = 0; x < width; x++) {
    rawRow[1 + x * 3] = r;
    rawRow[2 + x * 3] = g;
    rawRow[3 + x * 3] = b;
  }
  const rawData = Buffer.alloc(rawRow.length * height);
  for (let y = 0; y < height; y++) {
    rawRow.copy(rawData, y * rawRow.length);
  }

  const compressed = deflateSync(rawData);
  const idat = makeChunk("IDAT", compressed);

  const iend = makeChunk("IEND", Buffer.alloc(0));

  return Buffer.concat([signature, ihdr, idat, iend]);
}

function makeChunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const typeBuffer = Buffer.from(type, "ascii");
  const crcInput = Buffer.concat([typeBuffer, data]);
  const crc = crc32(crcInput);
  const crcBuffer = Buffer.alloc(4);
  crcBuffer.writeUInt32BE(crc >>> 0, 0);
  return Buffer.concat([length, typeBuffer, data, crcBuffer]);
}

function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc ^= buf[i];
    for (let j = 0; j < 8; j++) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
  }
  return crc ^ 0xffffffff;
}

const assetsDir = join(__dirname, "assets");
mkdirSync(assetsDir, { recursive: true });

const playerPng = createPng(32, 32, 0, 120, 255);
writeFileSync(join(assetsDir, "player.png"), playerPng);

const platformPng = createPng(64, 16, 80, 180, 80);
writeFileSync(join(assetsDir, "platform.png"), platformPng);

const collectiblePng = createPng(16, 16, 255, 215, 0);
writeFileSync(join(assetsDir, "collectible.png"), collectiblePng);

console.log("Placeholder assets generated in:", assetsDir);
