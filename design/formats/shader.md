# *.shader 文件格式规范

## 概述

*.shader 文件是 GG 游戏引擎用于存储着色器信息的文件格式，包含着色器的代码、属性、uniforms 等信息。着色器文件是渲染系统的核心组成部分，用于定义游戏对象的渲染效果。

**注意**：*.shader 文件采用 GG Shader (gs) 语言格式，设计为更易于人类阅读和机器分析的语法，参考了 WGSL 和 HLSL 的语法风格。

## 文件结构

一个完整的 *.shader 文件是一个基于 GG Shader (gs) 语言的文本文件，用于描述着色器的代码、属性和配置。

### 基本结构

```gs
# GG Shader Language

namespace package::shader::common;
using gg_shader::f32::{vec2, vec3, vec4, mat22, mat33, mat44, tex2}
using my_shaders::common::lighting

#? Standard PBR shader
shader StandardShader by PBR {
    albedo: texture = "white" # Albedo texture
    normal: texture = "default_normal" # Normal map
    metallic: f32 = 0.5 @range(0.0, 1.0) # Metallic value
    roughness: f32 = 0.5 @range(0.0, 1.0) # Roughness value
    
    render_queue = "opaque"
    # 渲染状态
    render_states {
        cull_mode = "back"
        blend_mode = "opaque"
        depth_test = true
        depth_write = true
        wireframe = false
    }
    
    # 顶点着色器
    vertex(
        @location(0) position: vec3,
        @location(1) normal: vec3,
        @location(2) uv: vec2
    ) -> @builtin(position) vec4 {
        let mut v_normal: vec3 = normal
        let mut v_uv: vec2 = uv
        
        let model = uniforms.model
        let view = uniforms.view
        let projection = uniforms.projection
        
        return projection * view * model * vec4(position, 1.0)
    }
    
    # 片段着色器
    fragment(
        @location(0) v_normal: vec3,
        @location(1) v_uv: vec2
    ) -> @location(0) vec4 {
        let albedo = texture_sample(uniforms.albedo, v_uv)
        let normal = normalize(v_normal)
        let ndotl = max(dot(normal, uniforms.light_direction), 0.0)
        let diffuse = albedo.rgb * uniforms.light_color * ndotl
        
        return vec4(diffuse, 1.0)
    }
    
    # uniforms 定义
    uniforms {
        model: mat44 # Model matrix
        view: mat44 # View matrix
        projection: mat44 # Projection matrix
        albedo: tex2 # Albedo texture
        normal: tex2 # Normal map
        light_direction: vec3 # Light direction
        light_color: vec3 # Light color
    }
    
    # 回退策略
    fallback {
        # 当 PBR 不可用时回退到 Phong
        when: "pbr_not_supported"
        shader: PhongShader
    }
}

shader StandardPhong by Phong  {

}

micro calculate_lighting(normal: vec3, view_dir: vec3, light_dir: vec3, light_color: vec3) -> vec3 {
    let ndotl = max(dot(normal, light_dir), 0.0)
    let diffuse = light_color * ndotl
    
    let half_dir = normalize(view_dir + light_dir)
    let ndoth = max(dot(normal, half_dir), 0.0)
    let specular = light_color * pow(ndoth, 32.0)
    
    return diffuse + specular
}

# 依赖项和引用由 meta 文件管理
# 元数据由 meta 文件管理
```

## GG Shader (gs) 语言规范

### 1. 基本语法

- **注释**：使用 `#` 进行单行注释，`#?` 用于文档注释
- **字符串**：使用双引号 `"` 包围，支持转义字符
- **数字**：支持整数和浮点数，如 `1`, `0.5`, `1.0f`
- **布尔值**：`true` 和 `false`
- **数组**：使用方括号 `[]` 包围，元素用逗号分隔
- **对象**：使用花括号 `{}` 包围，键值对用 `key: type = value` 格式
- **属性范围**：使用 `@range(min, max)` 定义属性范围
- **装饰器**：使用 `@` 作为装饰器前缀，如 `@location`, `@builtin`

### 2. 关键字

