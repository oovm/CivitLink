# Valkyrie 脚本语言规范

## 概述

Valkyrie 是 GG Game Engine 使用的脚本语言，结合了 Oak 语言的简洁性和 Valkyrie 语言的类型系统优势，为游戏开发提供了一种现代化、高性能的脚本解决方案。

## 语言特性

### 1. 语法特点

- **简洁易读**：借鉴 Oak 语言的简洁语法，代码可读性高
- **类型系统**：支持渐进式类型，结合静态类型和动态类型的优点
- **多范式**：支持函数式编程和面向对象编程
- **高性能**：针对游戏开发场景优化，执行效率高
- **异步支持**：内置异步编程能力，适合游戏逻辑处理

### 2. 基础语法

#### 注释

```valkyrie
# 单行注释

<#
  多行注释
#>
```

#### 数据类型

```valkyrie
# 基本类型
let integer = 123;
let float = 123.456;
let boolean = true;
let null_value = null;
let string = "Hello, Valkyrie!";

# 复合类型
let list = [1, 2, 3, 4, 5];
let map = {a: 1, b: 2, c: 3};
```

#### 变量声明

```valkyrie
# 可变变量
let mutable counter = 0;

# 不可变变量
let PI = 3.14159;
```

#### 函数定义

```valkyrie
# 基本函数
micro add(a, b) {
    return a + b;
}

# 带类型提示的函数
micro multiply(a: i32, b: i32): i32 {
    return a * b;
}

# 匿名函数
let square = micro(x) {
    return x * x;
};
```

#### 控制流

```valkyrie
# if 语句
if condition {
    # 代码块
} else if another_condition {
    # 代码块
} else {
    # 代码块
}

# 循环
# 遍历表达式结果
loop item in list {
    # 循环体
}

# 无限循环
loop {
    # 代码块
}

while condition {
    # 循环体
}

# match 语句
let result = match value {
    case 0 => "Zero",
    case 1 => "One",
    case _ => "Other"
};
```

### 3. 高级特性

#### 模式匹配

```valkyrie
let result = match value {
    case 0 => "Zero",
    case 1 => "One",
    case _ => "Other"
};
```

#### 异步编程

```valkyrie
async micro fetch_data(url) {
    let response = await http.get(url);
    return response.json();
}

# 调用异步函数
let data = await fetch_data("https://api.example.com/data");
```

#### 类与对象

```valkyrie
class Player {
    let name: String;
    let level: i32;
    
    constructor(name: String, level: i32) {
        this.name = name;
        this.level = level;
    }
    
    micro level_up() {
        this.level += 1;
    }
    
    micro get_info(): String {
        return "Player: " + this.name + ", Level: " + this.level;
    }
}

# 创建实例
let player = Player("Hero", 1);
player.level_up();
console.log(player.get_info());
```

#### 模块系统

```valkyrie
# 导入模块
using math;
using utils::{ add, multiply };

# 定义命名空间
namespace MyModule {
    micro calculate(a, b) {
        return add(a, b) * multiply(a, b);
    }
}

# 使用命名空间中的函数
MyModule::calculate(2, 3);
```

### 4. 游戏开发专用功能

#### 事件系统

Valkyrie 脚本提供了强大的事件系统，使用 `events` 关键字定义多播事件，以及 `event` 关键字定义唯一回调事件。

```valkyrie
# 定义组件时添加事件
component Player {
    name: String,
    health: i32,
    max_health: i32,
    events on_health_change: micro(current: i32, max: i32) -> (),
    events on_death: micro() -> ()
}

# 初始化组件
let player = world.spawn();
player.add_component("Player", {
    name: "Hero",
    health: 100,
    max_health: 100
});

# 订阅事件
player.on_health_change.subscribe(micro(current: i32, max: i32) {
    console::log("Health changed: " + current + "/" + max);
});

player.on_death.subscribe(micro() {
    console::log("Player died!");
});

# 触发事件（在系统中）
micro take_damage(player: Player, amount: i32) {
    player.health -= amount;
    player.on_health_change(player.health, player.max_health);
    if player.health <= 0 {
        player.on_death();
    }
}
```

#### 协程

```valkyrie
using gg::coroutine;

micro enemy_ai() {
    while (true) {
        # 移动敌人
        move_enemy();
        # 等待一段时间
        await coroutine.sleep(1000);
        # 攻击玩家
        attack_player();
        await coroutine.sleep(2000);
    }
}

# 启动协程
coroutine.spawn(enemy_ai);
```

#### 资源管理

```valkyrie
using gg::resources;

# 加载资源
let texture = resources.load_texture("assets/textures/player.png");
let sound = resources.load_sound("assets/sounds/explosion.wav");

# 使用资源
micro render() {
    graphics.draw_texture(texture, position);
}

micro play_explosion() {
    sound.play();
}
```

