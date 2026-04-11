# *.gameui 文件格式规范

## 概述

*.gameui 文件是 GG 游戏引擎用于存储游戏运行时 UI 的文件格式，基于 ECS/GameObject + Canvas 体系，类似 Unity uGUI。每个 UI 元素都是一个 Entity，通过挂载不同的 UI 组件来实现丰富的界面功能。

核心设计原则：

- **ECS 驱动**：所有 UI 元素都是 Entity，通过组件组合实现功能
- **Canvas 体系**：所有 UI 元素必须在 UiCanvas 下，由 Canvas 决定渲染模式
- **RectTransform 布局**：UI 元素使用 UiRectTransform 替代普通 Transform，支持锚点、轴心等布局特性
- **3D 空间支持**：通过 WorldSpace 模式，UI 可存在于 3D 世界中，支持着色器特效和复杂动画

> **重要声明**：*.gameui 仅用于游戏运行时 UI。编辑器界面请使用 *.vx 文件（DOM 模型），两者体系完全不同，不可混用。

## 文件结构

一个完整的 *.gameui 文件是一个 RON 格式的文本文件，用于描述游戏运行时 UI 的结构和属性。

### 基本结构

```ron
// 游戏 UI 文件
GameUiFile({
    version: "1.0",
    gameui: GameUi({
        name: "GameHUD",
        description: "Main game HUD",
        canvas_mode: ScreenSpace,
    }),
    entities: [
        Entity({
            id: 1,
            name: "Canvas",
            parent_id: None,
            components: [
                Component({
                    type: "UiCanvas",
                    properties: UiCanvasProperties({
                        canvas_mode: ScreenSpace,
                        sort_order: 0,
                        reference_resolution: [1920, 1080],
                        match_mode: Expand,
                        pixel_perfect: false,
                    }),
                }),
            ],
        }),
        Entity({
            id: 2,
            name: "HealthBar",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 1.0],
                        anchor_max: [0.0, 1.0],
                        offset_min: [20.0, -60.0],
                        offset_max: [220.0, -20.0],
                        pivot: [0.0, 1.0],
                        size_delta: [200.0, 40.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/health_bar_bg.png",
                        color: [1.0, 1.0, 1.0, 1.0],
                        material: "",
                        image_type: Sliced,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
            ],
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/textures/ui/health_bar_bg.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0001",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
})
```

## 字段详细说明

### 1. 版本信息

- **`version`**：字符串，Game UI 格式版本号，用于向后兼容。当前版本为 `"1.0"`。

### 2. Game UI 信息

- **`gameui`**：对象，包含 Game UI 的基本信息。
  - **`name`**：字符串，UI 名称。
  - **`description`**：字符串，UI 描述。
  - **`canvas_mode`**：枚举，Canvas 渲染模式，取值为 `ScreenSpace`、`WorldSpace`、`CameraSpace`。

### 3. 实体列表

- **`entities`**：数组，包含 UI 中的所有实体。
  - 每个实体包含：
    - **`id`**：数字，实体 ID，在文件内唯一。
    - **`name`**：字符串，实体名称。
    - **`parent_id`**：数字或 null，父实体 ID，如果是根实体（Canvas）则为 null。
    - **`components`**：数组，实体的组件列表。
      - 每个组件包含：
        - **`type`**：字符串，组件类型。
        - **`properties`**：对象，组件属性，根据组件类型不同而不同。

### 4. 依赖关系

- **`dependencies`**：数组，包含该 Game UI 依赖的其他资源。
  - 每个依赖项包含：
    - **`path`**：字符串，依赖资源的相对路径。
    - **`guid`**：字符串，依赖资源的全局唯一标识符。

### 5. 引用关系

- **`references`**：数组，包含引用该 Game UI 的其他资源。
  - 每个引用项包含：
    - **`path`**：字符串，引用资源的相对路径。
    - **`field`**：字符串，引用该 Game UI 的字段名称。

### 6. 元数据

- **`timestamp`**：字符串，ISO 格式的时间戳，表示该 Game UI 文件的最后修改时间。
- **`hash`**：字符串，Game UI 文件的 SHA1 哈希值，用于检测文件是否发生变化。

## Canvas 配置结构

UiCanvas 是所有 UI 元素的根容器，决定 UI 的渲染方式和空间关系。每个 *.gameui 文件必须包含一个根 UiCanvas 实体。

### ScreenSpace 模式

UI 绘制在屏幕最上层，不随相机移动，适合 HUD、血条、弹药数等覆盖层界面。

```ron
Component({
    type: "UiCanvas",
    properties: UiCanvasProperties({
        canvas_mode: ScreenSpace,
        sort_order: 0,
        reference_resolution: [1920, 1080],
        match_mode: Expand,
        pixel_perfect: false,
    }),
})
```

属性说明：

- **`sort_order`**：i32，Canvas 的排序顺序，数值越大越靠前渲染。
- **`reference_resolution`**：[f32, f32]，参考分辨率，用于自适应缩放计算。
- **`match_mode`**：枚举，自适应缩放匹配模式。
  - `Width`：以宽度为基准进行缩放。
  - `Height`：以高度为基准进行缩放。
  - `Expand`：水平或垂直扩展以填满屏幕，可能超出参考分辨率。
  - `Shrink`：水平或垂直收缩以适应屏幕，不会超出参考分辨率。
- **`pixel_perfect`**：bool，是否启用像素对齐，启用后 UI 元素会对齐到像素网格。

### WorldSpace 模式

UI 存在于 3D 世界中，像普通 3D 物体一样渲染，适合角色头顶血条、世界空间提示面板、VR/AR 界面等。

