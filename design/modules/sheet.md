# GG-Sheet 配置表管理工具设计

## 概述
GG-Sheet 是一个为 GG Game Engine 设计的配置表管理工具，用于处理策划配置表并自动生成对应的数据访问脚本。

## 目录结构
- **/asset/sheet**: 存放策划配置表文件（Excel 格式）
- **/asset/script/table**: 存放自动生成的 Valkyrie 脚本文件

## 核心功能

### 1. 配置表解析
- 支持 Excel 文件的解析
- 提取表结构和数据
- 处理不同类型的单元格数据
- 支持正确的表头格式（第一行注释，第二行名称，第三行类型）

### 2. 代码生成
- 自动生成 Valkyrie 脚本文件，命名为具体表名 + Table.v
- 生成的数据访问代码符合 Valkyrie 语法
- 支持配置表的字段类型推断
- 支持基本类型和复合类型

### 3. 自动监听
- 监听配置表文件的变化
- 文件修改时自动重新生成脚本
- 优化监听性能，避免不必要的重新生成

### 4. 命令行接口
- 支持初始化、检查、生成等操作
- 简洁明了的命令行参数
- 详细的错误报告

### 5. 配置表验证
- 验证配置表数据的正确性
- 检测数据类型、格式、引用关系等错误
- 提供详细的错误报告

## 技术实现

### 语言和依赖
- 使用 Rust 语言实现
- 依赖 calamine 库解析 Excel 文件
- 依赖 notify 库实现文件监听

## 命令行接口

### 基本命令
- `gg-sheet init`: 初始化工作区
- `gg-sheet check`: 检查配置表
- `gg-sheet generate`: 生成 Valkyrie 脚本
- `gg-sheet watch`: 启用监听模式

### 选项
- `--workspace <path>`: 指定工作目录
- `--verbose`: 启用详细日志
- `--quiet`: 静默模式，仅输出错误信息

## 配置表格式规范

### 表格类型

| 类型 | 说明 | 标记方式 |
| ---- | ---- | ------- |
| dict 表 | 字符串主键，最常用的配置形式 | 默认或 `@dict` 标记 |
| list 表 | 整数主键，按顺序访问 | `@list` 标记 |
| enum 表 | 枚举类型，附加额外数据 | `@enum` 标记 |
| class 表 | 全局配置类，单例模式 | `@class` 标记 |
| language 表 | 多语言支持 | `@language` 标记 |

### 表头定义
- **第一行**: 字段注释
- **第二行**: 字段名称
- **第三行**: 字段类型

### 支持的字段类型
- **基本类型**:
  - `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`
  - `f32`, `f64`
  - `bool`
  - `string` (生成为 `UTF8Text`)

- **复合类型**:
  - `list<V>`: 列表类型
  - `dict<V>`: 字典类型
  - `map<K, V>`: 映射类型

- **特殊类型**:
  - `color`: 颜色类型 (生成为 `Color`)

### 特殊标记
- `@T`: 表示该字段的值在表中必须唯一
- `@@T`: 表示该字段的值在表中必须唯一且作为主键

### 合表功能
- **命名约定**：下划线命名自动合并，如 `Item_Weapon` + `Item_Armor` → `Item`
- **合并规则**：相同结构自动合并，相同 ID 报错
- **保留表名**：`Language` 为保留表名

### 配置系统
- **项目配置**：`GGSheet.toml` 项目根目录
- **表格配置**：同名 `.toml` 文件
- **行映射**：支持旧表迁移

## Valkyrie 脚本生成格式

### 1. Dict 表生成格式

```valkyrie
# ItemQualityTable.v, 自动生成，修改无效
namespace package::table

class ItemQuality {
    quality_key: UTF8Text
    item_icon: UTF8Text
    display_color: Color
}

class ItemQualityTable {
    private _data: HashMap<UTF8Text, ItemQuality>
}

imply ItemQualityTable {
    micro load() -> ItemQualityTable {
        # {
        #    "common": ItemQuality { quality_key: "common", item_icon: "item_01.png", display_color: "#FFFFFF" },
        #    "rare": ItemQuality { quality_key: "rare", item_icon: "item_02.png", display_color: "#00FFFF" },
        #    "epic": ItemQuality { quality_key: "epic", item_icon: "item_03.png", display_color: "#FF00FF" }
        # }
        ItemQualityTable {
            _data: gg.load_sheet("ItemQuality") 
        }
    }

    micro find(self, key: UTF8Text) -> Option<ItemQuality> {
        return self._data[key]
    }

    micro all(self) -> Generator<Item=ItemQuality> {
        return self._data.values()
    }
}
```

### 2. List 表生成格式

```valkyrie
# ItemTable.v, 自动生成，修改无效
namespace package::table

class Item {
    id: i32
    name: UTF8Text
    price: i32
    description: UTF8Text
}

class ItemTable {
    private _data: HashMap<i32, Item>
}

imply ItemTable {
    micro load() -> ItemTable {
        # {
        #    1: Item { id: 1, name: "Sword", price: 100, description: "A sharp sword" },
        #    2: Item { id: 2, name: "Shield", price: 80, description: "A sturdy shield" },
        #    3: Item { id: 3, name: "Potion", price: 20, description: "Restores health" }
        # }
        ItemTable {
            _data: gg.load_sheet("Item") 
        }
    }

    micro find(self, id: i32) -> Option<Item> {
        return self._data[id]
    }

    micro all(self) -> Generator<Item=Item> {
        return self._data.values()
    }
}
```

