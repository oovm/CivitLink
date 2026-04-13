# GG Server 架构设计

## 概述

GG Server 是 GG 游戏引擎的服务端运行时，采用 Valkyrie Script → GG IR → GG VM 的执行模型。Rust 层只提供基础设施能力（网络、数据库、缓存等），所有业务逻辑由 Valkyrie 脚本编写。

## 设计目标

1. **高性能**：基于 Rust 异步运行时，充分利用多核性能
2. **脚本驱动**：所有业务逻辑由 Valkyrie 脚本编写
3. **类型安全**：编译期类型检查，零运行时开销
4. **热更新**：支持脚本热更新，无需重启服务
5. **多协议支持**：支持 TCP、WebSocket、HTTP/REST 等多种协议
6. **多数据库支持**：通过 ORM 抽象支持 SQLite、PostgreSQL、MySQL、MongoDB、Redis
7. **多人联机**：支持房间系统、状态同步、帧同步
8. **微服务架构**：支持服务拆分、服务发现、配置中心
9. **全栈开发**：同一套脚本可运行在客户端和服务端，`@main` 多入口

## 文件职责划分

### .schema 文件：只做定义

Schema 文件只负责声明式的定义，不包含任何实现逻辑：

- 数据模型定义（model）
- 消息定义（message）
- 服务接口定义（service）
- 枚举定义（enums）

### .script 文件：只做实现

Script 文件负责具体的业务逻辑实现：

- 服务方法实现
- 中间件实现
- 游戏逻辑
- 数据处理

## 整体架构

```mermaid
graph TB
    subgraph ClientLayer[客户端层]
        GameClient[游戏客户端]
        WebClient[Web 客户端]
        MobileClient[移动客户端]
    end

    subgraph RuntimeLayer[运行时层 - Rust]
        subgraph Protocol[协议层]
            TCPHandler[TCP Handler]
            WSHandler[WebSocket Handler]
            HTTPHandler[HTTP Handler]
        end

        subgraph Infrastructure[基础设施]
            Database[数据库驱动]
            Cache[缓存驱动]
            MessageQueue[消息队列]
            FileSystem[文件系统]
        end

        subgraph VM[GG VM]
            IR[GG IR]
            Executor[指令执行器]
            HostFunctions[宿主函数]
        end
    end

    subgraph ScriptLayer[脚本层 - Valkyrie]
        Schemas[Schema 定义]
        Scripts[Script 实现]
    end

    GameClient --> TCPHandler
    WebClient --> WSHandler
    MobileClient --> HTTPHandler

    TCPHandler --> VM
    WSHandler --> VM
    HTTPHandler --> VM

    VM --> Database
    VM --> Cache
    VM --> MessageQueue
    VM --> FileSystem

    VM --> Schemas
    VM --> Scripts
```

## 分层职责

### Rust 层：基础设施

Rust 层只负责提供底层能力，不包含任何业务逻辑：

| 模块 | 职责 |
|------|------|
| 协议层 | TCP/WebSocket/HTTP 连接管理、协议解析、消息序列化 |
| 数据库驱动 | 连接池、SQL 执行、事务管理 |
| 缓存驱动 | Redis 连接、缓存读写 |
| 消息队列 | 消息发布/订阅 |
| GG VM | IR 执行、内存管理、宿主函数调用 |
| 文件系统 | 文件读写、热更新监听 |

### Valkyrie 层：业务逻辑

所有业务逻辑由 Valkyrie 脚本编写：

| 文件类型 | 职责 |
|----------|------|
| .schema | 数据模型、消息、服务接口定义 |
| .script | 服务实现、中间件、游戏逻辑 |

## 核心组件

### 1. GG 虚拟机

GG VM 是 Valkyrie 脚本的执行引擎，由 Rust 实现：

```rust
pub struct GgVm {
    heap: Heap,
    stack: Stack,
    call_stack: CallStack,
    globals: Globals,
    host_functions: HostFunctionRegistry,
}

pub struct HostFunctionRegistry {
    functions: HashMap<String, HostFunction>,
}

pub type HostFunction = fn(&mut GgVm, &[Value]) -> Result<Value, VmError>;
```