```ron
Component({
    type: "UiCanvas",
    properties: UiCanvasProperties({
        canvas_mode: WorldSpace,
        sort_order: 0,
        reference_resolution: [800, 600],
        match_mode: Expand,
        pixel_perfect: false,
        width: 2.0,
        height: 1.5,
        distance: 0.0,
        event_camera: "MainCamera",
        sorting_layer: "UI",
        sorting_order: 1,
    }),
})
```

WorldSpace 额外属性说明：

- **`width`**：f32，Canvas 在世界空间中的宽度（世界单位）。
- **`height`**：f32，Canvas 在世界空间中的高度（世界单位）。
- **`distance`**：f32，Canvas 距离挂载点的距离（世界单位）。
- **`event_camera`**：字符串，处理 UI 交互事件的相机名称。
- **`sorting_layer`**：字符串，排序层名称。
- **`sorting_order`**：i32，排序层内的顺序。

### CameraSpace 模式

UI 跟随相机但保持固定距离，适合准星、提示标记等需要与视角关联但不属于 3D 世界的界面。

```ron
Component({
    type: "UiCanvas",
    properties: UiCanvasProperties({
        canvas_mode: CameraSpace,
        sort_order: 0,
        reference_resolution: [1920, 1080],
        match_mode: Expand,
        pixel_perfect: false,
        distance: 5.0,
        follow_mode: Smooth,
    }),
})
```

CameraSpace 额外属性说明：

- **`distance`**：f32，Canvas 距离相机的固定距离（世界单位）。
- **`follow_mode`**：枚举，跟随模式。
  - `Direct`：直接跟随相机，无延迟。
  - `Smooth`：平滑跟随相机，有插值延迟。

## Game UI 组件类型

### UiCanvas（必须组件）

UiCanvas 是所有 UI 的根容器组件，每个 *.gameui 文件的根实体必须挂载此组件。

