# *.schema 文件格式规范

## 概述

*.schema 文件是 GG 游戏引擎用于定义数据模型的文件格式，采用声明式 DSL 语法，为游戏后端、本地持久存储和通用服务后端提供统一的数据模型定义。Schema 文件与 Valkyrie 脚本深度集成，通过编译器生成类型安全的 Valkyrie 绑定和 GG IR。

## 设计目标

1. **游戏后端开发**：支持游戏服务器、排行榜、好友系统、公会系统等游戏后端服务
2. **本地持久存储**：支持游戏存档、配置持久化、缓存数据等本地存储场景
3. **通用服务后端**：支持 RESTful API、WebSocket 服务、实时通信等通用后端服务
4. **类型安全**：编译期类型检查，零运行时开销
5. **多数据库支持**：支持 SQLite（本地）、PostgreSQL/MySQL（服务器）、Redis（缓存）

## 基本语法

### 1. 注释

Schema 使用 `#` 进行单行注释。

```schema
# 这是一个注释
```

### 2. Namespace 语句

使用 `namespace` 语句声明当前文件的命名空间，用于 Valkyrie 绑定的模块组织。

```schema
namespace my_game;
```

### 3. Schema 块

使用 `schema name { ... }` 定义数据模型块，包含数据库映射配置和实体表定义。

```schema
namespace my_game;

schema game_backend {
    dialect: "sqlite";
    connection: "game.db";

    # 实体表定义...
}
```

Schema 块支持以下配置：

| 配置项 | 说明 |
|--------|------|
| `dialect` | 数据库方言：`sqlite`、`postgresql`、`mysql`、`redis` |
| `connection` | 数据库连接字符串或文件路径 |
| `pool_size` | 连接池大小（可选） |

### 4. 标量类型

Schema 提供了一套严谨的强类型系统：

- **整数**：`i8`, `i16`, `i32`, `i64` (有符号)；`u8`, `u16`, `u32`, `u64` (无符号)
- **浮点数**：`f32`, `f64`
- **文本**：`string` (UTF-8 编码的变长字符串)
- **逻辑**：`bool` (true 或 false)
- **二进制**：`bytes` (二进制数据)
- **时间与日期**：`datetime` (带时区的时间戳)
- **唯一标识**：`uuid` (UUID)
- **集合类型**：`[T]` (内联对象数组)，`[&T]` (引用数组)，`map<K,V>` (映射)
- **可选类型**：`T?` 或 `Option<T>`

### 5. 定义数据模型 (Model)

使用 `model` 关键字定义数据模型。只有需要映射为数据库表的实体才应该放在 schema 块内。

```schema
namespace my_game;

schema game_backend {
    dialect: "sqlite";
    connection: "game.db";

    model Player {
        @primary_key
        @auto_generate
        id: uuid;

        @unique
        @max_length(50)
        username: string;

        level: i32 = 1;
        experience: i64 = 0;
        gold: i64 = 0;
        created_at: datetime = now();
        updated_at: datetime = now();

        # 关系：[&T] 存储引用（外键）
        inventory: [&PlayerInventory];
    }

    model PlayerInventory {
        @primary_key
        @auto_generate
        id: uuid;

        @references(Player.id)
        @on_delete(cascade)
        player_id: uuid;

        item_id: uuid;
        quantity: i32 = 1;
    }
}
```

### 6. 列表类型说明

| 语法 | 说明 |
|------|------|
| `[T]` | 内联对象数组，对象数据直接存储在父表中（JSON 或平铺） |
| `[&T]` | 引用数组，存储外键引用 |

### 7. 字段注解

| 注解 | 适用范围 | 说明 |
|------|----------|------|
| `@primary_key` | 字段 | 标记为主键 |
| `@auto_generate` | 字段 | 自动生成（UUID、自增） |
| `@unique` | 字段 | 唯一约束 |
| `@index` | 字段 | 创建索引 |
| `@max_length(n)` | 字段 | 最大长度 |
| `@default(value)` | 字段 | 默认值 |
| `@references(Model.field)` | 字段 | 外键引用 |
| `@on_delete(action)` | 字段 | 外键删除行为 |
| `@on_update(value)` | 字段 | 更新时触发的值 |
| `@virtual` | 字段 | 虚拟字段，不存储 |
| `@computed(formula)` | 字段 | 计算列 |

### 8. 定义枚举 (Enums)

使用 `enums` 关键字定义枚举类型。枚举定义在 schema 块外。