### 2. 宿主函数

Rust 提供的宿主函数，供 Valkyrie 脚本调用：

```rust
impl GgVm {
    pub fn register_host_functions(&mut self) {
        self.register("db_query", host_db_query);
        self.register("db_execute", host_db_execute);
        self.register("cache_get", host_cache_get);
        self.register("cache_set", host_cache_set);
        self.register("mq_publish", host_mq_publish);
        self.register("mq_subscribe", host_mq_subscribe);
        self.register("send_message", host_send_message);
        self.register("broadcast", host_broadcast);
        self.register("spawn_task", host_spawn_task);
        self.register("sleep", host_sleep);
        self.register("now", host_now);
        self.register("uuid", host_uuid);
    }
}
```

### 3. Schema 定义示例

```schema
# game.schema
namespace game;

enums PlayerStatus {
    Offline = 0;
    Online = 1;
    InGame = 2;
}

schema game_db {
    dialect: "postgresql";
    connection: "postgresql://localhost/game";

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
        status: PlayerStatus = PlayerStatus::Offline;

        created_at: datetime = now();
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

service PlayerService {
    get_player(request: GetPlayerRequest) -> Player;
    create_player(request: CreatePlayerRequest) -> Player;
    update_player(request: UpdatePlayerRequest) -> Player;
    delete_player(request: GetPlayerRequest) -> bool;
}
```

### 4. Script 实现示例

```valkyrie
# player.script
using gg::orm;
using gg::cache;
using game;

impl PlayerService for PlayerServiceImpl {
    @main
    micro get_player(request: GetPlayerRequest): Player? {
        let player = orm::find_by_id::<Player>(request.id);

        if player.is_none() {
            return null;
        }

        return player;
    }

    micro create_player(request: CreatePlayerRequest): Player? {
        let existing = orm::query::<Player>()
            .filter("username", request.username)
            .first();

        if existing.is_some() {
            return null;
        }

        let player = Player {
            id: uuid(),
            username: request.username,
            level: 1,
            experience: 0,
            gold: 0,
            status: PlayerStatus::Offline,
            created_at: now(),
        };

        orm::insert(player);
        return player;
    }

    micro update_player(request: UpdatePlayerRequest): Player? {
        let player = orm::find_by_id::<Player>(request.id);

        if player.is_none() {
            return null;
        }

        if request.level.is_some() {
            player.level = request.level;
        }
        if request.experience.is_some() {
            player.experience = request.experience;
        }
        if request.gold.is_some() {
            player.gold = request.gold;
        }

        orm::update(player);
        return player;
    }

    micro delete_player(request: GetPlayerRequest): bool {
        let result = orm::delete_by_id::<Player>(request.id);
        return result > 0;
    }
}
```

### 5. ORM 抽象层

ORM 层屏蔽底层数据库差异，支持多种数据库后端：

