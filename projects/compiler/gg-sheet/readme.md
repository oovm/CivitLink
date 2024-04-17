# gg-sheet

GG 引擎的配置表管理工具，用于处理策划配置表并自动生成对应的数据访问脚本。

## 功能

- 支持 Excel 文件的解析
- 提取表结构和数据
- 自动生成 Valkyrie 脚本文件
- 监听配置表文件的变化
- 验证配置表数据的正确性

## 使用

```bash
gg-sheet init      # 初始化工作区
gg-sheet check     # 检查配置表
gg-sheet generate  # 生成 Valkyrie 脚本
gg-sheet watch     # 启用监听模式
```

## 依赖

- `calamine`: Excel 文件解析
- `csv`: CSV 文件处理
- `gg-core`: 核心类型定义
- `gg-compiler-core`: 编译器核心接口