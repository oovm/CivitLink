import { writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { deflateSync } from 'zlib';

const __dirname = dirname(fileURLToPath(import.meta.url));

function crc32(buf) {
    let crc = 0xffffffff;
    const table = new Int32Array(256);
    for (let i = 0; i < 256; i++) {
        let c = i;
        for (let j = 0; j < 8; j++) {
            c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
        }
        table[i] = c;
    }
    for (let i = 0; i < buf.length; i++) {
        crc = table[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
    }
    return (crc ^ 0xffffffff) >>> 0;
}

function createPNGChunk(type, data) {
    const typeBytes = Buffer.from(type, 'ascii');
    const length = Buffer.alloc(4);
    length.writeUInt32BE(data.length, 0);
    const crcInput = Buffer.concat([typeBytes, data]);
    const crcValue = Buffer.alloc(4);
    crcValue.writeUInt32BE(crc32(crcInput), 0);
    return Buffer.concat([length, typeBytes, data, crcValue]);
}

function createPNG(width, height, r, g, b, a) {
    const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

    const ihdrData = Buffer.alloc(13);
    ihdrData.writeUInt32BE(width, 0);
    ihdrData.writeUInt32BE(height, 4);
    ihdrData[8] = 8;
    ihdrData[9] = 6;
    ihdrData[10] = 0;
    ihdrData[11] = 0;
    ihdrData[12] = 0;
    const ihdr = createPNGChunk('IHDR', ihdrData);

    const rawRows = [];
    for (let y = 0; y < height; y++) {
        const row = Buffer.alloc(1 + width * 4);
        row[0] = 0;
        for (let x = 0; x < width; x++) {
            const idx = 1 + x * 4;
            row[idx] = r;
            row[idx + 1] = g;
            row[idx + 2] = b;
            row[idx + 3] = a;
        }
        rawRows.push(row);
    }
    const rawData = Buffer.concat(rawRows);
    const compressed = deflateSync(rawData);
    const idat = createPNGChunk('IDAT', compressed);

    const iend = createPNGChunk('IEND', Buffer.alloc(0));

    return Buffer.concat([signature, ihdr, idat, iend]);
}

const backgroundsDir = join(__dirname, 'assets', 'backgrounds');
const portraitsDir = join(__dirname, 'assets', 'portraits');

mkdirSync(backgroundsDir, { recursive: true });
mkdirSync(portraitsDir, { recursive: true });

const bgData = createPNG(1280, 720, 100, 149, 237, 255);
writeFileSync(join(backgroundsDir, 'bg_school.png'), bgData);
console.log('Created: assets/backgrounds/bg_school.png (1280x720, sky blue)');

const portraitData = createPNG(200, 400, 255, 182, 193, 255);
writeFileSync(join(portraitsDir, 'sakura_smile.png'), portraitData);
console.log('Created: assets/portraits/sakura_smile.png (200x400, pink)');

console.log('Placeholder image generation complete.');