```valkyrie
# orm.script
using gg::config;

namespace orm {
    micro find_by_id<T>(id: any): T? {
        let repo = Repository::get::<T>();
        return repo.find_by_id(id);
    }

    micro query<T>(): QueryBuilder<T> {
        let repo = Repository::get::<T>();
        return repo.query();
    }

    micro insert<T>(entity: T): T {
        let repo = Repository::get::<T>();
        return repo.insert(entity);
    }

    micro update<T>(entity: T): T {
        let repo = Repository::get::<T>();
        return repo.update(entity);
    }

    micro delete<T>(entity: T): i32 {
        let repo = Repository::get::<T>();
        return repo.delete(entity);
    }

    micro delete_by_id<T>(id: any): i32 {
        let repo = Repository::get::<T>();
        return repo.delete_by_id(id);
    }
}

class QueryBuilder<T> {
    let repo: Repository<T>;
    let filters: Array<Filter>;
    let orders: Array<Order>;
    let limit_value: i32?;
    let offset_value: i32?;

    micro filter(field: string, value: any): QueryBuilder<T> {
        this.filters.push({ field: field, op: "==", value: value });
        return this;
    }

    micro filter_gt(field: string, value: any): QueryBuilder<T> {
        this.filters.push({ field: field, op: ">", value: value });
        return this;
    }

    micro filter_lt(field: string, value: any): QueryBuilder<T> {
        this.filters.push({ field: field, op: "<", value: value });
        return this;
    }

    micro order_by(field: string, desc: bool): QueryBuilder<T> {
        this.orders.push({ field: field, desc: desc });
        return this;
    }

    micro limit(n: i32): QueryBuilder<T> {
        this.limit_value = n;
        return this;
    }

    micro offset(n: i32): QueryBuilder<T> {
        this.offset_value = n;
        return this;
    }

    micro first(): T? {
        this.limit_value = 1;
        let results = this.repo.execute(this);
        if results.len() > 0 {
            return results[0];
        }
        return null;
    }

    micro all(): Array<T> {
        return this.repo.execute(this);
    }
}
```

### 6. 多数据库适配

Schema 定义数据库类型，ORM 自动适配：

```schema
# game.schema
namespace game;

schema game_db {
    dialect: "postgresql";
    connection: "${DATABASE_URL}";
    ...
}

schema log_db {
    dialect: "mongodb";
    connection: "${MONGODB_URL}";
    ...
}

schema cache_db {
    dialect: "redis";
    connection: "${REDIS_URL}";
}
```

### 7. @main 多入口特性

Valkyrie 脚本支持 `@main` 注解标记入口点，编译器自动组织依赖：

```valkyrie
# server.script
using gg::engine;
using gg::net;
using game;

@main
micro start_server() {
    let config = engine::load_config("server.toml");

    net::listen(config.http_port, micro(request) {
        return router::handle(request);
    });

    net::ws_listen(config.ws_port, micro(socket) {
        return ws_handler::handle(socket);
    });

    engine::run_loop();
}

# admin.script
using gg::engine;
using gg::cli;
using game;

@main
micro admin_tool() {
    cli::run_repl();
}

# migration.script
using gg::orm;
using game;

@main
micro run_migrations() {
    orm::migrate("migrations/");
}
```

编译时指定入口：

```bash
gg build --entry start_server -o game_server
gg build --entry admin_tool -o admin_cli
gg build --entry run_migrations -o migrate
```

### 8. 全栈开发理念

Valkyrie 脚本体系支持全栈开发，同一套代码可运行在客户端和服务端：

```valkyrie
# shared/player.script
using gg::ecs;

component Position {
    x: f32,
    y: f32,
    z: f32,
}

component Velocity {
    x: f32,
    y: f32,
    z: f32,
}

system MovementSystem {
    micro execute(world: World): Result<()> {
        let delta_time = engine::delta_time();

        loop entity, (pos, vel) in world.query([Position, Velocity]).iter() {
            pos.x += vel.x * delta_time;
            pos.y += vel.y * delta_time;
            pos.z += vel.z * delta_time;
        }

        return Ok();
    }
}

# 客户端入口
@main
@target(client)
micro client_main() {
    let world = World::new();
    world.register_system(MovementSystem);

    engine::set_update_fn(micro(dt) {
        world.run_systems();
    });
}

# 服务端入口
@main
@target(server)
micro server_main() {
    let world = World::new();
    world.register_system(MovementSystem);

    net::ws_listen(7777, micro(socket) {
        let entity = world.spawn();
        socket.on_message(micro(msg) {
            handle_input(entity, msg);
        });
    });

    engine::run_loop(micro() {
        world.run_systems();
        broadcast_state(world);
    });
}
```

**平台条件编译**：