```schema
namespace my_game;

enums ItemRarity {
    Common = 0;
    Uncommon = 1;
    Rare = 2;
    Epic = 3;
    Legendary = 4;
}

schema game_backend {
    dialect: "sqlite";
    connection: "game.db";

    model Item {
        @primary_key
        @auto_generate
        id: uuid;

        name: string;
        rarity: ItemRarity;
        base_price: i32;
    }
}
```

### 9. 定义消息 (Message)

使用 `message` 关键字定义 RPC 请求/响应消息。消息定义在 schema 块外。

```schema
namespace my_game;

schema game_backend {
    dialect: "sqlite";
    connection: "game.db";

    model Player {
        @primary_key
        @auto_generate
        id: uuid;

        username: string;
        level: i32 = 1;
    }
}

message GetPlayerRequest {
    @required
    id: uuid;
}

message CreatePlayerRequest {
    @required
    @max_length(50)
    username: string;
}

message UpdatePlayerRequest {
    @required
    id: uuid;

    level: i32?;
    experience: i64?;
    gold: i64?;
}

message ListPlayersRequest {
    page: i32 = 1;
    page_size: i32 = 20;
    min_level: i32?;
}

message PlayerEvent {
    event_type: string;
    player: Player;
    timestamp: datetime;
}
```

### 10. 定义服务 (Service)

使用 `service` 关键字定义 RPC 服务。服务定义在 schema 块外。流类型使用 `Stream<T>` 语法。

```schema
namespace my_game;

schema game_backend {
    dialect: "sqlite";
    connection: "game.db";

    model Player {
        @primary_key
        @auto_generate
        id: uuid;

        username: string;
        level: i32 = 1;
    }
}

message GetPlayerRequest {
    @required
    id: uuid;
}

message ListPlayersRequest {
    page: i32 = 1;
    page_size: i32 = 20;
}

service PlayerService {
    # 一元 RPC
    get_player(request: GetPlayerRequest) -> Player;

    # 服务端流：返回 Stream<T>
    list_players(request: ListPlayersRequest) -> Stream<Player>;

    # 客户端流：参数为 Stream<T>
    batch_create(requests: Stream<CreatePlayerRequest>) -> BatchResponse;

    # 双向流
    watch_player(requests: Stream<WatchRequest>) -> Stream<PlayerEvent>;

    # 可附加注解
    @timeout(5s)
    @middleware("auth")
    delete_player(request: DeletePlayerRequest) -> DeleteResponse;
}
```

### 11. 导入机制 (Using)

使用 `using` 语句导入其他文件中定义的模型、消息、枚举和服务。

```schema
using common.Player;
using common.*;
```

## 完整示例

### 游戏后端 Schema

