import { copyFile, mkdir, readFile, writeFile, readdir, cp, rename } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { existsSync } from 'node:fs';

const baseDir = 'e:\\灵之镜有限公司\\gwg';

async function moveDir(src, dest) {
  if (!existsSync(src)) {
    console.log(`- 源不存在: ${src}`);
    return;
  }
  
  const destDir = dirname(dest);
  if (!existsSync(destDir)) {
    await mkdir(destDir, { recursive: true });
  }
  
  await cp(src, dest, { recursive: true });
  console.log(`✓ 移动: ${src} -> ${dest}`);
}

async function main() {
  console.log('开始移动 frameworks 到 core...\n');
  
  const modules = [
    'gwg-asset',
    'gwg-ecs',
    'gwg-reflection',
    'gwg-schedule',
    'gwg-types',
    'gwg-vm',
    'gwg-world'
  ];
  
  for (const module of modules) {
    const src = join(baseDir, 'crates', 'frameworks', module);
    const dest = join(baseDir, 'crates', 'core', module.replace('gwg-', ''));
    await moveDir(src, dest);
  }
  
  console.log('\n创建 core 父 crate...\n');
  
  // 创建 core/Cargo.toml
  const coreCargoContent = `[package]
name = "gwg-core"
version = "0.1.0"
edition = "2021"
description = "GWG Engine 核心框架"
license = "MIT"

[dependencies]
gwg-core-asset = { path = "asset", version = "0.1.0" }
gwg-core-ecs = { path = "ecs", version = "0.1.0" }
gwg-core-reflection = { path = "reflection", version = "0.1.0" }
gwg-core-schedule = { path = "schedule", version = "0.1.0" }
gwg-core-types = { path = "types", version = "0.1.0" }
gwg-core-vm = { path = "vm", version = "0.1.0" }
gwg-core-world = { path = "world", version = "0.1.0" }
`;
  
  const coreCargoPath = join(baseDir, 'crates', 'core', 'Cargo.toml');
  await writeFile(coreCargoPath, coreCargoContent, 'utf-8');
  console.log('✓ 创建: crates/core/Cargo.toml');
  
  // 创建 core/src/lib.rs
  const coreLibContent = `//! GWG Engine 核心框架
//! 
//! 提供引擎核心功能的统一入口，重新导出所有子模块。

pub use gwg_core_asset as asset;
pub use gwg_core_ecs as ecs;
pub use gwg_core_reflection as reflection;
pub use gwg_core_schedule as schedule;
pub use gwg_core_types as types;
pub use gwg_core_vm as vm;
pub use gwg_core_world as world;

pub mod prelude {
    //! 常用类型和 trait 的预导入模块

    pub use gwg_core_asset::prelude::*;
    pub use gwg_core_ecs::prelude::*;
    pub use gwg_core_reflection::prelude::*;
    pub use gwg_core_schedule::prelude::*;
    pub use gwg_core_types::prelude::*;
    pub use gwg_core_vm::prelude::*;
    pub use gwg_core_world::prelude::*;
}
`;
  
  const coreSrcDir = join(baseDir, 'crates', 'core', 'src');
  if (!existsSync(coreSrcDir)) {
    await mkdir(coreSrcDir, { recursive: true });
  }
  const coreLibPath = join(coreSrcDir, 'lib.rs');
  await writeFile(coreLibPath, coreLibContent, 'utf-8');
  console.log('✓ 创建: crates/core/src/lib.rs');
  
  console.log('\n更新子模块的包名...\n');
  
  // 更新子模块的包名
  for (const module of modules) {
    const shortName = module.replace('gwg-', '');
    const cargoPath = join(baseDir, 'crates', 'core', shortName, 'Cargo.toml');
    if (existsSync(cargoPath)) {
      let content = await readFile(cargoPath, 'utf-8');
      content = content.replace(`name = "${module}"`, `name = "gwg-core-${shortName}"`);
      await writeFile(cargoPath, content, 'utf-8');
      console.log(`✓ 更新: crates/core/${shortName}/Cargo.toml`);
    }
  }
  
  console.log('\n完成！frameworks 已成功移动到 core。');
}

main().catch(console.error);