```valkyrie
# utils.script
using gg::platform;

micro save_game(data: GameData) {
    @if(platform::is_client()) {
        local_storage::set("save", data);
    }
    @if(platform::is_server()) {
        orm::insert(data);
    }
}

micro load_game(id: string): GameData? {
    @if(platform::is_client()) {
        return local_storage::get("save");
    }
    @if(platform::is_server()) {
        return orm::find_by_id::<GameData>(id);
    }
}
```

## ECS 在服务端的适用性

由于 Valkyrie 脚本具备 ECS 能力，服务端自然可以使用 ECS 架构。但 ECS 并非适用于所有服务端场景，需要根据具体需求选择合适的架构。

### ECS 的优势与挑战

| 特性 | 服务端适用性 | 说明 |
|------|-------------|------|
| 数据导向设计 | ✅ 高度适合 | 缓存友好，批量处理效率高 |
| 组合优于继承 | ✅ 适合 | 灵活的游戏对象组合 |
| 批量处理 | ✅ 非常适合 | 大量实体并行更新 |
| 解耦 | ✅ 适合 | 系统/组件独立演进 |
| 持久化 | ⚠️ 需要额外处理 | ECS 是内存模型，需要序列化层 |
| 关系查询 | ❌ 不擅长 | 复杂关系查询用数据库更合适 |
| 事务一致性 | ⚠️ 需要额外处理 | ECS 事务模型与传统数据库不同 |

### 适用场景分析

**ECS 非常适合的场景**：

| 场景 | 原因 |
|------|------|
| 实时战斗/游戏逻辑 | MOBA、FPS、RTS 等需要大量实体每帧更新 |
| 房间内游戏状态 | 房间是天然的 ECS World 边界 |
| 物理/AI/技能系统 | 需要批量处理、高性能计算 |
| 本地/单机游戏 | 完全使用 ECS，存档时序列化 World |

**ECS 不太适合的场景**：

| 场景 | 推荐方案 |
|------|----------|
| 玩家账号管理 | 传统 CRUD + 数据库 |
| 好友/公会系统 | 传统服务 + 数据库 |
| 排行榜 | Redis |
| 匹配系统 | 传统服务 + 消息队列 |
| 交易/支付 | 传统服务 + 事务 |

### 推荐架构：ECS + 传统服务混合

```mermaid
graph TB
    subgraph GameLogic[游戏逻辑层 - ECS]
        World[ECS World]
        Entities[实体: 玩家/NPC/技能/道具]
        Components[组件: 位置/属性/状态]
        Systems[系统: 移动/战斗/AI]
    end

    subgraph DataService[数据服务层 - 传统架构]
        PlayerService[玩家服务]
        GuildService[公会服务]
        FriendService[好友服务]
    end

    subgraph Storage[存储层]
        DB[(数据库)]
        Cache[(缓存)]
    end

    World --> Entities
    Entities --> Components
    Components --> Systems

    Systems --> PlayerService
    PlayerService --> DB
    PlayerService --> Cache
```

### 房间内使用 ECS

房间是 ECS World 的天然边界，房间内的游戏逻辑使用 ECS 实现：

```valkyrie
# room.script
using gg::ecs;
using gg::engine;

component Position {
    x: f32,
    y: f32,
    z: f32,
}

component Velocity {
    x: f32,
    y: f32,
    z: f32,
}

component Health {
    current: i32,
    max: i32,
}

component PlayerTag {
    player_id: string,
}

system MovementSystem {
    micro execute(world: World): Result<()> {
        let delta_time = engine::delta_time();

        loop entity, (pos, vel) in world.query([Position, Velocity]).iter() {
            pos.x += vel.x * delta_time;
            pos.y += vel.y * delta_time;
            pos.z += vel.z * delta_time;
        }

        return Ok();
    }
}

system CombatSystem {
    micro execute(world: World): Result<()> {
        loop entity, health in world.query([Health]).iter() {
            if health.current <= 0 {
                world.despawn(entity);
            }
        }

        return Ok();
    }
}

class GameRoom {
    let world: World;
    let room_id: string;
    let tick: u64;

    constructor(room_id: string) {
        this.world = World::new();
        this.room_id = room_id;
        this.tick = 0;

        this.world.register_system(MovementSystem);
        this.world.register_system(CombatSystem);
    }

    micro tick_update() {
        this.world.run_systems();
        this.tick += 1;
    }

    micro spawn_player(player_id: string): EntityId {
        let entity = this.world.spawn();
        this.world.add_component(entity, Position { x: 0, y: 0, z: 0 });
        this.world.add_component(entity, Velocity { x: 0, y: 0, z: 0 });
        this.world.add_component(entity, Health { current: 100, max: 100 });
        this.world.add_component(entity, PlayerTag { player_id: player_id });
        return entity;
    }
}
```