- `shader`：定义着色器主体，格式为 `shader Name by Kind`
- `using`：引入类型和函数
- `namespace`：定义命名空间
- `render_states`：定义渲染状态
- `vertex`：定义顶点着色器函数
- `fragment`：定义片段着色器函数
- `compute`：定义计算着色器函数
- `uniforms`：定义着色器 uniforms
- `fallback`：定义着色器回退策略
- `@location`：指定输入/输出位置
- `@builtin`：指定内建变量
- `@group`：指定资源组
- `@binding`：指定资源绑定
- `let mut`：定义可变变量
- `let`：定义常量
- `return`：返回值，可省略

### 3. 数据类型

- **标量类型**：`bool`, `u32`, `i32`, `f32`
- **向量类型**：`vec2`, `vec3`, `vec4`
- **矩阵类型**：`mat22`, `mat33`, `mat44`
- **纹理类型**：`tex2`, `tex3`, `texcube`
- **采样器类型**：`sampler`
- **缓冲区类型**：`buffer<T>`

## 字段详细说明

### 1. 着色器主体

- **`shader Name by Kind`**：定义着色器，`Name` 为着色器名称，`Kind` 为着色器类型，如 `PBR`、`Unlit`、`Phong`、`Compute` 等。
  - **`#? Description`**：文档注释，描述着色器功能。
  - **属性定义**：直接在 shader 主体中定义属性，格式为 `name: type = default @range(min, max) # description`
  - **`render_queue`**：字符串，渲染队列，如 `opaque`、`transparent`、`overlay` 等。

### 2. 着色器属性

- **属性定义**：直接在 shader 主体中定义，格式为 `name: type = default @range(min, max) # description`
  - **`name`**：属性名称。
  - **`type`**：属性类型，如 `texture`、`f32`、`color`、`vector` 等。
  - **`default`**：属性默认值。
  - **`@range(min, max)`**：属性范围（可选）。
  - **`# description`**：属性描述（可选）。

### 3. 渲染状态

- **`render_states`**：对象，包含着色器的渲染状态。
  - **`cull_mode`**：字符串，剔除模式，如 `back`、`front`、`none`。
  - **`blend_mode`**：字符串，混合模式，如 `opaque`、`alpha`、`additive` 等。
  - **`depth_test`**：布尔值，是否启用深度测试。
  - **`depth_write`**：布尔值，是否启用深度写入。
  - **`wireframe`**：布尔值，是否以线框模式渲染。
  - **`stencil_test`**：布尔值，是否启用模板测试。
  - **`stencil_op`**：字符串，模板操作，如 `keep`、`replace`、`increment` 等。
  - **`multisample`**：布尔值，是否启用多重采样。
  - **`rasterizer_discard`**：布尔值，是否启用光栅化丢弃。

### 4. 着色器代码

- **`vertex(...) -> ...`**：顶点着色器函数。
- **`fragment(...) -> ...`**：片段着色器函数。
- **`compute(...)`**：计算着色器函数。
- 支持标准的着色器语法，包括变量定义、函数调用、运算符等。

### 5. Uniforms 定义

- **`uniforms`**：对象，包含着色器的 uniforms。
  - 每个 uniform 格式：`name: type # description`
  - **`name`**：uniform 名称。
  - **`type`**：uniform 类型，如 `mat44`、`vec3`、`f32` 等。
  - **`# description`**：uniform 描述（可选）。

### 6. 回退策略

- **`fallback`**：对象，定义着色器回退策略。
  - **`when`**：字符串，回退条件，如 `pbr_not_supported`、`compute_shader_not_supported` 等。
  - **`shader`**：类型引用，回退到的着色器类型。

### 7. 命名空间

- **`namespace Name`**：定义命名空间，`Name` 为命名空间名称，支持嵌套命名空间，如 `my_shaders::common`。
  - 命名空间内可以定义函数、常量、类型等。
  - 使用 `using namespace::path` 语句引入命名空间中的内容。

## 着色器类型

### 1. PBR 着色器

PBR (Physically Based Rendering) 着色器是一种基于物理原理的着色器类型，提供更真实的渲染效果。

### 2. Unlit 着色器

Unlit 着色器是一种不接受光照的着色器类型，适用于UI元素、特效等。

### 3. Phong 着色器

Phong 着色器是一种传统的光照模型着色器，适用于一些风格化的游戏。

### 4. Custom 着色器

