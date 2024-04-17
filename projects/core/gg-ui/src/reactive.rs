use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU64, Ordering},
};

/// 可克隆的对象 trait
pub trait CloneAny: Any + Send + Sync {
    /// 克隆为 Box<dyn CloneAny>
    fn clone_boxed(&self) -> Box<dyn CloneAny>;
}

impl<T: Any + Send + Sync + Clone + 'static> CloneAny for T {
    fn clone_boxed(&self) -> Box<dyn CloneAny> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CloneAny> {
    fn clone(&self) -> Self {
        (**self).clone_boxed()
    }
}

/// 属性值
#[derive(Clone)]
pub enum PropertyValue {
    /// 字符串值
    String(String),
    /// 数值
    Number(f64),
    /// 布尔值
    Boolean(bool),
    /// 对象值
    Object(Box<dyn CloneAny>),
    /// 数组值
    Array(Vec<PropertyValue>),
}

/// 信号唯一标识符
///
/// 每个信号在创建时自动分配一个全局唯一的 ID，
/// 用于依赖追踪系统中标识信号之间的依赖关系。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(u64);

impl SignalId {
    /// 分配下一个全局唯一的信号 ID
    fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        SignalId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// 全局依赖追踪器
///
/// 使用线程局部变量记录当前正在求值的 ComputedSignal 或 Effect
/// 读取了哪些信号，从而自动建立依赖关系。
struct DependencyTracker {
    /// 当前正在追踪的信号 ID 列表
    tracking: Vec<SignalId>,
    /// 当前正在求值的 ComputedSignal 或 Effect 的 ID
    tracker_id: Option<SignalId>,
}

impl DependencyTracker {
    /// 创建新的依赖追踪器
    fn new() -> Self {
        Self { tracking: Vec::new(), tracker_id: None }
    }
}

thread_local! {
    static DEPENDENCY_TRACKER: RefCell<DependencyTracker> = RefCell::new(DependencyTracker::new());
    static BATCH_DEPTH: RefCell<u32> = RefCell::new(0);
    static PENDING_EFFECTS: RefCell<Vec<SignalId>> = RefCell::new(Vec::new());
    static EFFECT_REGISTRY: RefCell<HashMap<SignalId, Box<dyn FnMut() + Send + Sync>>> = RefCell::new(HashMap::new());
    static COMPUTED_REGISTRY: RefCell<HashSet<SignalId>> = RefCell::new(HashSet::new());
    static DEPENDENCY_GRAPH: RefCell<HashMap<SignalId, HashSet<SignalId>>> = RefCell::new(HashMap::new());
    static REVERSE_GRAPH: RefCell<HashMap<SignalId, HashSet<SignalId>>> = RefCell::new(HashMap::new());
    static DIRTY_COMPUTED: RefCell<HashSet<SignalId>> = RefCell::new(HashSet::new());
    static EVALUATION_STACK: RefCell<Vec<SignalId>> = RefCell::new(Vec::new());
}

/// 注册信号读取到当前追踪器
///
/// 如果当前有活跃的追踪器（即某个 ComputedSignal 或 Effect 正在求值），
/// 将被读取的信号 ID 记录到追踪列表中。
fn track_signal_read(signal_id: SignalId) {
    DEPENDENCY_TRACKER.with(|tracker| {
        let mut t = tracker.borrow_mut();
        if t.tracker_id.is_some() {
            t.tracking.push(signal_id);
        }
    });
}

/// 开始追踪信号依赖
///
/// 设置当前追踪器 ID，开始记录信号读取。
fn start_tracking(tracker_id: SignalId) {
    DEPENDENCY_TRACKER.with(|tracker| {
        let mut t = tracker.borrow_mut();
        t.tracker_id = Some(tracker_id);
        t.tracking.clear();
    });
}

/// 结束追踪并返回被追踪的信号 ID 集合
///
/// 清除追踪器状态，返回本次追踪期间读取的所有信号 ID。
fn stop_tracking() -> (Option<SignalId>, HashSet<SignalId>) {
    DEPENDENCY_TRACKER.with(|tracker| {
        let mut t = tracker.borrow_mut();
        let id = t.tracker_id.take();
        let deps: HashSet<SignalId> = t.tracking.drain(..).collect();
        (id, deps)
    })
}

/// 更新依赖图
///
/// 将 tracker_id（ComputedSignal 或 Effect）对其依赖信号的映射关系
/// 更新到全局依赖图和反向依赖图中。
fn update_dependencies(tracker_id: SignalId, deps: &HashSet<SignalId>) {
    DEPENDENCY_GRAPH.with(|graph| {
        let mut g = graph.borrow_mut();
        g.insert(tracker_id, deps.clone());
    });

    REVERSE_GRAPH.with(|reverse| {
        let mut r = reverse.borrow_mut();
        for &dep_id in deps {
            r.entry(dep_id).or_default().insert(tracker_id);
        }
    });
}

/// 移除某个追踪者的旧依赖关系
fn remove_old_dependencies(tracker_id: SignalId) {
    DEPENDENCY_GRAPH.with(|graph| {
        let mut g = graph.borrow_mut();
        if let Some(old_deps) = g.remove(&tracker_id) {
            REVERSE_GRAPH.with(|reverse| {
                let mut r = reverse.borrow_mut();
                for dep_id in old_deps {
                    if let Some(set) = r.get_mut(&dep_id) {
                        set.remove(&tracker_id);
                    }
                }
            });
        }
    });
}

/// 通知依赖图中的下游节点
///
/// 当一个信号的值发生变化时，将所有依赖该信号的 ComputedSignal 标记为脏，
/// 并调度依赖该信号的 Effect 执行。
fn notify_dependents(signal_id: SignalId) {
    let dependents: HashSet<SignalId>;
    REVERSE_GRAPH.with(|reverse| {
        dependents = reverse.borrow().get(&signal_id).cloned().unwrap_or_default();
    });

    for dep_id in &dependents {
        COMPUTED_REGISTRY.with(|registry| {
            if registry.borrow().contains(dep_id) {
                DIRTY_COMPUTED.with(|dirty| {
                    dirty.borrow_mut().insert(*dep_id);
                });
            }
        });

        EFFECT_REGISTRY.with(|registry| {
            if registry.borrow().contains_key(dep_id) {
                PENDING_EFFECTS.with(|pending| {
                    pending.borrow_mut().push(*dep_id);
                });
            }
        });
    }
}

/// 检测循环依赖
///
/// 在求值栈中检查是否已存在指定的信号 ID，
/// 如果存在则表示存在循环依赖，抛出 panic。
fn check_cycle(signal_id: SignalId) {
    EVALUATION_STACK.with(|stack| {
        if stack.borrow().contains(&signal_id) {
            panic!(
                "Circular dependency detected: Signal {:?} is already being evaluated. \
                 This creates an infinite loop in the reactive dependency graph.",
                signal_id
            );
        }
    });
}

/// 执行待处理的 Effect
///
/// 取出所有待处理的 Effect ID 并执行对应的回调函数。
/// 如果当前处于 batch 模式（batch_depth > 0），则延迟执行。
fn run_pending_effects() {
    BATCH_DEPTH.with(|depth| {
        if *depth.borrow() > 0 {
            return;
        }
    });

    let effect_ids: Vec<SignalId>;
    PENDING_EFFECTS.with(|pending| {
        effect_ids = pending.borrow_mut().drain(..).collect();
    });

    for id in effect_ids {
        EFFECT_REGISTRY.with(|registry| {
            let mut reg = registry.borrow_mut();
            if let Some(callback) = reg.get_mut(&id) {
                callback();
            }
        });
    }
}

/// 响应式信号，支持 get/set/subscribe 操作及依赖追踪
///
/// 当信号被 ComputedSignal 或 Effect 读取时，自动建立依赖关系；
/// 当信号值变化时，自动通知所有依赖者。
pub struct Signal<T> {
    /// 信号唯一标识符
    id: SignalId,
    /// 当前值
    value: T,
    /// 订阅者列表
    subscribers: Vec<Box<dyn Fn(&T) + Send + Sync>>,
}

impl<T> Signal<T> {
    /// 创建新的响应式信号
    pub fn new(value: T) -> Self {
        Self { id: SignalId::next(), value, subscribers: Vec::new() }
    }