```schema
# 游戏后端数据模型定义
namespace game_backend;

# 枚举定义
enums ItemRarity {
    Common = 0;
    Uncommon = 1;
    Rare = 2;
    Epic = 3;
    Legendary = 4;
}

enums PlayerStatus {
    Offline = 0;
    Online = 1;
    InGame = 2;
    Away = 3;
}

# Schema 块：实体表定义
schema game_db {
    dialect: "postgresql";
    connection: "postgresql://user:pass@localhost/game";
    pool_size: 20;

    model Player {
        @primary_key
        @auto_generate
        id: uuid;

        @unique
        @max_length(50)
        username: string;

        @max_length(255)
        email: string?;

        level: i32 = 1;
        experience: i64 = 0;
        gold: i64 = 0;
        gems: i64 = 0;

        status: PlayerStatus = PlayerStatus::Offline;
        last_login: datetime?;

        created_at: datetime = now();
        updated_at: datetime = now();

        # 关系
        inventory: [&PlayerInventory];
        friends: [&Friendship];
    }

    model PlayerInventory {
        @primary_key
        @auto_generate
        id: uuid;

        @references(Player.id)
        @on_delete(cascade)
        player_id: uuid;

        item_id: uuid;
        quantity: i32 = 1;
        slot: i32;

        created_at: datetime = now();
    }

    model Item {
        @primary_key
        @auto_generate
        id: uuid;

        name: string;
        description: string?;
        rarity: ItemRarity;
        base_price: i32;
        max_stack: i32 = 1;

        created_at: datetime = now();
    }

    model Friendship {
        @primary_key
        @auto_generate
        id: uuid;

        @references(Player.id)
        @on_delete(cascade)
        player_id: uuid;

        @references(Player.id)
        @on_delete(cascade)
        friend_id: uuid;

        created_at: datetime = now();

        @unique(["player_id", "friend_id"])
    }

    model Guild {
        @primary_key
        @auto_generate
        id: uuid;

        @unique
        @max_length(50)
        name: string;

        @max_length(10)
        tag: string;

        @references(Player.id)
        leader_id: uuid;

        level: i32 = 1;
        experience: i64 = 0;
        member_count: i32 = 1;

        created_at: datetime = now();
    }

    model GuildMember {
        @primary_key
        @auto_generate
        id: uuid;

        @references(Guild.id)
        @on_delete(cascade)
        guild_id: uuid;

        @references(Player.id)
        @on_delete(cascade)
        player_id: uuid;

        role: string = "member";
        joined_at: datetime = now();
    }
}

# RPC 消息定义
message GetPlayerRequest {
    @required
    id: uuid;
}

message CreatePlayerRequest {
    @required
    @max_length(50)
    username: string;

    @max_length(255)
    email: string?;
}

message UpdatePlayerRequest {
    @required
    id: uuid;

    level: i32?;
    experience: i64?;
    gold: i64?;
    gems: i64?;
}

message ListPlayersRequest {
    page: i32 = 1;
    page_size: i32 = 20;
    min_level: i32?;
    status: PlayerStatus?;
}

message AddItemRequest {
    @required
    player_id: uuid;

    @required
    item_id: uuid;

    quantity: i32 = 1;
}

message BatchResponse {
    success: bool;
    count: i32;
    errors: [string];
}

message GetGuildRequest {
    @required
    id: uuid;
}

# RPC 服务定义
service PlayerService {
    get_player(request: GetPlayerRequest) -> Player;
    create_player(request: CreatePlayerRequest) -> Player;
    update_player(request: UpdatePlayerRequest) -> Player;
    delete_player(request: GetPlayerRequest) -> bool;

    list_players(request: ListPlayersRequest) -> Stream<Player>;
    watch_player(request: GetPlayerRequest) -> Stream<PlayerEvent>;

    @timeout(10s)
    batch_create(requests: Stream<CreatePlayerRequest>) -> BatchResponse;
}

service InventoryService {
    get_inventory(request: GetPlayerRequest) -> [PlayerInventory];
    add_item(request: AddItemRequest) -> PlayerInventory;
    remove_item(request: AddItemRequest) -> bool;
}

service FriendService {
    add_friend(player_id: uuid, friend_id: uuid) -> Friendship;
    remove_friend(player_id: uuid, friend_id: uuid) -> bool;
    list_friends(request: GetPlayerRequest) -> [Player];
}

service GuildService {
    create_guild(name: string, tag: string, leader_id: uuid) -> Guild;
    join_guild(guild_id: uuid, player_id: uuid) -> GuildMember;
    leave_guild(guild_id: uuid, player_id: uuid) -> bool;
    list_members(request: GetGuildRequest) -> [GuildMember];
}
```

### 本地存储 Schema

```schema
# 本地游戏存档数据模型
namespace local_save;

schema save_db {
    dialect: "sqlite";
    connection: "savegame.db";

    model SaveSlot {
        @primary_key
        id: i32;

        player_name: string;
        level: i32;
        play_time: i64;

        scene: string;
        position_x: f32;
        position_y: f32;
        position_z: f32;

        created_at: datetime = now();
        updated_at: datetime = now();

        # 引用关系
        progress: [&PlayerProgress];
        settings: [&PlayerSettings];
    }

    model PlayerProgress {
        @primary_key
        id: i32;

        @references(SaveSlot.id)
        @on_delete(cascade)
        save_slot_id: i32;

        quest_id: string;
        completed: bool = false;
        progress: json;

        created_at: datetime = now();
    }

    model PlayerSettings {
        @primary_key
        id: i32;

        @references(SaveSlot.id)
        @on_delete(cascade)
        save_slot_id: i32;

        music_volume: f32 = 1.0;
        sfx_volume: f32 = 1.0;
        voice_volume: f32 = 1.0;

        language: string = "en";
        subtitles: bool = true;

        graphics_quality: string = "high";
        shadow_quality: string = "medium";
        anti_aliasing: string = "fxaa";
    }
}

message LoadSaveRequest {
    @required
    slot_id: i32;
}

message SaveGameRequest {
    @required
    slot_id: i32;

    player_name: string;
    level: i32;
    scene: string;
    position: [f32, f32, f32];
}

service SaveService {
    list_saves() -> [SaveSlot];
    load_save(request: LoadSaveRequest) -> SaveSlot;
    save_game(request: SaveGameRequest) -> SaveSlot;
    delete_save(request: LoadSaveRequest) -> bool;
}
```

### 实时通信 Schema