自定义着色器，开发者可以根据需要编写自定义的着色器代码。

## 内置函数库

### 数学函数
- `sin(x)`：正弦函数
- `cos(x)`：余弦函数
- `tan(x)`：正切函数
- `asin(x)`：反正弦函数
- `acos(x)`：反余弦函数
- `atan(x)`：反正切函数
- `sqrt(x)`：平方根函数
- `pow(x, y)`：幂函数
- `exp(x)`：指数函数
- `log(x)`：自然对数函数
- `abs(x)`：绝对值函数
- `sign(x)`：符号函数
- `floor(x)`：向下取整函数
- `ceil(x)`：向上取整函数
- `fract(x)`：小数部分函数
- `min(x, y)`：最小值函数
- `max(x, y)`：最大值函数
- `clamp(x, min, max)`： clamp 函数
- `mix(x, y, t)`：线性插值函数
- `step(edge, x)`：阶跃函数
- `smoothstep(edge0, edge1, x)`：平滑阶跃函数

### 向量函数
- `length(v)`：向量长度
- `normalize(v)`：向量归一化
- `dot(a, b)`：向量点积
- `cross(a, b)`：向量叉积
- `distance(a, b)`：向量距离
- `reflect(i, n)`：向量反射
- `refract(i, n, eta)`：向量折射

### 矩阵函数
- `mat22(v)`：从向量创建 2x2 矩阵
- `mat33(v)`：从向量创建 3x3 矩阵
- `mat44(v)`：从向量创建 4x4 矩阵
- `transpose(m)`：矩阵转置
- `inverse(m)`：矩阵求逆
- `determinant(m)`：矩阵行列式

### 纹理函数
- `texture_sample(tex, uv)`：采样纹理
- `texture_sample_lod(tex, uv, lod)`：指定 LOD 采样纹理
- `texture_sample_compare(tex, uv, compare)`：带比较的纹理采样

### 光线追踪函数
- `ray_query_initialize(ray, t_min, t_max)`：初始化光线查询
- `ray_query_intersect_any()`：光线与任何物体相交
- `ray_query_intersect_closest()`：光线与最近物体相交
- `ray_query_get_intersection_kind()`：获取相交类型
- `ray_query_get_intersection_object_uid()`：获取相交对象 ID
- `ray_query_get_intersection_distance()`：获取相交距离
- `ray_query_get_intersection_face_index()`：获取相交面索引
- `ray_query_get_intersection_barycentrics()`：获取相交重心坐标

## 使用场景

### 1. 游戏对象渲染

- **角色渲染**：使用 PBR 着色器渲染游戏角色，获得更真实的视觉效果。
- **环境渲染**：使用 PBR 着色器渲染游戏环境，获得更真实的光照效果。
- **特效渲染**：使用自定义着色器渲染游戏特效，如粒子、光效等。

### 2. UI 渲染

- **界面渲染**：使用 Unlit 着色器渲染游戏界面，获得更清晰的 UI 效果。
- **HUD 渲染**：使用 Unlit 着色器渲染游戏 HUD，如血条、雷达等。

### 3. 计算密集型任务

- **粒子系统**：使用计算着色器处理粒子物理和渲染。
- **流体模拟**：使用计算着色器模拟流体效果。
- **物理模拟**：使用计算着色器加速物理计算。
- **AI 计算**：使用计算着色器加速 AI 决策。

### 4. 光线追踪

- **实时光影**：使用光线追踪实现真实的阴影效果。
- **全局光照**：使用光线追踪实现全局光照效果。
- **反射和折射**：使用光线追踪实现真实的反射和折射效果。
- **环境光遮蔽**：使用光线追踪实现环境光遮蔽效果。

## 最佳实践

