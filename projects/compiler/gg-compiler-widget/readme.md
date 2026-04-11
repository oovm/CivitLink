# gg-compiler-widget

GG 引擎 Widget 编译器模块，负责将 `.widget` 文件（Editor UI 单文件组件）编译为可运行时加载的 Widget 产物。

## 功能

- Widget 文件解析（template/script/style 三部分拆分）
- Template IR 定义与语义分析（组件类型检查、属性验证、数据绑定分析、事件绑定验证）
- Style IR 定义与编译（SCSS 子集、Tailwind CSS 子集、主题变量解析）
- WidgetArtifact 产物定义与打包
- ComponentRegistry 组件注册系统
- WidgetTransformer 编译管线集成
