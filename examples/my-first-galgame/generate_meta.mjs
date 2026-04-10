import fs from 'fs';
import path from 'path';

// 模拟 gg-meta 的功能，生成 meta 文件
function generateMetaFile(filePath) {
    const stats = fs.statSync(filePath);
    const size = stats.size;
    const name = path.basename(filePath);
    const relativePath = path.relative('e:\\灵之镜有限公司\\gg-game-engine', filePath);
    const assetType = getAssetType(name);
    
    // 生成 UUID v7 格式的 GUID
    const guid = generateUUIDv7();
    const timestamp = new Date().toISOString();
    
    // 构建 meta 内容
    const metaContent = `MetaFile({
    version: "1.0",
    asset: Asset({
        type: "${assetType}",
        path: "${relativePath.replace(/\\/g, '/')}",
        guid: "${guid}",
        name: "${name}",
        size: ${size},
        modified: "${timestamp}",
    }),
    import_settings: None,
    dependencies: [],
    references: [],
    timestamp: "${timestamp}",
    hash: None,
})`;
    
    const metaPath = filePath + '.meta';
    fs.writeFileSync(metaPath, metaContent);
    console.log(`Generated meta file for ${filePath}`);
}

function getAssetType(filename) {
    const ext = path.extname(filename).toLowerCase();
    switch (ext) {
        case '.toml':
            return 'config';
        case '.gscript':
            return 'script';
        default:
            return 'unknown';
    }
}

// 生成 UUID v7
function generateUUIDv7() {
    const now = Date.now();
    const random = Math.floor(Math.random() * 0x10000000000000000).toString(16).padStart(16, '0');
    const timestamp = Math.floor(now / 1000).toString(16).padStart(12, '0');
    return `${timestamp.substring(0, 8)}-${timestamp.substring(8, 12)}-7${random.substring(1, 4)}-${(parseInt(random.substring(4, 5), 16) | 0x8).toString(16)}${random.substring(5, 8)}-${random.substring(8).padEnd(12, '0')}`;
}

// 处理目录中的文件
const directory = 'e:\\灵之镜有限公司\\gg-game-engine\\examples\\my-first-galgame';

// 处理 game.toml
const gameTomlPath = path.join(directory, 'game.toml');
if (fs.existsSync(gameTomlPath)) {
    generateMetaFile(gameTomlPath);
}

// 处理 scripts 目录
const scriptsDir = path.join(directory, 'scripts');
if (fs.existsSync(scriptsDir)) {
    const scriptFiles = fs.readdirSync(scriptsDir);
    for (const file of scriptFiles) {
        // 跳过 .meta 文件
        if (file.endsWith('.meta')) {
            continue;
        }
        const scriptPath = path.join(scriptsDir, file);
        if (fs.statSync(scriptPath).isFile()) {
            generateMetaFile(scriptPath);
        }
    }
}

console.log('Meta files generated successfully!');