### 5. 与 GG Engine 集成

#### 引擎 API 调用

```valkyrie
using gg::engine;
using gg::input;
using gg::graphics;

micro update(delta_time: Float) {
    # 处理输入
    if (input.is_key_pressed(Key::Space)) {
        player.jump();
    }
    
    # 更新游戏状态
    player.update(delta_time);
    enemies.update(delta_time);
}

micro render() {
    graphics.clear(Color::BLACK);
    player.render();
    enemies.render();
    ui.render();
}

# 注册游戏循环
engine.set_update_fn(update);
engine.set_render_fn(render);
```

#### ECS 扩展

Valkyrie 脚本提供了专门的 ECS 扩展语法，使用 `component` 和 `system` 关键字来定义组件和系统。基于事件的设计是推荐的最佳实践。

```valkyrie
using gg::ecs;
using gg::engine;

# 定义组件
component Position {
    x: f32,
    y: f32
}

component Velocity {
    dx: f32,
    dy: f32
}

component Player {
    name: String,
    health: i32,
    max_health: i32,
    events on_health_change: micro(current: i32, max: i32) -> (),
    events on_death: micro() -> (),
    events on_jump: micro() -> ()
}

# 定义系统
system MovementSystem {
    micro execute(world: World): Result<()> {
        # 查询拥有 Position 和 Velocity 组件的实体
        let query: Query<(Position, Velocity)> = world.query("Position", "Velocity");
        loop entity, (pos, vel) in query.iter() {
            # 更新位置
            pos.x += vel.dx * delta_time;
            pos.y += vel.dy * delta_time;
        }
        
        return Ok()
    }
}

# 玩家系统
system PlayerSystem {
    micro execute(world: World): Result<()> {
        # 获取玩家实体
        let player_query: Query<Player> = world.query("Player");
        loop entity, player in player_query.iter() {
            # 处理玩家逻辑
            handle_player_input(player);
            check_health(player);
        }
        
        return Ok()
    }
}

# 玩家控制器类
class PlayerController {
    # 初始化玩家
    micro init_player(world: World, name: String, position: [f32, f32]): Player {
        let player_entity = world.spawn();
        let player = player_entity.add_component("Player", {
            name: name,
            health: 100,
            max_health: 100
        });
        
        # 注册事件监听器
        player.on_health_change.subscribe(micro(current: i32, max: i32) {
            on_health_change_handler(player, current, max);
        });
        
        player.on_death.subscribe(micro() {
            on_death_handler(world, player);
        });
        
        player.on_jump.subscribe(micro() {
            on_jump_handler(player);
        });
        
        return player;
    }
    
    # 处理玩家输入
    micro handle_player_input(player: Player) {
        # 处理输入逻辑
    }
    
    # 检查生命值
    micro check_health(player: Player) {
        if player.health <= 0 {
            player.on_death();
        }
    }
    
    # 事件处理函数
    micro on_health_change_handler(player: Player, current: i32, max: i32) {
        console::log(player.name + " health: " + current + "/" + max);
    }
    
    micro on_death_handler(world: World, player: Player) {
        console::log(player.name + " has died!");
    }
    
    micro on_jump_handler(player: Player) {
        console::log(player.name + " jumped!");
    }
}

# 全局控制器实例
let player_controller = PlayerController();
```

#### 与 Prefab 协作

Valkyrie 脚本可以与 Prefab 系统无缝集成，通过在 Prefab 中引用脚本文件来实现基于事件的游戏逻辑。

**在 Prefab 中引用脚本：**

```ron
Component({
    type: "Script",
    properties: ScriptProperties({
        path: "assets/scripts/Player.script",
        class_name: "PlayerSystem",
        enabled: true,
    }),
})
```

**脚本与 Prefab 交互：**