### ECS 与数据服务的桥接

玩家进入/离开房间时，需要在 ECS 和数据服务之间同步状态：

```valkyrie
# room_bridge.script
using gg::orm;
using game;

class RoomBridge {
    micro on_player_join(room: GameRoom, player_id: string) {
        let player_data = orm::find_by_id::<Player>(player_id);

        if player_data.is_none() {
            return;
        }

        let entity = room.spawn_player(player_id);

        let health = room.world.get_component::<Health>(entity);
        health.current = player_data.health;
        health.max = player_data.max_health;
    }

    micro on_player_leave(room: GameRoom, player_id: string) {
        loop entity, tag in room.world.query([PlayerTag]).iter() {
            if tag.player_id == player_id {
                let pos = room.world.get_component::<Position>(entity);
                let health = room.world.get_component::<Health>(entity);

                let player = orm::find_by_id::<Player>(player_id);
                if player.is_some() {
                    player.health = health.current;
                    player.position_x = pos.x;
                    player.position_y = pos.y;
                    player.position_z = pos.z;
                    orm::update(player);
                }

                room.world.despawn(entity);
                break;
            }
        }
    }
}
```

### 架构选择总结

| 场景 | 推荐架构 | 原因 |
|------|----------|------|
| 房间内游戏逻辑 | ✅ ECS | 高频更新、批量处理、状态同步 |
| 实时战斗/物理 | ✅ ECS | 性能敏感、大量实体交互 |
| AI/技能系统 | ✅ ECS | 组件化设计、易于扩展 |
| 玩家账号管理 | ❌ 传统 CRUD | 需要持久化、事务、查询 |
| 好友/公会系统 | ❌ 传统服务 | 关系型数据、复杂查询 |
| 排行榜 | ❌ Redis | 有序集合、高性能读写 |
| 匹配系统 | ❌ 传统服务 + MQ | 跨房间、异步处理 |

**核心原则**：
- **ECS 用于有状态的、需要频繁更新的游戏逻辑**
- **传统服务用于需要持久化、查询、事务的业务数据**
- **两者通过桥接层协作，各司其职**

## 多人联机架构

### 房间系统

```valkyrie
# room_service.script
using gg::collection;

class RoomManager {
    let rooms: Map<string, GameRoom>;
    let player_rooms: Map<string, string>;

    constructor() {
        this.rooms = Map::new();
        this.player_rooms = Map::new();
    }

    micro create_room(host_id: string, config: RoomConfig): GameRoom {
        let room_id = uuid();
        let room = GameRoom(room_id);

        this.rooms.insert(room_id, room);
        this.player_rooms.insert(host_id, room_id);

        return room;
    }

    micro join_room(player_id: string, room_id: string): bool {
        let room = this.rooms.get(room_id);

        if room.is_none() {
            return false;
        }

        this.player_rooms.insert(player_id, room_id);
        return true;
    }

    micro leave_room(player_id: string): bool {
        let room_id = this.player_rooms.get(player_id);

        if room_id.is_none() {
            return false;
        }

        this.player_rooms.remove(player_id);
        return true;
    }
}
```

### 状态同步

