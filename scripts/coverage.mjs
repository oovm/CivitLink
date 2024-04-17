import { spawn } from "node:child_process"
import { platform } from "node:os"
import { fileURLToPath } from "node:url"
import { dirname, join } from "node:path"

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const projectRoot = join(__dirname, "..")

const isWindows = platform() === "win32"
const shell = isWindows ? true : false

const colors = {
    reset: "\x1b[0m",
    red: "\x1b[31m",
    green: "\x1b[32m",
    yellow: "\x1b[33m",
    blue: "\x1b[34m",
    cyan: "\x1b[36m",
}

function log(color, message) {
    console.log(`${colors[color]}${message}${colors.reset}`)
}

function runCommand(command, args = [], options = {}) {
    return new Promise((resolve, reject) => {
        const child = spawn(command, args, {
            cwd: projectRoot,
            shell,
            stdio: "inherit",
            ...options,
        })

        child.on("close", (code) => {
            if (code === 0) {
                resolve(code)
            } else {
                reject(new Error(`Command failed with exit code ${code}`))
            }
        })

        child.on("error", (err) => {
            reject(err)
        })
    })
}

function checkCommandExists(command) {
    return new Promise((resolve) => {
        const child = spawn(command, ["--version"], {
            cwd: projectRoot,
            shell,
            stdio: "ignore",
        })

        child.on("close", (code) => {
            resolve(code === 0)
        })

        child.on("error", () => {
            resolve(false)
        })
    })
}

async function installCargoLlvmCov() {
    log("yellow", "cargo-llvm-cov 未安装，正在安装...")
    await runCommand("cargo", ["install", "cargo-llvm-cov"])
    log("green", "cargo-llvm-cov 安装成功！")
}

async function cleanCoverage() {
    log("cyan", "清理旧的覆盖率数据...")
    try {
        await runCommand("cargo", ["llvm-cov", "clean"])
    } catch (err) {
        log("yellow", "清理失败，可能没有旧数据，继续执行")
    }
}

async function runCoverage(outputType = "html", excludePackages = []) {
    log("cyan", "生成覆盖率报告...")
    
    const args = ["llvm-cov"]
    
    if (outputType === "html") {
        args.push("--html")
    } else if (outputType === "json") {
        args.push("--json")
    } else if (outputType === "lcov") {
        args.push("--lcov")
    }
    
    args.push("--workspace")
    
    for (const pkg of excludePackages) {
        args.push("--exclude", pkg)
    }
    
    await runCommand("cargo", args)
}

async function openReport(outputType) {
    if (outputType === "html") {
        log("cyan", "HTML 报告已生成在 target/llvm-cov/html/index.html")
    }
}

async function main() {
    log("blue", "\n╔══════════════════════════════════════════════════════════╗")
    log("blue", "║               WAE 测试覆盖率报告生成器                        ║")
    log("blue", "╚══════════════════════════════════════════════════════════╝\n")

    const args = process.argv.slice(2)
    const outputType = args[0] || "html"
    
    let excludePackages = []
    const excludeIndex = args.indexOf("--exclude")
    if (excludeIndex !== -1 && args.length > excludeIndex + 1) {
        excludePackages = args[excludeIndex + 1].split(",").map(p => p.trim())
    }

    try {
        const hasLlvmCov = await checkCommandExists("cargo-llvm-cov")
        if (!hasLlvmCov) {
            await installCargoLlvmCov()
        }

        await cleanCoverage()
        await runCoverage(outputType, excludePackages)
        await openReport(outputType)

        log("green", "\n✅ 覆盖率报告生成成功！\n")
        process.exit(0)
    } catch (err) {
        log("red", `\n❌ 错误: ${err.message}\n`)
        process.exit(1)
    }
}

main().catch((err) => {
    log("red", `\n致命错误: ${err.message}`)
    process.exit(1)
})