1. **代码组织**：将着色器代码按照功能组织，使用注释提高代码可读性。
2. **性能优化**：避免在着色器中使用复杂的计算，优化着色器性能。
3. **代码复用**：使用函数和共享代码，减少代码重复。
4. **依赖管理**：依赖项由 meta 文件管理，保持 .shader 文件的简洁性。
5. **版本控制**：将 .shader 文件纳入版本控制系统，确保团队协作时的一致性。
6. **跨平台兼容性**：编写跨平台兼容的着色器代码，避免使用平台特定的特性。
7. **格式规范**：遵循 gs 语言的语法规范，保持文件格式的一致性。
8. **工具支持**：使用 GG 引擎提供的工具来验证和转换 .shader 文件。
9. **回退策略**：为高级特性提供回退方案，确保在低配置设备上的兼容性。
10. **光线追踪优化**：合理使用光线追踪，避免过度使用导致性能问题。
11. **命名空间管理**：使用命名空间组织代码，避免命名冲突。
12. **模块复用**：将通用功能封装到命名空间中，提高代码复用率。

## 示例完整文件

### PBR 着色器示例（使用命名空间）

```gs
# GG Shader Language

using gg_shader::f32::{vec2, vec3, vec4, mat22, mat33, mat44, tex2}
using my_shaders::common::lighting
using my_shaders::pbr::utils

#? Physically Based Rendering shader
shader PBRShader by PBR {
    albedo: texture = "white" # Albedo texture
    normal: texture = "default_normal" # Normal map
    metallic_roughness: texture = "default_metallic_roughness" # Metallic roughness texture
    ao: texture = "white" # Ambient occlusion texture
    emissive: texture = "black" # Emissive texture
    
    render_queue = "opaque"
    
    render_states {
        cull_mode = "back"
        blend_mode = "opaque"
        depth_test = true
        depth_write = true
        wireframe = false
    }
    
    vertex(
        @location(0) position: vec3,
        @location(1) normal: vec3,
        @location(2) uv: vec2,
        @location(3) tangent: vec3
    ) -> struct {
        @builtin(position) pos: vec4,
        @location(0) v_position: vec3,
        @location(1) v_normal: vec3,
        @location(2) v_uv: vec2,
        @location(3) v_tangent: vec3
    } {
        let mut output: struct {
            @builtin(position) pos: vec4,
            @location(0) v_position: vec3,
            @location(1) v_normal: vec3,
            @location(2) v_uv: vec2,
            @location(3) v_tangent: vec3
        }
        
        let model = uniforms.model
        let view = uniforms.view
        let projection = uniforms.projection
        
        output.v_position = (model * vec4(position, 1.0)).xyz
        output.v_normal = (mat33(model) * normal)
        output.v_uv = uv
        output.v_tangent = (mat33(model) * tangent)
        output.pos = projection * view * model * vec4(position, 1.0)
        
        return output
    }
    
    fragment(
        @location(0) v_position: vec3,
        @location(1) v_normal: vec3,
        @location(2) v_uv: vec2,
        @location(3) v_tangent: vec3
    ) -> @location(0) vec4 {
        let albedo = texture_sample(uniforms.albedo, v_uv)
        let normal_map = texture_sample(uniforms.normal, v_uv).rgb * 2.0 - 1.0
        let metallic_roughness = texture_sample(uniforms.metallic_roughness, v_uv)
        let metallic = metallic_roughness.r
        let roughness = metallic_roughness.g
        let ao = texture_sample(uniforms.ao, v_uv).r
        let emissive = texture_sample(uniforms.emissive, v_uv).rgb
        
        let N = normalize(v_normal)
        let T = normalize(v_tangent - dot(v_tangent, N) * N)
        let B = cross(N, T)
        let TBN = mat33(T, B, N)
        let normal = normalize(TBN * normal_map)
        
        let V = normalize(uniforms.camera_position - v_position)
        let L = normalize(uniforms.light_direction)
        
        # 使用命名空间中的函数
        let lighting = utils::calculate_pbr_lighting(
            normal, V, L, albedo.rgb, metallic, roughness,
            uniforms.light_color, uniforms.light_intensity
        )
        
        let ambient = vec3(0.03) * albedo.rgb * ao
        let final_color = lighting + ambient + emissive
        
        return vec4(final_color, 1.0)
    }
    
    uniforms {
        model: mat44 # Model matrix
        view: mat44 # View matrix
        projection: mat44 # Projection matrix
        camera_position: vec3 # Camera position
        light_direction: vec3 # Light direction
        light_color: vec3 # Light color
        light_intensity: f32 # Light intensity
        albedo: tex2 # Albedo texture
        normal: tex2 # Normal map
        metallic_roughness: tex2 # Metallic roughness texture
        ao: tex2 # Ambient occlusion texture
        emissive: tex2 # Emissive texture
    }
    
    # 回退策略
    fallback {
        # 当 PBR 不可用时回退到 Phong
        when: "pbr_not_supported"
        shader: PhongShader
    }
}

# 命名空间定义
namespace my_shaders::common {
    # 通用光照函数
    micro calculate_lighting(normal: vec3, view_dir: vec3, light_dir: vec3, light_color: vec3) -> vec3 {
        let ndotl = max(dot(normal, light_dir), 0.0)
        let diffuse = light_color * ndotl
        
        let half_dir = normalize(view_dir + light_dir)
        let ndoth = max(dot(normal, half_dir), 0.0)
        let specular = light_color * pow(ndoth, 32.0)
        
        return diffuse + specular
    }
}

namespace my_shaders::pbr {
    # PBR 相关函数
    micro distribution_ggx(N: vec3, H: vec3, roughness: f32) -> f32 {
        let a = roughness * roughness
        let a2 = a * a
        let ndot_h = max(dot(N, H), 0.0)
        let ndot_h2 = ndot_h * ndot_h
        
        let nom = a2
        let denom = (ndot_h2 * (a2 - 1.0) + 1.0)
        let denom_sq = denom * denom
        
        return nom / (3.14159265359 * denom_sq)
    }
    
    fn geometry_schlick_ggx(ndot_v: f32, roughness: f32) -> f32 {
        let r = (roughness + 1.0)
        let k = (r * r) / 8.0
        
        let nom = ndot_v
        let denom = ndot_v * (1.0 - k) + k
        
        return nom / denom
    }
    
    fn geometry_smith(N: vec3, V: vec3, L: vec3, roughness: f32) -> f32 {
        let ndot_v = max(dot(N, V), 0.0)
        let ndot_l = max(dot(N, L), 0.0)
        let ggx2 = geometry_schlick_ggx(ndot_v, roughness)
        let ggx1 = geometry_schlick_ggx(ndot_l, roughness)
        
        return ggx1 * ggx2
    }
    
    fn fresnel_schlick(cos_theta: f32, F0: vec3) -> vec3 {
        return F0 + (vec3(1.0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0)
    }
    
    namespace utils {
        # PBR 工具函数
        fn calculate_pbr_lighting(
            normal: vec3, view_dir: vec3, light_dir: vec3,
            albedo: vec3, metallic: f32, roughness: f32,
            light_color: vec3, light_intensity: f32
        ) -> vec3 {
            let H = normalize(view_dir + light_dir)
            let F0 = mix(vec3(0.04), albedo, metallic)
            
            let NDF = my_shaders::pbr::distribution_ggx(normal, H, roughness)
            let G = my_shaders::pbr::geometry_smith(normal, view_dir, light_dir, roughness)
            let F = my_shaders::pbr::fresnel_schlick(max(dot(H, view_dir), 0.0), F0)
            
            let kS = F
            let kD = vec3(1.0) - kS
            let kD_mixed = kD * (1.0 - metallic)
            
            let numerator = NDF * G * F
            let denominator = 4.0 * max(dot(normal, view_dir), 0.0) * max(dot(normal, light_dir), 0.0)
            let specular = numerator / max(denominator, 0.001)
            
            let ndotl = max(dot(normal, light_dir), 0.0)
            let diffuse = albedo / 3.14159265359
            
            return (kD_mixed * diffuse + specular) * light_color * light_intensity * ndotl
        }
    }
}

# 依赖项和引用由 meta 文件管理
# 元数据由 meta 文件管理
```