    /// 获取信号的唯一标识符
    pub fn id(&self) -> SignalId {
        self.id
    }

    /// 获取当前值的引用
    ///
    /// 如果当前有活跃的依赖追踪器，自动注册此信号为被追踪的依赖。
    pub fn get(&self) -> &T {
        track_signal_read(self.id);
        &self.value
    }

    /// 订阅值变化
    pub fn subscribe(&mut self, handler: Box<dyn Fn(&T) + Send + Sync>) {
        self.subscribers.push(handler);
    }
}

impl<T: Clone> Signal<T> {
    /// 设置新值并通知订阅者和依赖者
    ///
    /// 通知所有订阅者（回调函数），然后通过依赖图通知
    /// 依赖此信号的 ComputedSignal 和 Effect。
    pub fn set(&mut self, value: T) {
        self.value = value;
        for handler in &self.subscribers {
            handler(&self.value);
        }
        notify_dependents(self.id);
        run_pending_effects();
    }
}

/// 计算信号，其值由其他信号派生而来
///
/// ComputedSignal 在首次 `get()` 时执行计算函数并缓存结果，
/// 同时自动追踪计算函数中读取的所有信号作为依赖。
/// 当依赖信号变化时，ComputedSignal 被标记为脏，
/// 下次 `get()` 时重新计算。
pub struct ComputedSignal<T> {
    /// 计算信号的唯一标识符
    id: SignalId,
    /// 计算函数
    compute_fn: Box<dyn Fn() -> T + Send + Sync>,
    /// 缓存的计算结果
    cached_value: Option<T>,
    /// 是否需要重新计算
    dirty: bool,
}

impl<T> ComputedSignal<T> {
    /// 创建新的计算信号
    ///
    /// 计算函数在首次 `get()` 时执行，自动追踪其中读取的信号依赖。
    pub fn new(compute_fn: Box<dyn Fn() -> T + Send + Sync>) -> Self {
        let id = SignalId::next();
        COMPUTED_REGISTRY.with(|registry| {
            registry.borrow_mut().insert(id);
        });
        Self { id, compute_fn, cached_value: None, dirty: true }
    }