```valkyrie
using gg::ecs;
using gg::engine;

# 玩家组件
component Player {
    name: String,
    health: i32,
    max_health: i32,
    inventory: Array<String>,
    position: [f32, f32],
    events on_health_change: micro(current: i32, max: i32) -> (),
    events on_inventory_change: micro(inventory: Array<String>) -> (),
    events on_death: micro() -> ()
}

# 玩家系统
system PlayerSystem {
    micro execute(world: World): Result<()> {
        # 获取玩家实体
        let player_query: Query<Player> = world.query("Player");
        loop entity, player in player_query.iter() {
            # 处理玩家逻辑
            player_controller.update_player(player);
        }
        
        return Ok()
    }
}

# 玩家控制器
class PlayerController {
    # 初始化玩家
    micro init_player(world: World, name: String, position: [f32, f32]): Player {
        let player_entity = world.spawn();
        let player = player_entity.add_component("Player", {
            name: name,
            health: 100,
            max_health: 100,
            inventory: [],
            position: position
        });
        
        # 注册事件监听器
        player.on_health_change.subscribe(micro(current: i32, max: i32) {
            on_health_change_handler(player, current, max);
        });
        
        player.on_inventory_change.subscribe(micro(inventory: Array<String>) {
            on_inventory_change_handler(player, inventory);
        });
        
        player.on_death.subscribe(micro() {
            on_death_handler(world, player);
        });
        
        return player;
    }
    
    # 更新玩家
    micro update_player(player: Player) {
        # 处理玩家逻辑更新
    }
    
    # 添加物品
    micro add_item(player: Player, item: String) {
        player.inventory.push(item);
        player.on_inventory_change(player.inventory);
    }
    
    # 受伤
    micro take_damage(player: Player, amount: i32) {
        player.health -= amount;
        player.on_health_change(player.health, player.max_health);
        if player.health <= 0 {
            player.on_death();
        }
    }
    
    # 事件处理函数
    micro on_health_change_handler(player: Player, current: i32, max: i32) {
        console::log(player.name + " health: " + current + "/" + max);
    }
    
    micro on_inventory_change_handler(player: Player, inventory: Array<String>) {
        console::log(player.name + " inventory: " + inventory.join(", "));
    }
    
    micro on_death_handler(world: World, player: Player) {
        console::log(player.name + " has died!");
    }
}

# 全局控制器实例
let player_controller = PlayerController();
```

### 类型系统更新

Valkyrie 脚本使用具体的数值类型，包括：

- **整数类型**：`i32`（32位整数）、`i64`（64位整数）
- **浮点类型**：`f32`（32位浮点数）、`f64`（64位浮点数）
- **布尔类型**：`Bool`
- **字符串类型**：`String`
- **数组类型**：`Array<T>`
- **映射类型**：`Map<K, V>`
- **可选类型**：`Option<T>`

### 语法更新

- **if 语句**：不需要括号
  ```valkyrie
  if condition {
      # 代码块
  } else if another_condition {
      # 代码块
  } else {
      # 代码块
  }
  ```
- **loop 语句**：不需要括号，简化语法
  ```valkyrie
  # 遍历表达式结果
  loop entity, component in query.iter() {
      # 代码块
  }

  # 无限循环
  loop {
      # 代码块
  }
  ```
- **while 语句**：不需要括号
  ```valkyrie
  while condition {
      # 代码块
  }
  ```
- **：match 语句**：不需要括号
  ```valkyrie
  let result = match value {
      case 0： "Zero",
      case 1 => "One",
      case _ => "Other"
  };
  ```
- **系统定义**：自动生成 name() 方法
  ```valkyrie
  system MySystem {
      micro execute(world: World): Result<()> {
          # 系统逻辑
          return Ok()
      }
  }
  ```

## 最佳实践

1. **代码组织**：将游戏逻辑按功能模块组织，使用清晰的命名空间
2. **性能优化**：避免在热路径中使用复杂的计算，使用信号系统进行状态管理
3. **错误处理**：使用 try-catch 处理可能的异常，确保游戏稳定性
4. **内存管理**：合理使用资源加载和释放，避免内存泄漏
5. **代码风格**：遵循一致的代码风格，使用适当的缩进和命名约定

## 示例代码

### 角色系统

```valkyrie
using gg::signal;

class Character {
    let name: String;
    let health = signal(100);
    let level: i32;
    
    constructor(name: String, level: i32) {
        this.name = name;
        this.level = level;
        
        # 监听生命值变化
        this.health.subscribe((value) => {
            if (value <= 0) {
                this.on_death();
            }
        });
    }
    
    micro take_damage(amount: i32) {
        this.health(this.health() - amount);
    }
    
    micro heal(amount: i32) {
        this.health(min(this.health() + amount, 100));
    }
    
    micro on_death() {
        console.log(this.name + " has died!");
    }
    
    micro level_up() {
        this.level += 1;
        this.health(100); # 升级时回满生命值
        console.log(this.name + " leveled up to " + this.level + "!");
    }
}

# 创建角色
let hero = Character("Hero", 1);
let enemy = Character("Enemy", 1);

# 战斗模拟
hero.take_damage(30);
enemy.take_damage(70);
hero.level_up();
```

## 总结

Valkyrie 脚本语言为 GG Game Engine 提供了一种现代化、高效的脚本解决方案，结合了 Oak 语言的简洁性和 Valkyrie 语言的强大特性。通过支持渐进式类型、异步编程、面向对象和函数式编程范式，Valkyrie 使得游戏开发变得更加灵活和高效。

开发者可以使用 Valkyrie 脚本快速实现游戏逻辑、UI 交互、AI 行为等各种游戏功能，同时享受类型安全和高性能带来的好处。