### 计算着色器示例

```gs
# GG Shader Language

using gg_shader::f32::{vec2, vec3, vec4, mat22, mat33, mat44, tex2, buffer}
using my_shaders::compute::particles

#? Particle simulation compute shader
shader ParticleCompute by Compute {
    
    # 计算着色器
    compute(@builtin(global_invocation_id) id: vec3) {
        let index = id.x
        if index >= uniforms.num_particles {
            return
        }
        
        # 读取粒子数据
        let particle = uniforms.particles[index]
        
        # 更新粒子位置
        let mut new_position = particle.position + particle.velocity * uniforms.delta_time
        
        # 边界检测
        if new_position.x < -10.0 || new_position.x > 10.0 {
            particle.velocity.x = -particle.velocity.x * 0.8
        }
        if new_position.y < -10.0 || new_position.y > 10.0 {
            particle.velocity.y = -particle.velocity.y * 0.8
        }
        if new_position.z < -10.0 || new_position.z > 10.0 {
            particle.velocity.z = -particle.velocity.z * 0.8
        }
        
        # 应用重力
        particle.velocity.y -= 9.8 * uniforms.delta_time
        
        # 更新粒子数据
        particle.position = new_position
        uniforms.particles[index] = particle
    }
    
    uniforms {
        num_particles: u32 # Number of particles
        delta_time: f32 # Delta time
        particles: buffer<struct {
            position: vec3
            velocity: vec3
            lifetime: f32
            size: f32
        }> # Particle data buffer
    }
    
    # 回退策略
    fallback {
        # 当计算着色器不可用时回退到 CPU 计算
        when: "compute_shader_not_supported"
        shader: ParticleCPU
    }
}

# 命名空间定义
namespace my_shaders::compute {
    namespace particles {
        # 粒子系统相关函数
        micro update_particle_position(position: vec3, velocity: vec3, delta_time: f32) -> vec3 {
            return position + velocity * delta_time
        }
        
        micro apply_bounds(position: vec3, velocity: vec3, bounds: vec3) -> vec3 {
            let mut new_velocity = velocity
            
            if position.x < -bounds.x || position.x > bounds.x {
                new_velocity.x = -new_velocity.x * 0.8
            }
            if position.y < -bounds.y || position.y > bounds.y {
                new_velocity.y = -new_velocity.y * 0.8
            }
            if position.z < -bounds.z || position.z > bounds.z {
                new_velocity.z = -new_velocity.z * 0.8
            }
            
            return new_velocity
        }
    }
}

# 依赖项和引用由 meta 文件管理
# 元数据由 meta 文件管理
```