```schema
# 实时聊天服务数据模型
namespace chat;

schema chat_db {
    dialect: "redis";
    connection: "redis://localhost:6379";

    model ChatRoom {
        @primary_key
        @auto_generate
        id: uuid;

        name: string;
        description: string?;
        is_private: bool = false;

        created_at: datetime = now();

        messages: [&ChatMessage];
        members: [&RoomMember];
    }

    model ChatMessage {
        @primary_key
        @auto_generate
        id: uuid;

        @references(ChatRoom.id)
        @on_delete(cascade)
        room_id: uuid;

        sender_id: uuid;
        sender_name: string;

        @max_length(1000)
        content: string;

        created_at: datetime = now();
    }

    model RoomMember {
        @primary_key
        @auto_generate
        id: uuid;

        @references(ChatRoom.id)
        @on_delete(cascade)
        room_id: uuid;

        player_id: uuid;
        role: string = "member";

        joined_at: datetime = now();
    }
}

message SendMessageRequest {
    @required
    room_id: uuid;

    @required
    sender_id: uuid;

    @required
    @max_length(1000)
    content: string;
}

message JoinRoomRequest {
    @required
    room_id: uuid;

    @required
    player_id: uuid;
}

service ChatService {
    create_room(name: string, is_private: bool) -> ChatRoom;
    join_room(request: JoinRoomRequest) -> RoomMember;
    leave_room(request: JoinRoomRequest) -> bool;

    send_message(request: SendMessageRequest) -> ChatMessage;
    stream_messages(room_id: uuid) -> Stream<ChatMessage>;

    chat(messages: Stream<SendMessageRequest>) -> Stream<ChatMessage>;
}
```

## 代码生成

### GG IR 生成

编译器为每个 Schema 生成 GG IR，供 GG VM 执行：

```
generated/game_backend/
├── schema.ir           # GG IR 中间表示
├── bindings.v          # Valkyrie 绑定
└── migrations/         # 数据库迁移文件
    ├── 001_init.sql
    └── ...
```

### Valkyrie 绑定生成

编译器为每个 Schema 生成 Valkyrie 脚本绑定：

```valkyrie
using game_backend;

let player = game_backend::Player {
    id: uuid(),
    username: "Hero",
    email: null,
    level: 1,
    experience: 0,
    gold: 100,
    gems: 0,
    status: game_backend::PlayerStatus::Online,
    last_login: null,
};

let service = game_backend::PlayerService::new();
let result = await service.create_player({
    username: "Hero",
    email: null,
});
```

## 使用场景

### 1. 在 Valkyrie 脚本中使用

```valkyrie
using game_backend;
using gg::database;

let db = database::connect("game.db");

let player = await game_backend::Player::find_by_id(db, player_id);

let new_player = game_backend::Player {
    id: uuid(),
    username: "NewPlayer",
    level: 1,
    experience: 0,
    gold: 100,
    gems: 0,
    status: game_backend::PlayerStatus::Online,
};
await game_backend::Player::insert(db, new_player);

player.level = player.level + 1;
await game_backend::Player::update(db, player);

let service = game_backend::PlayerService::new();
let result = await service.get_player({ id: player_id });
```

### 2. 在 Prefab 中引用

```von
Component {
    type: "SchemaBinding",
    properties: SchemaBindingProperties {
        schema: "assets/schemas/game.schema",
        model: "Player",
        service: "PlayerService",
    },
}
```

## 最佳实践

1. **命名规范**：模型名称使用 PascalCase，表名自动生成为 snake_case，字段名使用 snake_case。
2. **主键设计**：推荐使用 UUID 作为主键，便于分布式系统。
3. **索引优化**：为常用查询字段添加 `@index` 注解。
4. **关系设计**：合理使用 `@references` 和 `@on_delete`，注意级联删除的影响。
5. **版本控制**：将 Schema 文件纳入版本控制系统。
6. **迁移管理**：使用迁移文件管理数据库变更。
7. **Schema 组织**：只有实体表放在 schema 块内，枚举、消息、服务放在 schema 块外。

## 总结

*.schema 文件格式采用声明式 DSL 语法，为 GG 游戏引擎提供了一种统一、类型安全的数据模型定义方式。核心特性包括：

- **声明式语法**：简洁直观的 DSL，易于理解和维护
- **类型安全**：编译期类型检查，生成 GG IR 和 Valkyrie 绑定
- **多数据库支持**：支持 SQLite、PostgreSQL、MySQL、Redis
- **RPC 服务集成**：支持一元、流式等多种 RPC 模式
- **与 GG Engine 深度集成**：与 Valkyrie 脚本、Prefab 系统无缝协作

Schema 系统是 GG 游戏引擎数据层的核心组成部分，为游戏开发者提供了一种高效、灵活的方式来管理游戏数据。