```valkyrie
# state_sync.script
using gg::ecs;
using gg::engine;

component Dirty;

system StateSyncSystem {
    micro execute(world: World): Result<()> {
        let mut changes = [];

        loop entity, (pos, health, tag) in world.query([Position, Health, PlayerTag, Dirty]).iter() {
            changes.push({
                entity_id: tag.player_id,
                position: pos,
                health: health,
            });
            world.remove_component::<Dirty>(entity);
        }

        if changes.len() > 0 {
            engine::broadcast("state_sync", changes);
        }

        return Ok();
    }
}
```

### 帧同步

```valkyrie
# frame_sync.script
using gg::ecs;
using gg::engine;

class FrameSyncManager {
    let frame_rate: i32;
    let current_frame: u64;
    let input_buffer: Map<u64, Map<string, bytes>>;

    constructor(frame_rate: i32) {
        this.frame_rate = frame_rate;
        this.current_frame = 0;
        this.input_buffer = Map::new();
    }

    micro receive_input(player_id: string, frame: u64, input: bytes) {
        let frame_inputs = this.input_buffer.get_or_insert(frame, Map::new());
        frame_inputs.insert(player_id, input);
    }

    micro tick(room: GameRoom, expected_players: i32) {
        let frame_inputs = this.input_buffer.get(this.current_frame);

        if frame_inputs.is_none() {
            return;
        }

        if frame_inputs.len() == expected_players {
            engine::broadcast("frame_sync", {
                frame: this.current_frame,
                inputs: frame_inputs,
            });

            this.current_frame += 1;
        }
    }
}
```

### 匹配系统

```valkyrie
# match_service.script
using gg::collection;
using gg::time;

struct MatchTicket {
    player_id: string,
    rating: i32,
    joined_at: datetime,
}

class MatchManager {
    let queues: Map<string, Array<MatchTicket>>;
    let config: MatchConfig;

    constructor() {
        this.queues = Map::new();
        this.config = MatchConfig {
            rating_range: 100,
            expand_rate: 0.1,
        };
    }

    micro join_queue(mode: string, ticket: MatchTicket) {
        let queue = this.queues.get_or_insert(mode, []);
        queue.push(ticket);
    }

    micro leave_queue(player_id: string, mode: string) {
        let queue = this.queues.get(mode);

        if queue.is_some() {
            let filtered = queue.iter().filter(|t| t.player_id != player_id);
            this.queues.insert(mode, filtered);
        }
    }

    micro process(): Array<MatchResult> {
        let results = [];

        loop mode, queue in this.queues {
            let matched_groups = self.find_matches(mode, queue);

            loop group in matched_groups {
                results.push({
                    players: group,
                    mode: mode,
                });
            }
        }

        return results;
    }

    micro find_matches(mode: string, queue: Array<MatchTicket>): Array<Array<string>> {
        let groups = [];
        let processed = Set::new();

        loop ticket in queue {
            if processed.contains(ticket.player_id) {
                continue;
            }

            let wait_time = now() - ticket.joined_at;
            let expanded_range = self.config.rating_range * (1.0 + self.config.expand_rate * wait_time.as_secs_f32());

            let matched = [ticket.player_id];
            processed.insert(ticket.player_id);

            loop other in queue {
                if processed.contains(other.player_id) {
                    continue;
                }

                let rating_diff = (other.rating - ticket.rating).abs();
                if rating_diff <= expanded_range {
                    matched.push(other.player_id);
                    processed.insert(other.player_id);
                }
            }

            if matched.len() >= 2 {
                groups.push(matched);
            }
        }

        return groups;
    }
}
```

## 微服务架构

### 服务间通信

```valkyrie
# rpc_client.script
using gg::rpc;
using gg::registry;

class ServiceClient {
    micro call(service_name: string, method: string, request: any): any {
        let endpoints = registry::discover(service_name);

        if endpoints.len() == 0 {
            return null;
        }

        let endpoint = endpoints[0];
        return rpc::call(endpoint, method, request);
    }
}
```

### 服务注册

```valkyrie
# service_registry.script
using gg::registry;
using gg::config;

class ServiceBootstrap {
    micro on_init() {
        registry::register({
            service_name: "player-service",
            address: config::get("server.address"),
            port: config::get("server.port"),
        });
    }

    micro on_shutdown() {
        registry::deregister("player-service");
    }
}
```