## 工具支持

### 1. 语法高亮

GG 引擎的编辑器插件提供了对 gs 语言的语法高亮支持，包括：
- 关键字高亮
- 字符串高亮
- 注释高亮
- 装饰器高亮
- 语法错误提示

### 2. 格式验证

使用 `gg shader validate` 命令可以验证 .shader 文件的格式是否正确：

```bash
# 验证单个文件
gg shader validate assets/shaders/standard.shader

# 验证目录下所有文件
gg shader validate assets/shaders/
```

### 3. 格式转换

使用 `gg shader convert` 命令可以在旧的 JSON 格式和新的 gs 格式之间进行转换：

```bash
# 将 JSON 格式转换为 gs 格式
gg shader convert --from json --to gs assets/shaders/old.json assets/shaders/new.shader

# 将 gs 格式转换为 JSON 格式（用于兼容旧系统）
gg shader convert --from gs --to json assets/shaders/new.shader assets/shaders/old.json
```

### 4. 跨平台编译

使用 `gg shader compile` 命令可以将 .shader 文件编译为不同平台的着色器代码：

```bash
# 编译为 WGSL
gg shader compile --target wgsl assets/shaders/standard.shader assets/shaders/standard.wgsl

# 编译为 HLSL
gg shader compile --target hlsl assets/shaders/standard.shader assets/shaders/standard.hlsl

# 编译为 GLSL
gg shader compile --target glsl assets/shaders/standard.shader assets/shaders/standard.glsl
```

## 总结

GG Shader (gs) 语言为 GG 游戏引擎提供了一种更易于人类阅读和机器分析的着色器文件格式。通过采用类似 WGSL 和 HLSL 的语法风格，它解决了传统着色器文件在可读性和可维护性方面的问题。

这种设计使得 GG 游戏引擎能够：
- 更快速地创建和管理着色器
- 支持更复杂的着色器配置
- 提供更好的跨平台兼容性
- 简化着色器的版本控制和团队协作
- 为未来的功能扩展提供更灵活的基础
- 确保在不同硬件配置上的兼容性
- 通过命名空间组织代码，提高代码复用率

着色器系统是 GG 游戏引擎中重要的组成部分，为游戏开发者提供了一种高效、灵活的方式来定义和管理游戏对象的渲染效果。