```ron
Component({
    type: "UiCanvas",
    properties: UiCanvasProperties({
        canvas_mode: ScreenSpace,
        sort_order: 0,
        reference_resolution: [1920, 1080],
        match_mode: Expand,
        pixel_perfect: false,
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `canvas_mode` | 枚举 | `ScreenSpace` | Canvas 渲染模式 |
| `sort_order` | i32 | `0` | 渲染排序顺序 |
| `reference_resolution` | [f32, f32] | `[1920, 1080]` | 参考分辨率 |
| `match_mode` | 枚举 | `Expand` | 自适应缩放匹配模式 |
| `pixel_perfect` | bool | `false` | 是否启用像素对齐 |

### UiRectTransform（必须组件，替代 Transform）

UiRectTransform 是 UI 元素的布局组件，替代普通 Transform。所有 UiCanvas 下的子实体必须使用 UiRectTransform。通过锚点和偏移量实现灵活的相对布局。

```ron
Component({
    type: "UiRectTransform",
    properties: UiRectTransformProperties({
        anchor_min: [0.0, 0.0],
        anchor_max: [1.0, 1.0],
        offset_min: [0.0, 0.0],
        offset_max: [0.0, 0.0],
        pivot: [0.5, 0.5],
        size_delta: [0.0, 0.0],
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `anchor_min` | [f32, f32] | `[0.0, 0.0]` | 锚点最小值，0-1 归一化，对应父容器的左下角比例 |
| `anchor_max` | [f32, f32] | `[1.0, 1.0]` | 锚点最大值，0-1 归一化，对应父容器的右上角比例 |
| `offset_min` | [f32, f32] | `[0.0, 0.0]` | 相对于锚点最小值的偏移量 [left, bottom] |
| `offset_max` | [f32, f32] | `[0.0, 0.0]` | 相对于锚点最大值的偏移量 [right, top] |
| `pivot` | [f32, f32] | `[0.5, 0.5]` | 轴心点，0-1 归一化，决定旋转和缩放的中心 |
| `size_delta` | [f32, f32] | `[0.0, 0.0]` | 相对于锚点区域的尺寸增量 [width, height] |

锚点布局规则：

- 当 `anchor_min` == `anchor_max` 时，元素为固定尺寸，由 `size_delta` 决定大小
- 当 `anchor_min` != `anchor_max` 时，元素随父容器拉伸，`offset` 控制边距

### UiImage

UiImage 用于显示图片，是最基础的 UI 可视化组件。支持多种图片类型和填充方式。

```ron
Component({
    type: "UiImage",
    properties: UiImageProperties({
        sprite: "assets/textures/ui/button.png",
        color: [1.0, 1.0, 1.0, 1.0],
        material: "",
        image_type: Sliced,
        fill_method: Horizontal,
        fill_amount: 1.0,
        preserve_aspect: false,
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `sprite` | String | `""` | 纹理资源路径 |
| `color` | [f32, f32, f32, f32] | `[1.0, 1.0, 1.0, 1.0]` | 颜色 tint [r, g, b, a] |
| `material` | String | `""` | UiMaterial 资源路径，为空则使用默认材质 |
| `image_type` | 枚举 | `Simple` | 图片类型 |
| `fill_method` | 枚举 | `Horizontal` | 填充方式，仅在 `image_type` 为 `Filled` 时生效 |
| `fill_amount` | f32 | `1.0` | 填充量，0.0-1.0，仅在 `image_type` 为 `Filled` 时生效 |
| `preserve_aspect` | bool | `false` | 是否保持图片原始宽高比 |

`image_type` 枚举值：

- `Simple`：普通显示，图片拉伸到矩形区域
- `Sliced`：九宫格切片，边角不变形，中间拉伸
- `Tiled`：平铺，图片重复铺满区域
- `Filled`：填充，按指定方向和量显示图片

`fill_method` 枚举值：

- `Horizontal`：水平填充
- `Vertical`：垂直填充
- `Radial90`：90 度径向填充
- `Radial180`：180 度径向填充
- `Radial360`：360 度径向填充

### UiText

UiText 用于显示文本，支持多种字体样式和对齐方式。可通过 material 字段引用 SDF 着色器实现高质量文字渲染。

```ron
Component({
    type: "UiText",
    properties: UiTextProperties({
        text: "Hello World",
        font: "assets/fonts/main_font.ttf",
        font_size: 24.0,
        font_style: Normal,
        color: [1.0, 1.0, 1.0, 1.0],
        alignment: TopLeft,
        overflow: Wrap,
        line_spacing: 1.0,
        material: "",
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `text` | String | `""` | 显示的文本内容 |
| `font` | String | `""` | 字体资源路径 |
| `font_size` | f32 | `24.0` | 字体大小 |
| `font_style` | 枚举 | `Normal` | 字体样式 |
| `color` | [f32, f32, f32, f32] | `[1.0, 1.0, 1.0, 1.0]` | 文本颜色 [r, g, b, a] |
| `alignment` | 枚举 | `TopLeft` | 文本对齐方式 |
| `overflow` | 枚举 | `Wrap` | 文本溢出处理方式 |
| `line_spacing` | f32 | `1.0` | 行间距倍率 |
| `material` | String | `""` | UiMaterial 资源路径，支持 SDF 着色器 |

`font_style` 枚举值：

- `Normal`：常规
- `Bold`：粗体
- `Italic`：斜体
- `BoldItalic`：粗斜体

`alignment` 枚举值：

- `TopLeft`、`TopCenter`、`TopRight`
- `MiddleLeft`、`MiddleCenter`、`MiddleRight`
- `BottomLeft`、`BottomCenter`、`BottomRight`

`overflow` 枚举值：

- `Wrap`：自动换行
- `Truncate`：截断不显示
- `Overflow`：超出边界继续显示

### UiButton

UiButton 是交互组件，支持多种过渡效果。需要配合 UiImage 或 UiText 作为视觉目标。

```ron
Component({
    type: "UiButton",
    properties: UiButtonProperties({
        interactable: true,
        transition: ColorTint,
        target_graphic: "HealthBarButton",
        colors: ColorBlock({
            normal_color: [1.0, 1.0, 1.0, 1.0],
            highlighted_color: [0.96, 0.96, 0.96, 1.0],
            pressed_color: [0.78, 0.78, 0.78, 1.0],
            disabled_color: [0.78, 0.78, 0.78, 0.5],
            fade_duration: 0.1,
        }),
        on_click: "on_health_bar_click",
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `interactable` | bool | `true` | 是否可交互 |
| `transition` | 枚举 | `ColorTint` | 过渡效果类型 |
| `target_graphic` | String | `""` | 目标视觉 Entity 名称引用 |
| `colors` | ColorBlock | 见下方 | 颜色过渡配置，仅在 `transition` 为 `ColorTint` 时生效 |
| `on_click` | String | `""` | 点击事件回调函数名 |

`transition` 枚举值：

- `None`：无过渡效果
- `ColorTint`：颜色变化
- `SpriteSwap`：精灵切换
- `Animation`：动画播放

ColorBlock 结构：

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `normal_color` | [f32, f32, f32, f32] | `[1.0, 1.0, 1.0, 1.0]` | 正常状态颜色 |
| `highlighted_color` | [f32, f32, f32, f32] | `[0.96, 0.96, 0.96, 1.0]` | 悬停状态颜色 |
| `pressed_color` | [f32, f32, f32, f32] | `[0.78, 0.78, 0.78, 1.0]` | 按下状态颜色 |
| `disabled_color` | [f32, f32, f32, f32] | `[0.78, 0.78, 0.78, 0.5]` | 禁用状态颜色 |
| `fade_duration` | f32 | `0.1` | 颜色过渡持续时间（秒） |

### UiMask

UiMask 用于裁剪子元素，实现圆形头像、滚动列表可见区域等效果。

```ron
Component({
    type: "UiMask",
    properties: UiMaskProperties({
        show_mask_graphic: false,
        sprite: "assets/textures/ui/circle_mask.png",
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `show_mask_graphic` | bool | `false` | 是否显示遮罩图形本身 |
| `sprite` | String | `""` | 遮罩纹理资源路径，为空则使用矩形遮罩 |

### UiLayout

UiLayout 是自动布局组件，自动排列子元素，无需手动设置每个子元素的位置。

```ron
Component({
    type: "UiLayout",
    properties: UiLayoutProperties({
        layout_type: Vertical,
        padding: [10.0, 10.0, 10.0, 10.0],
        spacing: 5.0,
        child_alignment: UpperCenter,
        constraint: Flexible,
        constraint_count: 1,
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `layout_type` | 枚举 | `Vertical` | 布局类型 |
| `padding` | [f32, f32, f32, f32] | `[0.0, 0.0, 0.0, 0.0]` | 内边距 [left, right, top, bottom] |
| `spacing` | f32 | `0.0` | 子元素间距 |
| `child_alignment` | 枚举 | `UpperLeft` | 子元素对齐方式 |
| `constraint` | 枚举 | `Flexible` | 约束方式 |
| `constraint_count` | i32 | `1` | 约束数量，仅在约束为 `FixedColumnCount` 或 `FixedRowCount` 时生效 |

`layout_type` 枚举值：

- `Horizontal`：水平排列
- `Vertical`：垂直排列
- `Grid`：网格排列

`constraint` 枚举值：

- `Flexible`：灵活布局，子元素自动适应
- `FixedColumnCount`：固定列数
- `FixedRowCount`：固定行数

### UiScrollRect

UiScrollRect 是滚动区域组件，配合 UiMask 和 UiLayout 实现可滚动的列表、面板等。

```ron
Component({
    type: "UiScrollRect",
    properties: UiScrollRectProperties({
        horizontal: true,
        vertical: true,
        movement_type: Elastic,
        elasticity: 0.1,
        scroll_sensitivity: 1.0,
        content: "ScrollContent",
        viewport: "ScrollViewport",
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `horizontal` | bool | `true` | 是否支持水平滚动 |
| `vertical` | bool | `true` | 是否支持垂直滚动 |
| `movement_type` | 枚举 | `Elastic` | 滚动行为类型 |
| `elasticity` | f32 | `0.1` | 弹性回弹系数，仅在 `movement_type` 为 `Elastic` 时生效 |
| `scroll_sensitivity` | f32 | `1.0` | 滚动灵敏度 |
| `content` | String | `""` | 内容 Entity 名称引用 |
| `viewport` | String | `""` | 视口 Entity 名称引用 |

`movement_type` 枚举值：

- `Unrestricted`：无限制滚动
- `Elastic`：弹性滚动，超出边界后回弹
- `Clamped`：夹紧滚动，不可超出边界

### UiToggle

UiToggle 是开关组件，可单独使用或通过 ToggleGroup 实现单选组。

```ron
Component({
    type: "UiToggle",
    properties: UiToggleProperties({
        is_on: true,
        group: "DifficultyGroup",
        transition: ColorTint,
        on_value_changed: "on_difficulty_changed",
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `is_on` | bool | `false` | 当前开关状态 |
| `group` | String | `""` | ToggleGroup 名称引用，同一组内只能有一个为 on |
| `transition` | 枚举 | `ColorTint` | 过渡效果类型，同 UiButton |
| `on_value_changed` | String | `""` | 状态变化事件回调函数名 |

### UiSlider

UiSlider 是滑动条组件，用于调节数值。

```ron
Component({
    type: "UiSlider",
    properties: UiSliderProperties({
        value: 0.5,
        min_value: 0.0,
        max_value: 1.0,
        whole_numbers: false,
        direction: LeftToRight,
        on_value_changed: "on_volume_changed",
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `value` | f32 | `0.0` | 当前值 |
| `min_value` | f32 | `0.0` | 最小值 |
| `max_value` | f32 | `1.0` | 最大值 |
| `whole_numbers` | bool | `false` | 是否仅允许整数值 |
| `direction` | 枚举 | `LeftToRight` | 滑动方向 |
| `on_value_changed` | String | `""` | 值变化事件回调函数名 |

`direction` 枚举值：

- `LeftToRight`：从左到右
- `RightToLeft`：从右到左
- `BottomToTop`：从下到上
- `TopToBottom`：从上到下

### UiProgressBar

UiProgressBar 是进度条组件，用于显示进度。与 UiSlider 不同，UiProgressBar 不可交互，仅用于展示。

```ron
Component({
    type: "UiProgressBar",
    properties: UiProgressBarProperties({
        value: 0.75,
        min_value: 0.0,
        max_value: 1.0,
        fill_rect: "HealthBarFill",
        direction: LeftToRight,
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `value` | f32 | `0.0` | 当前值 |
| `min_value` | f32 | `0.0` | 最小值 |
| `max_value` | f32 | `1.0` | 最大值 |
| `fill_rect` | String | `""` | 填充区域 Entity 名称引用 |
| `direction` | 枚举 | `LeftToRight` | 填充方向，同 UiSlider |

### UiRawImage

UiRawImage 用于显示原始纹理，不经过 Sprite 处理，支持自定义 UV 矩形。适合显示 RenderTexture、视频画面、小地图等。

```ron
Component({
    type: "UiRawImage",
    properties: UiRawImageProperties({
        texture: "assets/textures/ui/minimap_render.png",
        color: [1.0, 1.0, 1.0, 1.0],
        uv_rect: [0.0, 0.0, 1.0, 1.0],
        material: "",
    }),
})
```

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `texture` | String | `""` | 原始纹理资源路径 |
| `color` | [f32, f32, f32, f32] | `[1.0, 1.0, 1.0, 1.0]` | 颜色 tint [r, g, b, a] |
| `uv_rect` | [f32, f32, f32, f32] | `[0.0, 0.0, 1.0, 1.0]` | UV 矩形 [x, y, width, height] |
| `material` | String | `""` | UiMaterial 资源路径 |

## UiMaterial 引用机制

UiImage 和 UiText 可通过 `material` 字段引用 *.material 文件，实现自定义着色器特效。*.material 文件中的 `shader` 字段再引用 *.shader 文件，形成二级引用链。

### 引用链路

```
UiImage.material → dissolve.material → dissolve.shader
```

### UI 专用着色器类型

| 着色器类型 | 说明 | 典型用途 |
|-----------|------|---------|
| `UiUnlit` | 基础 UI 渲染着色器，无光照计算 | 默认 UI 渲染、2D 精灵 |
| `UiSdf` | SDF 文字渲染着色器，支持高质量缩放 | 文字渲染、矢量图标 |
| `UiCustom` | 自定义特效着色器，完全可编程 | 溶解、扭曲、发光等特效 |

### 示例：溶解特效按钮

UiImage 引用溶解材质：

```ron
Component({
    type: "UiImage",
    properties: UiImageProperties({
        sprite: "assets/textures/ui/button.png",
        color: [1.0, 1.0, 1.0, 1.0],
        material: "assets/materials/ui/dissolve.material",
        image_type: Simple,
        fill_method: Horizontal,
        fill_amount: 1.0,
        preserve_aspect: false,
    }),
})
```

dissolve.material 文件内容（参见 *.material 格式规范）：

```ron
MaterialFile({
    version: "1.0",
    material: Material({
        name: "DissolveEffect",
        shader: "assets/shaders/ui/dissolve.shader",
        properties: [
            MaterialProperty({
                name: "dissolve_threshold",
                type: "float",
                value: "0.0",
            }),
            MaterialProperty({
                name: "edge_color",
                type: "color",
                value: "[1.0, 0.5, 0.0, 1.0]",
            }),
        ],
    }),
    dependencies: [
        Dependency({
            path: "assets/shaders/ui/dissolve.shader",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0100",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "b3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b866",
})
```

## Game UI 动画系统

Game UI 与 *.anim 文件无缝集成，动画轨道目标为 UI 组件属性，实现 UI 动画效果。

### 动画轨道目标

UI 组件属性可作为动画轨道的目标，格式为 `组件名.属性名`：

| 组件 | 可动画属性 | 说明 |
|------|-----------|------|
| UiRectTransform | `anchor_min`、`anchor_max`、`offset_min`、`offset_max`、`pivot`、`size_delta` | 布局动画 |
| UiImage | `color`、`fill_amount` | 颜色渐变、填充动画 |
| UiText | `text`、`color`、`font_size` | 文字变化、颜色渐变 |
| UiCanvas | `sort_order` | 层级切换 |

### UI 专用缓动函数

除标准缓动函数外，UI 动画支持以下专用缓动：

- `ui_bounce`：弹跳效果，适合按钮反馈
- `ui_back_in` / `ui_back_out` / `ui_back_in_out`：回弹效果，适合面板弹出
- `ui_elastic_in` / `ui_elastic_out` / `ui_elastic_in_out`：弹性效果，适合列表项

### 示例：按钮点击动画

```ron
AnimationFile({
    version: "1.0",
    animation: Animation({
        name: "ButtonClick",
        description: "Button click scale animation",
        duration: 0.3,
        loop: false,
        speed: 1.0,
        blend_time: 0.05,
    }),
    tracks: [
        Track({
            name: "UiRectTransform",
            target: "ui_rect_transform",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        size_delta: [200.0, 60.0],
                    }),
                    easing: "ui_bounce",
                }),
                Keyframe({
                    time: 0.1,
                    properties: Properties({
                        size_delta: [180.0, 54.0],
                    }),
                    easing: "ui_bounce",
                }),
                Keyframe({
                    time: 0.3,
                    properties: Properties({
                        size_delta: [200.0, 60.0],
                    }),
                    easing: "ui_bounce",
                }),
            ],
        }),
        Track({
            name: "UiImage",
            target: "ui_image",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        color: [1.0, 1.0, 1.0, 1.0],
                    }),
                    easing: "ease_in_out",
                }),
                Keyframe({
                    time: 0.1,
                    properties: Properties({
                        color: [0.8, 0.8, 0.8, 1.0],
                    }),
                    easing: "ease_in_out",
                }),
                Keyframe({
                    time: 0.3,
                    properties: Properties({
                        color: [1.0, 1.0, 1.0, 1.0],
                    }),
                    easing: "ease_in_out",
                }),
            ],
        }),
    ],
    events: [],
    dependencies: [],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "c4b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b877",
})
```

### 示例：面板滑入动画

```ron
AnimationFile({
    version: "1.0",
    animation: Animation({
        name: "PanelSlideIn",
        description: "Panel slide in from right",
        duration: 0.4,
        loop: false,
        speed: 1.0,
        blend_time: 0.1,
    }),
    tracks: [
        Track({
            name: "UiRectTransform",
            target: "ui_rect_transform",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        offset_min: [1920.0, 0.0],
                        offset_max: [1920.0, 0.0],
                    }),
                    easing: "ui_back_out",
                }),
                Keyframe({
                    time: 0.4,
                    properties: Properties({
                        offset_min: [0.0, 0.0],
                        offset_max: [0.0, 0.0],
                    }),
                    easing: "ui_back_out",
                }),
            ],
        }),
    ],
    events: [],
    dependencies: [],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "d5b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b888",
})
```

### 示例：进度条填充动画

```ron
AnimationFile({
    version: "1.0",
    animation: Animation({
        name: "ProgressBarFill",
        description: "Progress bar fill animation",
        duration: 1.0,
        loop: false,
        speed: 1.0,
        blend_time: 0.0,
    }),
    tracks: [
        Track({
            name: "UiImage",
            target: "ui_image",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        fill_amount: 0.0,
                    }),
                    easing: "ease_out",
                }),
                Keyframe({
                    time: 1.0,
                    properties: Properties({
                        fill_amount: 1.0,
                    }),
                    easing: "ease_out",
                }),
            ],
        }),
    ],
    events: [],
    dependencies: [],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e6b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b899",
})
```

## 完整示例文件

### 示例 1：游戏 HUD（ScreenSpace 模式）

包含血条（UiProgressBar）、弹药数（UiText）、准星（UiImage）、小地图框（UiRawImage）。

```ron
// 游戏 HUD 示例
GameUiFile({
    version: "1.0",
    gameui: GameUi({
        name: "GameHUD",
        description: "Main game HUD with health bar, ammo, crosshair and minimap",
        canvas_mode: ScreenSpace,
    }),
    entities: [
        Entity({
            id: 1,
            name: "HUDCanvas",
            parent_id: None,
            components: [
                Component({
                    type: "UiCanvas",
                    properties: UiCanvasProperties({
                        canvas_mode: ScreenSpace,
                        sort_order: 0,
                        reference_resolution: [1920, 1080],
                        match_mode: Expand,
                        pixel_perfect: false,
                    }),
                }),
            ],
        }),
        Entity({
            id: 2,
            name: "HealthBarContainer",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 1.0],
                        anchor_max: [0.0, 1.0],
                        offset_min: [20.0, -60.0],
                        offset_max: [220.0, -20.0],
                        pivot: [0.0, 1.0],
                        size_delta: [200.0, 40.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/health_bar_bg.png",
                        color: [0.2, 0.2, 0.2, 0.8],
                        material: "",
                        image_type: Sliced,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
                Component({
                    type: "UiProgressBar",
                    properties: UiProgressBarProperties({
                        value: 0.75,
                        min_value: 0.0,
                        max_value: 1.0,
                        fill_rect: "HealthBarFill",
                        direction: LeftToRight,
                    }),
                }),
            ],
        }),
        Entity({
            id: 3,
            name: "HealthBarFill",
            parent_id: Some(2),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [2.0, 2.0],
                        offset_max: [-2.0, -2.0],
                        pivot: [0.0, 0.5],
                        size_delta: [0.0, 0.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/health_bar_fill.png",
                        color: [0.0, 0.8, 0.2, 1.0],
                        material: "",
                        image_type: Simple,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
            ],
        }),
        Entity({
            id: 4,
            name: "AmmoText",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [1.0, 1.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [-120.0, -50.0],
                        offset_max: [-20.0, -10.0],
                        pivot: [1.0, 1.0],
                        size_delta: [100.0, 40.0],
                    }),
                }),
                Component({
                    type: "UiText",
                    properties: UiTextProperties({
                        text: "30 / 90",
                        font: "assets/fonts/hud_font.ttf",
                        font_size: 28.0,
                        font_style: Bold,
                        color: [1.0, 1.0, 1.0, 1.0],
                        alignment: MiddleRight,
                        overflow: Overflow,
                        line_spacing: 1.0,
                        material: "",
                    }),
                }),
            ],
        }),
        Entity({
            id: 5,
            name: "Crosshair",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.5, 0.5],
                        anchor_max: [0.5, 0.5],
                        offset_min: [-16.0, -16.0],
                        offset_max: [16.0, 16.0],
                        pivot: [0.5, 0.5],
                        size_delta: [32.0, 32.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/crosshair.png",
                        color: [1.0, 1.0, 1.0, 0.9],
                        material: "",
                        image_type: Simple,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: true,
                    }),
                }),
            ],
        }),
        Entity({
            id: 6,
            name: "MinimapFrame",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [1.0, 1.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [-220.0, -220.0],
                        offset_max: [-20.0, -20.0],
                        pivot: [1.0, 1.0],
                        size_delta: [200.0, 200.0],
                    }),
                }),
                Component({
                    type: "UiRawImage",
                    properties: UiRawImageProperties({
                        texture: "assets/textures/ui/minimap_render.png",
                        color: [1.0, 1.0, 1.0, 1.0],
                        uv_rect: [0.0, 0.0, 1.0, 1.0],
                        material: "",
                    }),
                }),
                Component({
                    type: "UiMask",
                    properties: UiMaskProperties({
                        show_mask_graphic: false,
                        sprite: "assets/textures/ui/circle_mask.png",
                    }),
                }),
            ],
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/textures/ui/health_bar_bg.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0001",
        }),
        Dependency({
            path: "assets/textures/ui/health_bar_fill.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0002",
        }),
        Dependency({
            path: "assets/fonts/hud_font.ttf",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0003",
        }),
        Dependency({
            path: "assets/textures/ui/crosshair.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0004",
        }),
        Dependency({
            path: "assets/textures/ui/minimap_render.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0005",
        }),
        Dependency({
            path: "assets/textures/ui/circle_mask.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0006",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "f7b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b900",
})
```

### 示例 2：角色头顶血条（WorldSpace 模式）

包含 Canvas（WorldSpace）、血条背景（UiImage）、血条填充（UiImage）。

```ron
// 角色头顶血条示例
GameUiFile({
    version: "1.0",
    gameui: GameUi({
        name: "EnemyHealthBar",
        description: "Enemy health bar floating above head",
        canvas_mode: WorldSpace,
    }),
    entities: [
        Entity({
            id: 1,
            name: "HealthBarCanvas",
            parent_id: None,
            components: [
                Component({
                    type: "UiCanvas",
                    properties: UiCanvasProperties({
                        canvas_mode: WorldSpace,
                        sort_order: 0,
                        reference_resolution: [200, 30],
                        match_mode: Expand,
                        pixel_perfect: false,
                        width: 2.0,
                        height: 0.3,
                        distance: 0.0,
                        event_camera: "MainCamera",
                        sorting_layer: "UI",
                        sorting_order: 1,
                    }),
                }),
            ],
        }),
        Entity({
            id: 2,
            name: "HealthBarBg",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [0.0, 0.0],
                        offset_max: [0.0, 0.0],
                        pivot: [0.5, 0.5],
                        size_delta: [0.0, 0.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/health_bar_bg.png",
                        color: [0.15, 0.15, 0.15, 0.9],
                        material: "",
                        image_type: Sliced,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
            ],
        }),
        Entity({
            id: 3,
            name: "HealthBarFill",
            parent_id: Some(2),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [2.0, 2.0],
                        offset_max: [-2.0, -2.0],
                        pivot: [0.0, 0.5],
                        size_delta: [0.0, 0.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/health_bar_fill.png",
                        color: [0.9, 0.1, 0.1, 1.0],
                        material: "",
                        image_type: Simple,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
                Component({
                    type: "UiProgressBar",
                    properties: UiProgressBarProperties({
                        value: 0.6,
                        min_value: 0.0,
                        max_value: 1.0,
                        fill_rect: "HealthBarFill",
                        direction: LeftToRight,
                    }),
                }),
            ],
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/textures/ui/health_bar_bg.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0010",
        }),
        Dependency({
            path: "assets/textures/ui/health_bar_fill.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0011",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "a8b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b911",
})
```

### 示例 3：游戏主菜单（ScreenSpace 模式）

包含背景图、标题文字、开始按钮、设置按钮、退出按钮。

```ron
// 游戏主菜单示例
GameUiFile({
    version: "1.0",
    gameui: GameUi({
        name: "MainMenu",
        description: "Game main menu with title and buttons",
        canvas_mode: ScreenSpace,
    }),
    entities: [
        Entity({
            id: 1,
            name: "MenuCanvas",
            parent_id: None,
            components: [
                Component({
                    type: "UiCanvas",
                    properties: UiCanvasProperties({
                        canvas_mode: ScreenSpace,
                        sort_order: 0,
                        reference_resolution: [1920, 1080],
                        match_mode: Expand,
                        pixel_perfect: false,
                    }),
                }),
            ],
        }),
        Entity({
            id: 2,
            name: "Background",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [0.0, 0.0],
                        offset_max: [0.0, 0.0],
                        pivot: [0.5, 0.5],
                        size_delta: [0.0, 0.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/menu_bg.png",
                        color: [1.0, 1.0, 1.0, 1.0],
                        material: "",
                        image_type: Simple,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
            ],
        }),
        Entity({
            id: 3,
            name: "TitleText",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.5, 1.0],
                        anchor_max: [0.5, 1.0],
                        offset_min: [-300.0, -200.0],
                        offset_max: [300.0, -100.0],
                        pivot: [0.5, 1.0],
                        size_delta: [600.0, 100.0],
                    }),
                }),
                Component({
                    type: "UiText",
                    properties: UiTextProperties({
                        text: "GG Game",
                        font: "assets/fonts/title_font.ttf",
                        font_size: 72.0,
                        font_style: Bold,
                        color: [1.0, 0.85, 0.2, 1.0],
                        alignment: MiddleCenter,
                        overflow: Overflow,
                        line_spacing: 1.0,
                        material: "assets/materials/ui/sdf_glow.material",
                    }),
                }),
            ],
        }),
        Entity({
            id: 4,
            name: "ButtonContainer",
            parent_id: Some(1),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.5, 0.5],
                        anchor_max: [0.5, 0.5],
                        offset_min: [-120.0, -120.0],
                        offset_max: [120.0, 60.0],
                        pivot: [0.5, 0.5],
                        size_delta: [240.0, 180.0],
                    }),
                }),
                Component({
                    type: "UiLayout",
                    properties: UiLayoutProperties({
                        layout_type: Vertical,
                        padding: [0.0, 0.0, 10.0, 10.0],
                        spacing: 15.0,
                        child_alignment: MiddleCenter,
                        constraint: Flexible,
                        constraint_count: 1,
                    }),
                }),
            ],
        }),
        Entity({
            id: 5,
            name: "StartButton",
            parent_id: Some(4),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [0.0, 0.0],
                        offset_min: [0.0, 0.0],
                        offset_max: [240.0, 50.0],
                        pivot: [0.0, 0.0],
                        size_delta: [240.0, 50.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/button.png",
                        color: [1.0, 1.0, 1.0, 1.0],
                        material: "",
                        image_type: Sliced,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
                Component({
                    type: "UiButton",
                    properties: UiButtonProperties({
                        interactable: true,
                        transition: ColorTint,
                        target_graphic: "StartButton",
                        colors: ColorBlock({
                            normal_color: [1.0, 1.0, 1.0, 1.0],
                            highlighted_color: [0.96, 0.96, 0.96, 1.0],
                            pressed_color: [0.78, 0.78, 0.78, 1.0],
                            disabled_color: [0.78, 0.78, 0.78, 0.5],
                            fade_duration: 0.1,
                        }),
                        on_click: "on_start_game",
                    }),
                }),
            ],
        }),
        Entity({
            id: 6,
            name: "StartButtonText",
            parent_id: Some(5),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [0.0, 0.0],
                        offset_max: [0.0, 0.0],
                        pivot: [0.5, 0.5],
                        size_delta: [0.0, 0.0],
                    }),
                }),
                Component({
                    type: "UiText",
                    properties: UiTextProperties({
                        text: "Start Game",
                        font: "assets/fonts/main_font.ttf",
                        font_size: 28.0,
                        font_style: Bold,
                        color: [1.0, 1.0, 1.0, 1.0],
                        alignment: MiddleCenter,
                        overflow: Overflow,
                        line_spacing: 1.0,
                        material: "",
                    }),
                }),
            ],
        }),
        Entity({
            id: 7,
            name: "SettingsButton",
            parent_id: Some(4),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [0.0, 0.0],
                        offset_min: [0.0, 0.0],
                        offset_max: [240.0, 50.0],
                        pivot: [0.0, 0.0],
                        size_delta: [240.0, 50.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/button.png",
                        color: [1.0, 1.0, 1.0, 1.0],
                        material: "",
                        image_type: Sliced,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
                Component({
                    type: "UiButton",
                    properties: UiButtonProperties({
                        interactable: true,
                        transition: ColorTint,
                        target_graphic: "SettingsButton",
                        colors: ColorBlock({
                            normal_color: [1.0, 1.0, 1.0, 1.0],
                            highlighted_color: [0.96, 0.96, 0.96, 1.0],
                            pressed_color: [0.78, 0.78, 0.78, 1.0],
                            disabled_color: [0.78, 0.78, 0.78, 0.5],
                            fade_duration: 0.1,
                        }),
                        on_click: "on_open_settings",
                    }),
                }),
            ],
        }),
        Entity({
            id: 8,
            name: "SettingsButtonText",
            parent_id: Some(7),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [0.0, 0.0],
                        offset_max: [0.0, 0.0],
                        pivot: [0.5, 0.5],
                        size_delta: [0.0, 0.0],
                    }),
                }),
                Component({
                    type: "UiText",
                    properties: UiTextProperties({
                        text: "Settings",
                        font: "assets/fonts/main_font.ttf",
                        font_size: 28.0,
                        font_style: Normal,
                        color: [1.0, 1.0, 1.0, 1.0],
                        alignment: MiddleCenter,
                        overflow: Overflow,
                        line_spacing: 1.0,
                        material: "",
                    }),
                }),
            ],
        }),
        Entity({
            id: 9,
            name: "QuitButton",
            parent_id: Some(4),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [0.0, 0.0],
                        offset_min: [0.0, 0.0],
                        offset_max: [240.0, 50.0],
                        pivot: [0.0, 0.0],
                        size_delta: [240.0, 50.0],
                    }),
                }),
                Component({
                    type: "UiImage",
                    properties: UiImageProperties({
                        sprite: "assets/textures/ui/button.png",
                        color: [1.0, 1.0, 1.0, 1.0],
                        material: "",
                        image_type: Sliced,
                        fill_method: Horizontal,
                        fill_amount: 1.0,
                        preserve_aspect: false,
                    }),
                }),
                Component({
                    type: "UiButton",
                    properties: UiButtonProperties({
                        interactable: true,
                        transition: ColorTint,
                        target_graphic: "QuitButton",
                        colors: ColorBlock({
                            normal_color: [1.0, 1.0, 1.0, 1.0],
                            highlighted_color: [0.96, 0.96, 0.96, 1.0],
                            pressed_color: [0.78, 0.78, 0.78, 1.0],
                            disabled_color: [0.78, 0.78, 0.78, 0.5],
                            fade_duration: 0.1,
                        }),
                        on_click: "on_quit_game",
                    }),
                }),
            ],
        }),
        Entity({
            id: 10,
            name: "QuitButtonText",
            parent_id: Some(9),
            components: [
                Component({
                    type: "UiRectTransform",
                    properties: UiRectTransformProperties({
                        anchor_min: [0.0, 0.0],
                        anchor_max: [1.0, 1.0],
                        offset_min: [0.0, 0.0],
                        offset_max: [0.0, 0.0],
                        pivot: [0.5, 0.5],
                        size_delta: [0.0, 0.0],
                    }),
                }),
                Component({
                    type: "UiText",
                    properties: UiTextProperties({
                        text: "Quit",
                        font: "assets/fonts/main_font.ttf",
                        font_size: 28.0,
                        font_style: Normal,
                        color: [1.0, 1.0, 1.0, 1.0],
                        alignment: MiddleCenter,
                        overflow: Overflow,
                        line_spacing: 1.0,
                        material: "",
                    }),
                }),
            ],
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/textures/ui/menu_bg.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0020",
        }),
        Dependency({
            path: "assets/textures/ui/button.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0021",
        }),
        Dependency({
            path: "assets/fonts/title_font.ttf",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0022",
        }),
        Dependency({
            path: "assets/fonts/main_font.ttf",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0023",
        }),
        Dependency({
            path: "assets/materials/ui/sdf_glow.material",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0024",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "b9b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b922",
})
```

## 与 Editor UI 的边界

### 明确声明

- **\*.gameui** 仅用于游戏运行时 UI，基于 ECS/GameObject + Canvas 体系
- **\*.vx** 仅用于编辑器界面，基于 DOM 模型

两者体系完全不同，不可混用。

### 对比表

| 特性 | *.gameui | *.vx |
|------|----------|------|
| 用途 | 游戏运行时 UI | 编辑器界面 |
| 体系 | ECS/GameObject + Canvas | DOM 模型 |
| 布局 | UiRectTransform（锚点/偏移） | CSS Flex/Grid |
| 渲染 | GPU 渲染，着色器驱动 | WebView 渲染 |
| 3D 空间 | 支持（WorldSpace 模式） | 不支持 |
| 着色器特效 | 支持（UiMaterial/UiCustom） | 不支持 |
| 动画 | *.anim 集成，关键帧动画 | CSS Transition/Animation |
| 交互 | UiButton/UiToggle/UiSlider | DOM 事件 |
| 性能 | 高性能，适合复杂游戏 UI | 适合工具界面，开发效率高 |
| 自适应 | reference_resolution + match_mode | 响应式布局 |
| 文件格式 | RON | XML/HTML |

## 最佳实践

1. **Canvas 组织**：每个独立的 UI 界面使用一个 *.gameui 文件，避免单个文件过于庞大。
2. **层次结构**：保持实体层次结构清晰，Canvas → Panel → Element，避免过深的嵌套。
3. **锚点使用**：合理使用锚点实现自适应布局，避免硬编码像素位置。
4. **图集合并**：将频繁使用的 UI 纹理合并为图集，减少 Draw Call。
5. **材质复用**：相同特效的 UI 元素共享同一个 UiMaterial 实例。
6. **WorldSpace 优化**：WorldSpace 模式的 UI 注意控制 Canvas 尺寸和 event_camera 设置，避免不必要的射线检测开销。
7. **动画性能**：UI 动画尽量使用 UiImage.color 和 UiRectTransform.offset 等轻量属性，避免频繁修改 UiText.text。
8. **依赖管理**：定期检查并清理无效的依赖关系。
9. **哈希计算**：使用 SHA1 算法计算 Game UI 文件的哈希值。
10. **版本控制**：将 *.gameui 文件纳入版本控制系统，确保团队协作时的一致性。

## 总结

*.gameui 文件格式为 GG 游戏引擎提供了一种统一、有效的方式来管理游戏运行时 UI。通过基于 ECS 的组件体系、Canvas 渲染模式、UiRectTransform 布局系统，以及与 UiMaterial 和 *.anim 的深度集成，它解决了游戏运行时 UI 的构建、渲染和动画问题。

这种设计使得 GG 游戏引擎能够：

- 快速创建和管理游戏运行时 UI
- 支持多种渲染模式（ScreenSpace、WorldSpace、CameraSpace）
- 通过 UiRectTransform 实现灵活的自适应布局
- 通过 UiMaterial 和自定义着色器实现丰富的 UI 特效
- 与动画系统无缝集成，支持 UI 专用缓动函数
- 与编辑器 UI（*.vx）明确分离，各司其职

Game UI 系统是 GG 游戏引擎中重要的组成部分，为游戏开发者提供了一种高效、灵活的方式来构建和管理游戏运行时界面。