### 消息队列

```valkyrie
# chat_service.script
using gg::mq;

class ChatService {
    micro on_init() {
        mq::subscribe("chat.world", micro(message) {
            let event = message.payload;
            engine::broadcast("world", event);
        });
    }

    micro send_message(channel: string, sender_id: string, content: string) {
        let message = {
            id: uuid(),
            channel: channel,
            sender_id: sender_id,
            content: content,
            created_at: now(),
        };

        mq::publish("chat." + channel, message);
        return message;
    }
}
```

## 游戏常见场景

### 排行榜系统

```valkyrie
# rank_service.script
using gg::cache;

class RankService {
    micro update_score(board: string, player_id: string, score: i64) {
        let redis = cache::get_connection("rank_cache");
        redis.zadd(board, score, player_id);
    }

    micro get_rank(board: string, player_id: string): any {
        let redis = cache::get_connection("rank_cache");
        let rank = redis.zrevrank(board, player_id);

        if rank.is_none() {
            return null;
        }

        let score = redis.zscore(board, player_id);
        return {
            player_id: player_id,
            score: score,
            rank: rank + 1,
        };
    }

    micro get_top_n(board: string, n: i32): Array<any> {
        let redis = cache::get_connection("rank_cache");
        let results = redis.zrevrange_with_scores(board, 0, n - 1);

        return results.iter().enumerate().map(micro(i, item) {
            return {
                player_id: item.player_id,
                score: item.score,
                rank: i + 1,
            };
        });
    }
}
```

### 好友系统

```valkyrie
# friend_service.script
using gg::orm;
using gg::cache;
using gg::mq;

class FriendService {
    micro send_request(player_id: string, friend_id: string): bool {
        let existing = orm::query::<Friendship>()
            .filter("player_id", player_id)
            .filter("friend_id", friend_id)
            .first();

        if existing.is_some() {
            return false;
        }

        let friendship = Friendship {
            id: uuid(),
            player_id: player_id,
            friend_id: friend_id,
            status: FriendshipStatus::Pending,
            created_at: now(),
        };

        orm::insert(friendship);
        mq::publish("friend.request", { from: player_id, to: friend_id });
        return true;
    }

    micro get_friends(player_id: string): Array<string> {
        let cache_key = "friends:" + player_id;
        let cached = cache::get(cache_key);

        if cached.is_some() {
            return cached;
        }

        let friendships = orm::query::<Friendship>()
            .filter("player_id", player_id)
            .filter("status", FriendshipStatus::Accepted)
            .all();

        let friends = friendships.iter().map(micro(f) { return f.friend_id; });
        cache::set(cache_key, friends, 300);

        return friends;
    }
}
```

## 热更新机制

GG Server 支持脚本热更新，无需重启服务即可更新业务逻辑。

### Rust 层：文件监听

```rust
pub struct HotReloader {
    watcher: FileWatcher,
    vm: Arc<Mutex<GgVm>>,
}

impl HotReloader {
    pub async fn start(&mut self) {
        while let Some(event) = self.watcher.next().await {
            match event {
                FileEvent::Modified(path) => {
                    if path.extension() == Some("script".as_ref()) {
                        if let Err(e) = self.reload_script(&path).await {
                            tracing::error!("Failed to reload {}: {}", path.display(), e);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    async fn reload_script(&mut self, path: &Path) -> Result<(), ReloadError> {
        let source = tokio::fs::read_to_string(path).await?;

        let mut vm = self.vm.lock().await;
        vm.reload_script(path, &source)?;

        tracing::info!("Reloaded script: {}", path.display());
        Ok(())
    }
}
```

## 部署架构

### 单机部署

