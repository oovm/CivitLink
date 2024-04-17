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

# 多行注释
# 多行注释
```

#### 数据类型

```valkyrie
// 基本类型
let integer = 123;
let float = 123.456;
let boolean = true;
let null_value = null;
let string = "Hello, Valkyrie!";

// 复合类型
let list = [1, 2, 3, 4, 5];
let map = {a: 1, b: 2, c: 3};
```

#### 变量声明

```valkyrie
// 可变变量
let mutable counter = 0;

// 不可变变量
let PI = 3.14159;
```

#### 函数定义

```valkyrie
# 基本函数
micro add(a, b) {
    return a + b;
}

# 带类型提示的函数
micro multiply(a: Int, b: Int): Int {
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
if (condition) {
    # 代码块
} else if (another_condition) {
    # 代码块
} else {
    # 代码块
}

# 循环
for (let i = 0; i < 10; i++) {
    # 循环体
}

foreach (item in list) {
    # 循环体
}

while (condition) {
    # 循环体
}
```

### 3. 高级特性

#### 模式匹配

```valkyrie
let result = match (value) {
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
    let level: Int;
    
    constructor(name: String, level: Int) {
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

#### 信号系统

```valkyrie
using gg::signal;

let health = signal(100);
let score = signal(0);

# 订阅信号变化
health.subscribe((new_value) => {
    if (new_value <= 0) {
        console.log("Game Over!");
    }
});

# 更新信号
health(health() - 10);
score(score() + 100);
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

#### 组件系统

```valkyrie
using gg::ecs;

class PositionComponent {
    let x: Float;
    let y: Float;
    
    constructor(x: Float, y: Float) {
        this.x = x;
        this.y = y;
    }
}

class VelocityComponent {
    let dx: Float;
    let dy: Float;
    
    constructor(dx: Float, dy: Float) {
        this.dx = dx;
        this.dy = dy;
    }
}

# 系统
micro movement_system(entities: List<Entity>) {
    foreach (entity in entities) {
        if (entity.has_components(PositionComponent, VelocityComponent)) {
            let pos = entity.get_component(PositionComponent);
            let vel = entity.get_component(VelocityComponent);
            
            pos.x += vel.dx * delta_time;
            pos.y += vel.dy * delta_time;
        }
    }
}

# 注册系统
engine.add_system(movement_system);
```

## 最佳实践

1. **代码组织**：将游戏逻辑按功能模块组织，使用清晰的命名空间
2. **性能优化**：避免在热路径中使用复杂的计算，使用信号系统进行状态管理
3. **错误处理**：使用 try-catch 处理可能的异常，确保游戏稳定性
4. **内存管理**：合理使用资源加载和释放，避免内存泄漏
5. **代码风格**：遵循一致的代码风格，使用适当的缩进和命名约定

## 示例代码

### 简单游戏循环

```valkyrie
using gg::engine;
using gg::input;
using gg::graphics;

let player_position = {x: 100, y: 100};
let player_speed = 5;

micro update(delta_time: Float) {
    # 处理输入
    if (input.is_key_down(Key::Left)) {
        player_position.x -= player_speed;
    }
    if (input.is_key_down(Key::Right)) {
        player_position.x += player_speed;
    }
    if (input.is_key_down(Key::Up)) {
        player_position.y -= player_speed;
    }
    if (input.is_key_down(Key::Down)) {
        player_position.y += player_speed;
    }
}

micro render() {
    graphics.clear(Color::BLACK);
    graphics.draw_rect(player_position.x, player_position.y, 50, 50, Color::WHITE);
}

# 启动游戏
engine.set_update_fn(update);
engine.set_render_fn(render);
engine.start();
```

### 角色系统

```valkyrie
using gg::signal;

class Character {
    let name: String;
    let health = signal(100);
    let level: Int;
    
    constructor(name: String, level: Int) {
        this.name = name;
        this.level = level;
        
        # 监听生命值变化
        this.health.subscribe((value) => {
            if (value <= 0) {
                this.on_death();
            }
        });
    }
    
    micro take_damage(amount: Int) {
        this.health(this.health() - amount);
    }
    
    micro heal(amount: Int) {
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