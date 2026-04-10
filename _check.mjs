import { writeFileSync, readFileSync, renameSync, existsSync } from "fs";

const cargoTomlPath = "e:\\灵之镜有限公司\\gg-game-engine\\projects\\core\\gg-asset\\Cargo.toml";
const bakPath = cargoTomlPath + ".bak";
const tmpToml = `[package]
name = "gg-asset"
version = "0.1.0"
edition = "2024"
authors = ["Lingames"]
license = "MIT"

[dependencies]
`;

const original = readFileSync(cargoTomlPath, "utf-8");
renameSync(cargoTomlPath, bakPath);
writeFileSync(cargoTomlPath, tmpToml, "utf-8");

try {
    const { execSync } = await import("child_process");
    const result = execSync("cargo check --manifest-path \"e:\\灵之镜有限公司\\gg-game-engine\\projects\\core\\gg-asset\\Cargo.toml\" 2>&1", {
        encoding: "utf-8",
        cwd: "e:\\灵之镜有限公司\\gg-game-engine\\projects\\core\\gg-asset",
    });
    console.log(result);
} catch (e) {
    console.log("STDOUT:", e.stdout);
    console.log("STDERR:", e.stderr);
} finally {
    renameSync(bakPath, cargoTomlPath);
}
