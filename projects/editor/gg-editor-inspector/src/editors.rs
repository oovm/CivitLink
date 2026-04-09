//! Galgame 组件专用属性编辑器
//! 为 DialogueNode、PortraitState、AudioControl 和 SceneBackground 提供类型化编辑器

/// 属性编辑器 trait
///
/// 定义组件属性编辑器的通用接口，每个编辑器对应一种组件类型。
pub trait PropertyEditor {
    /// 获取编辑器显示名称
    fn display_name(&self) -> &str;

    /// 获取对应的组件类型标识
    fn component_type_id(&self) -> &str;
}

/// DialogueNode 属性编辑器
///
/// 用于编辑对话节点的文本、说话者、命令和选项等属性。
pub struct DialogueNodeEditor;

impl PropertyEditor for DialogueNodeEditor {
    fn display_name(&self) -> &str {
        "Dialogue Node"
    }

    fn component_type_id(&self) -> &str {
        "DialogueNode"
    }
}

/// PortraitState 属性编辑器
///
/// 用于编辑立绘状态的位置、表情、缩放和透明度等属性。
pub struct PortraitStateEditor;

impl PropertyEditor for PortraitStateEditor {
    fn display_name(&self) -> &str {
        "Portrait State"
    }

    fn component_type_id(&self) -> &str {
        "PortraitState"
    }
}

/// AudioControl 属性编辑器
///
/// 用于编辑音频控制的 BGM 路径、音量和淡入淡出等属性。
pub struct AudioControlEditor;

impl PropertyEditor for AudioControlEditor {
    fn display_name(&self) -> &str {
        "Audio Control"
    }

    fn component_type_id(&self) -> &str {
        "AudioControl"
    }
}

/// SceneBackground 属性编辑器
///
/// 用于编辑场景背景的资源路径、转场效果和氛围滤镜等属性。
pub struct SceneBackgroundEditor;

impl PropertyEditor for SceneBackgroundEditor {
    fn display_name(&self) -> &str {
        "Scene Background"
    }

    fn component_type_id(&self) -> &str {
        "SceneBackground"
    }
}