```mermaid
graph TB
    subgraph SingleServer[单机服务器]
        subgraph Runtime[Rust 运行时]
            VM[GG VM]
            DB[数据库驱动]
            Cache[缓存驱动]
        end

        subgraph Scripts[Valkyrie 脚本]
            Schemas[Schema 定义]
            Implementations[Script 实现]
        end

        subgraph Storage[存储]
            SQLite[(SQLite)]
            Redis[(Redis)]
        end
    end

    Client[客户端] --> VM
    VM --> Schemas
    VM --> Implementations
    VM --> DB
    VM --> Cache
    DB --> SQLite
    Cache --> Redis
```

### 微服务部署

```mermaid
graph TB
    subgraph Gateway[网关]
        APIGW[API Gateway]
    end

    subgraph PlayerService[玩家服务]
        PlayerVM[GG VM]
        PlayerSchemas[Schema]
        PlayerScripts[Script]
    end

    subgraph GuildService[公会服务]
        GuildVM[GG VM]
        GuildSchemas[Schema]
        GuildScripts[Script]
    end

    subgraph Infrastructure[基础设施]
        Registry[服务注册]
        MQ[消息队列]
        DB[(数据库)]
    end

    Client --> APIGW
    APIGW --> PlayerVM
    APIGW --> GuildVM

    PlayerVM --> Registry
    GuildVM --> Registry

    PlayerVM --> MQ
    GuildVM --> MQ

    PlayerVM --> DB
    GuildVM --> DB
```

## 配置示例

### 运行时配置

```toml
[runtime]
name = "game-server"
version = "1.0.0"

[runtime.tcp]
bind = "0.0.0.0:7777"
max_connections = 10000

[runtime.websocket]
bind = "0.0.0.0:7778"
max_connections = 5000

[runtime.http]
bind = "0.0.0.0:7779"

[database.game_db]
dialect = "postgresql"
connection = "postgresql://user:pass@localhost/game"
pool_size = 20

[cache.rank_cache]
dialect = "redis"
connection = "redis://localhost:6379"
pool_size = 10

[registry]
type = "consul"
endpoints = ["localhost:8500"]

[mq]
type = "nats"
endpoints = ["localhost:4222"]

[scripts]
paths = ["scripts/**/*.script", "schemas/**/*.schema"]
watch = true
```

## 总结

GG Server 采用 Valkyrie Script → GG IR → GG VM 的执行模型：

**Rust 层职责**：
- 提供高性能网络协议处理
- 提供数据库、缓存、消息队列驱动
- 提供 GG VM 执行引擎
- 提供宿主函数供脚本调用
- 不包含任何业务逻辑

**Valkyrie 层职责**：
- .schema 文件：数据模型、消息、服务接口定义
- .script 文件：服务实现、游戏逻辑、中间件

**ORM 抽象层**：
- 屏蔽底层数据库差异（PostgreSQL、MySQL、MongoDB、Redis 等）
- 提供统一的 QueryBuilder 接口
- Schema 定义数据库类型，ORM 自动适配

**ECS 适用性**：
- 房间内游戏逻辑 → ECS
- 全局数据服务 → 传统 CRUD
- 两者通过桥接层协作

**全栈开发**：
- 同一套 Valkyrie 脚本可运行在客户端和服务端
- `@main` 多入口点，编译时指定入口
- `@target` 平台条件编译
- 共享组件、系统、工具类代码

核心特性：
1. **脚本驱动**：所有业务逻辑由 Valkyrie 脚本编写
2. **类型安全**：编译期类型检查
3. **热更新**：支持脚本热更新
4. **多协议支持**：TCP、WebSocket、HTTP/REST
5. **多数据库支持**：通过 ORM 抽象支持 SQLite、PostgreSQL、MySQL、MongoDB、Redis
6. **多人联机**：房间系统、状态同步、帧同步、匹配系统
7. **微服务架构**：服务拆分、服务发现、配置中心
8. **全栈开发**：客户端/服务端代码共享，`@main` 多入口

GG Server 是 GG 游戏引擎服务端的核心组件，为游戏开发者提供了一种高效、灵活的方式来构建游戏后端服务。