    /// 获取计算信号的唯一标识符
    pub fn id(&self) -> SignalId {
        self.id
    }
}

impl<T: Clone> ComputedSignal<T> {
    /// 获取计算信号的当前值
    ///
    /// 如果信号被标记为脏或尚未计算过，则重新执行计算函数。
    /// 计算期间自动追踪依赖关系，并在计算完成后更新依赖图。
    /// 如果检测到循环依赖，将触发 panic。
    pub fn get(&mut self) -> T {
        if self.dirty || self.cached_value.is_none() {
            check_cycle(self.id);

            EVALUATION_STACK.with(|stack| {
                stack.borrow_mut().push(self.id);
            });

            remove_old_dependencies(self.id);
            start_tracking(self.id);

            let new_value = (self.compute_fn)();

            let (tracker_id, deps) = stop_tracking();

            if let Some(tid) = tracker_id {
                update_dependencies(tid, &deps);
            }

            EVALUATION_STACK.with(|stack| {
                let mut s = stack.borrow_mut();
                s.pop();
            });

            self.cached_value = Some(new_value);
            self.dirty = false;

            DIRTY_COMPUTED.with(|dirty| {
                dirty.borrow_mut().remove(&self.id);
            });
        }

        self.cached_value.clone().unwrap()
    }

    /// 将计算信号标记为脏
    ///
    /// 下次调用 `get()` 时将重新执行计算函数。
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// 检查计算信号是否需要重新计算
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl<T> Drop for ComputedSignal<T> {
    fn drop(&mut self) {
        COMPUTED_REGISTRY.with(|registry| {
            registry.borrow_mut().remove(&self.id);
        });
        remove_old_dependencies(self.id);
    }
}

/// 创建 Effect 副作用
///
/// Effect 会立即执行一次回调函数，并自动追踪其中读取的信号依赖。
/// 当依赖信号变化时，Effect 回调自动重新执行。
///
/// 返回 Effect 的唯一标识符，可用于后续手动移除。
pub fn effect<F>(callback: F) -> SignalId
where
    F: FnMut() + Send + Sync + 'static,
{
    let id = SignalId::next();

    let mut cb = Box::new(callback) as Box<dyn FnMut() + Send + Sync>;

    remove_old_dependencies(id);
    start_tracking(id);

    cb();

    let (tracker_id, deps) = stop_tracking();

    if let Some(tid) = tracker_id {
        update_dependencies(tid, &deps);
    }

    EFFECT_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(id, cb);
    });

    id
}

/// 移除指定 Effect
///
/// 从 Effect 注册表中移除指定 ID 的 Effect，
/// 同时清除其在依赖图中的依赖关系。
pub fn remove_effect(id: SignalId) {
    EFFECT_REGISTRY.with(|registry| {
        registry.borrow_mut().remove(&id);
    });
    remove_old_dependencies(id);
}

/// 批量更新信号
///
/// 在 batch 闭包中对多个信号进行更新，所有 Effect 的执行
/// 将延迟到闭包结束后统一触发，避免中间状态触发不必要的副作用。
pub fn batch<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    BATCH_DEPTH.with(|depth| {
        *depth.borrow_mut() += 1;
    });

    let result = f();

    BATCH_DEPTH.with(|depth| {
        *depth.borrow_mut() -= 1;
    });

    run_pending_effects();

    result
}