### 3. Enum 表生成格式

```valkyrie
# QualityTable.v, 自动生成，修改无效
namespace package::table

class Quality {
    id: i32
    name: UTF8Text
    comment: UTF8Text
    icon: UTF8Text
}

class QualityTable {
    private _data: HashMap<UTF8Text, Quality>
    private _values: List<Quality>
}

imply QualityTable {
    micro load() -> QualityTable {
        # {
        #    "Norma": Quality { id: 0, name: "Norma", comment: "普通品质", icon: "icon_01.png" },
        #    "Rare": Quality { id: 1, name: "Rare", comment: "稀有品质", icon: "icon_02.png" },
        #    "Epic": Quality { id: 2, name: "Epic", comment: "史诗品质", icon: "icon_03.png" },
        #    "Super": Quality { id: 3, name: "Super", comment: "传说品质", icon: "icon_04.png" }
        # }
        QualityTable {
            _data: gg.load_sheet("Quality"),
            _values: gg.load_sheet_list("Quality")
        }
    }

    micro find(self, name: UTF8Text) -> Option<Quality> {
        return self._data[name]
    }

    micro all(self) -> Generator<Item=Quality> {
        return self._values.values()
    }
}

# 枚举常量
const QualityEnum = {
    Norma: "Norma",
    Rare: "Rare",
    Epic: "Epic",
    Super: "Super"
}
```

### 4. Class 表生成格式

```valkyrie
# GameConfigTable.v, 自动生成，修改无效
namespace package::table

class GameConfig {
    max_hp: i32
    user_name: UTF8Text
    move_speed: f32
}

class GameConfigTable {
    private _data: GameConfig
}

imply GameConfigTable {
    micro load() -> GameConfigTable {
        # GameConfig { max_hp: 100, user_name: "Player", move_speed: 5.0 }
        GameConfigTable {
            _data: gg.load_sheet("GameConfig") 
        }
    }

    micro get(self) -> GameConfig {
        return self._data
    }
}

# 单例访问
micro get_game_config() -> GameConfig {
    static mut config: Option<GameConfigTable> = None
    if config == None {
        config = Some(GameConfigTable.load())
    }
    return config.unwrap().get()
}
```

### 5. Language 表生成格式

```valkyrie
# LanguageTable.v, 自动生成，修改无效
namespace package::table

class Language {
    key: UTF8Text
    zh_cn: UTF8Text
    en_us: UTF8Text
    ja_jp: UTF8Text
}

class LanguageTable {
    private _data: HashMap<UTF8Text, Language>
}

imply LanguageTable {
    micro load() -> LanguageTable {
        # {
        #    "hello": Language { key: "hello", zh_cn: "你好", en_us: "Hello", ja_jp: "こんにちは" },
        #    "welcome": Language { key: "welcome", zh_cn: "欢迎", en_us: "Welcome", ja_jp: "ようこそ" }
        # }
        LanguageTable {
            _data: gg.load_sheet("Language") 
        }
    }

    micro find(self, key: UTF8Text) -> Option<Language> {
        return self._data[key]
    }

    micro get_text(self, key: UTF8Text, lang: UTF8Text) -> UTF8Text {
        if let Some(lang_data) = self._data[key] {
            match lang {
                "zh_cn" => return lang_data.zh_cn
                "en_us" => return lang_data.en_us
                "ja_jp" => return lang_data.ja_jp
                else => return lang_data.zh_cn
            }
        }
        return ""
    }
}

# 全局语言设置
class LanguageManager {
    current_lang: UTF8Text
    table: LanguageTable
}

static mut lang_manager: Option<LanguageManager> = None

micro init_language(lang: UTF8Text) {
    lang_manager = Some(LanguageManager{
        current_lang: lang,
        table: LanguageTable.load()
    })
}

micro get_text(key: UTF8Text) -> UTF8Text {
    if let Some(manager) = lang_manager {
        return manager.table.get_text(key, manager.current_lang)
    }
    return ""
}
```

## 工作流程

1. 策划在 `/asset/sheet` 目录下创建和编辑 Excel 配置表
2. 运行 `gg-sheet generate` 命令生成 Valkyrie 脚本
3. 游戏运行时加载生成的 Valkyrie 脚本访问配置数据
4. 开发过程中可以使用 `gg-sheet watch` 命令自动监听配置表变化

## 扩展与未来规划

- 支持更多的配置表格式（如 CSV、JSON 等）
- 支持更多的输出脚本格式
- 提供图形界面
- 集成版本控制系统

## 错误处理

- 配置表格式错误：提供详细的错误位置和原因
- 数据类型错误：检测并报告类型不匹配的问题
- 引用关系错误：检测并报告无效的引用

## 性能优化

- 缓存解析结果，避免重复解析
- 增量更新，只处理修改的文件
- 并行处理多个配置表

## 集成建议

- 在 CI/CD 流程中集成 `gg-sheet check` 命令，确保配置表的正确性
- 在开发环境中使用 `gg-sheet watch` 命令，实时更新配置
- 建立配置表的版本控制和审核机制