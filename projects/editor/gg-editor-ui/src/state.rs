//! 状态管理模块
//!
//! 提供响应式状态管理功能，支持信号和订阅机制

/// 响应式信号，支持 get/set/subscribe 操作
pub struct Signal<T> {
    /// 当前值
    value: T,
    /// 订阅者列表
    subscribers: Vec<Box<dyn Fn(&T)>>,
}

impl<T> Signal<T> {
    /// 创建新的响应式信号
    pub fn new(value: T) -> Self {
        Self { value, subscribers: Vec::new() }
    }

    /// 获取当前值的引用
    pub fn get(&self) -> &T {
        &self.value
    }

    /// 订阅值变化
    pub fn subscribe(&mut self, handler: Box<dyn Fn(&T)>) {
        self.subscribers.push(handler);
    }
}

impl<T: Clone> Signal<T> {
    /// 设置新值并通知订阅者
    pub fn set(&mut self, value: T) {
        self.value = value;
        for handler in &self.subscribers {
            handler(&self.value);
        }
    }
}

/// 计算属性，基于其他信号的值计算
pub struct Computed<T, F: Fn() -> T> {
    /// 计算函数
    compute: F,
    /// 缓存的值
    cached_value: Option<T>,
    /// 订阅者列表
    subscribers: Vec<Box<dyn Fn(&T)>>,
}

impl<T, F: Fn() -> T> Computed<T, F> {
    /// 创建新的计算属性
    pub fn new(compute: F) -> Self {
        Self { compute, cached_value: None, subscribers: Vec::new() }
    }

    /// 获取计算值
    pub fn get(&mut self) -> &T {
        if self.cached_value.is_none() {
            self.cached_value = Some((self.compute)());
        }
        self.cached_value.as_ref().unwrap()
    }

    /// 订阅值变化
    pub fn subscribe(&mut self, handler: Box<dyn Fn(&T)>) {
        self.subscribers.push(handler);
    }

    /// 手动更新计算值
    pub fn update(&mut self) {
        let new_value = (self.compute)();
        self.cached_value = Some(new_value);
        for handler in &self.subscribers {
            handler(self.cached_value.as_ref().unwrap());
        }
    }
}
