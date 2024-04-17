use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use gg_error::GResult;

/// 批处理管理器
pub struct BatchingManager {
    /// 批处理组
    batches: Vec<Batch>,
    /// 最大批处理大小
    max_batch_size: usize,
    /// 当前批处理索引
    current_batch: usize,
}

/// 批处理
#[derive(Debug, Default)]
pub struct Batch {
    /// 绘制调用次数
    draw_calls: u32,
    /// 三角形数量
    triangles: u32,
    /// 顶点数量
    vertices: u32,
    /// 材质ID
    material_id: u64,
    /// 渲染命令
    render_commands: Vec<RenderCommand>,
}

/// 渲染命令
#[derive(Debug, Clone)]
pub struct RenderCommand {
    /// 顶点数据
    pub vertex_data: Vec<f32>,
    /// 索引数据
    pub index_data: Vec<u32>,
    /// 材质ID
    pub material_id: u64,
    /// 变换矩阵
    pub transform: [f32; 16],
}

impl BatchingManager {
    /// 创建新的批处理管理器
    pub fn new(max_batch_size: usize) -> Self {
        Self { batches: vec![Batch::default()], max_batch_size, current_batch: 0 }
    }

    /// 添加渲染命令
    pub fn add_render_command(&mut self, command: RenderCommand) {
        let index_count = (command.index_data.len() / 3) as u32;
        let vertex_count = (command.vertex_data.len() / 3) as u32;
        let material_id = command.material_id;

        let current_batch = &mut self.batches[self.current_batch];

        if current_batch.material_id == 0 || current_batch.material_id == material_id {
            if current_batch.render_commands.len() < self.max_batch_size {
                current_batch.render_commands.push(command);
                current_batch.draw_calls += 1;
                current_batch.triangles += index_count;
                current_batch.vertices += vertex_count;
                if current_batch.material_id == 0 {
                    current_batch.material_id = material_id;
                }
                return;
            }
        }

        let mut new_batch = Batch::default();
        new_batch.render_commands.push(command);
        new_batch.draw_calls += 1;
        new_batch.triangles += index_count;
        new_batch.vertices += vertex_count;
        new_batch.material_id = material_id;
        self.batches.push(new_batch);
        self.current_batch += 1;
    }

    /// 执行批处理
    pub fn execute_batches(&mut self) -> GResult<()> {
        for batch in &self.batches {
            if !batch.render_commands.is_empty() {}
        }

        self.batches.clear();
        self.batches.push(Batch::default());
        self.current_batch = 0;

        Ok(())
    }

    /// 获取批处理统计信息
    pub fn get_batch_stats(&self) -> (u32, u32) {
        let total_draw_calls = self.batches.iter().map(|b| b.draw_calls).sum::<u32>();
        let total_batches = self.batches.len() as u32;
        (total_draw_calls, total_batches)
    }
}

/// 缓存管理器
pub struct CacheManager<T> {
    /// 缓存存储
    cache: HashMap<String, Arc<Mutex<CacheItem<T>>>>,
    /// 最大缓存大小
    max_cache_size: usize,
    /// 当前缓存大小
    current_cache_size: usize,
    /// 缓存命中次数
    hits: u32,
    /// 缓存未命中次数
    misses: u32,
}

/// 缓存项
#[derive(Debug)]
pub struct CacheItem<T> {
    /// 数据
    data: T,
    /// 大小
    size: usize,
    /// 最后访问时间
    last_accessed: Instant,
    /// 引用计数
    ref_count: usize,
}

impl<T> CacheManager<T> {
    /// 创建新的缓存管理器
    pub fn new(max_cache_size: usize) -> Self {
        Self { cache: HashMap::new(), max_cache_size, current_cache_size: 0, hits: 0, misses: 0 }
    }

    /// 获取缓存项
    pub fn get(&mut self, key: &str) -> Option<Arc<Mutex<CacheItem<T>>>> {
        if let Some(item) = self.cache.get(key) {
            if let Ok(mut item) = item.lock() {
                item.last_accessed = Instant::now();
                item.ref_count += 1;
            }
            self.hits += 1;
            Some(Arc::clone(item))
        }
        else {
            self.misses += 1;
            None
        }
    }

    /// 添加缓存项
    pub fn put(&mut self, key: String, data: T, size: usize) -> GResult<Arc<Mutex<CacheItem<T>>>> {
        if self.current_cache_size + size > self.max_cache_size {
            self.evict();
        }

        let item = Arc::new(Mutex::new(CacheItem { data, size, last_accessed: Instant::now(), ref_count: 1 }));

        self.cache.insert(key, Arc::clone(&item));
        self.current_cache_size += size;

        Ok(item)
    }

    /// 释放缓存项
    pub fn release(&mut self, key: &str) {
        let should_remove = if let Some(item) = self.cache.get(key) {
            if let Ok(mut item) = item.lock() {
                item.ref_count -= 1;
                item.ref_count <= 0
            }
            else {
                false
            }
        }
        else {
            false
        };

        if should_remove {
            if let Some(item) = self.cache.remove(key) {
                if let Ok(locked) = item.lock() {
                    self.current_cache_size -= locked.size;
                }
            }
        }
    }

    /// 清理缓存
    fn evict(&mut self) {
        let mut items: Vec<(String, Instant)> = self
            .cache
            .iter()
            .filter_map(|(key, item)| {
                if let Ok(item) = item.lock() {
                    if item.ref_count <= 0 { Some((key.clone(), item.last_accessed)) } else { None }
                }
                else {
                    None
                }
            })
            .collect();

        items.sort_by(|a, b| a.1.cmp(&b.1));

        for (key, _) in items {
            if let Some(item) = self.cache.remove(&key) {
                if let Ok(locked) = item.lock() {
                    self.current_cache_size -= locked.size;

                    if self.current_cache_size <= self.max_cache_size * 3 / 4 {
                        break;
                    }
                }
            }
        }
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> (u32, u32, usize) {
        (self.hits, self.misses, self.current_cache_size)
    }

    /// 清除缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_cache_size = 0;
        self.hits = 0;
        self.misses = 0;
    }
}

/// 多线程管理器
pub struct MultiThreadManager {
    /// 线程池大小
    pool_size: usize,
    /// 任务队列
    task_queue: Arc<Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>>,
    /// 线程句柄
    threads: Vec<thread::JoinHandle<()>>,
    /// 停止标志
    stop: Arc<Mutex<bool>>,
}

impl MultiThreadManager {
    /// 创建新的多线程管理器
    pub fn new(pool_size: usize) -> Self {
        let task_queue: Arc<Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>> = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(Mutex::new(false));
        let mut threads = Vec::new();

        for _ in 0..pool_size {
            let task_queue_clone = Arc::clone(&task_queue);
            let stop_clone = Arc::clone(&stop);

            let thread = thread::spawn(move || {
                while !*stop_clone.lock().unwrap() {
                    let task = {
                        let mut queue = task_queue_clone.lock().unwrap();
                        queue.pop()
                    };

                    if let Some(task) = task {
                        task();
                    }
                    else {
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            });

            threads.push(thread);
        }

        Self { pool_size, task_queue, threads, stop }
    }

    /// 添加任务
    pub fn add_task<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_queue.lock().unwrap().push(Box::new(task));
    }

    /// 等待所有任务完成
    pub fn wait_completion(&self) {
        while !self.task_queue.lock().unwrap().is_empty() {
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    /// 停止线程池
    pub fn stop(&mut self) {
        *self.stop.lock().unwrap() = true;

        for thread in self.threads.drain(..) {
            thread.join().unwrap();
        }
    }
}

impl Drop for MultiThreadManager {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 内存池
pub struct MemoryPool<T> {
    /// 空闲对象池
    free_objects: Vec<T>,
    /// 最大池大小
    max_pool_size: usize,
    /// 创建对象的闭包
    create_fn: Box<dyn Fn() -> T>,
}

impl<T> MemoryPool<T> {
    /// 创建新的内存池
    pub fn new(max_pool_size: usize, create_fn: impl Fn() -> T + 'static) -> Self {
        Self { free_objects: Vec::new(), max_pool_size, create_fn: Box::new(create_fn) }
    }

    /// 获取对象
    pub fn acquire(&mut self) -> T {
        if let Some(obj) = self.free_objects.pop() { obj } else { (self.create_fn)() }
    }

    /// 释放对象
    pub fn release(&mut self, obj: T) {
        if self.free_objects.len() < self.max_pool_size {
            self.free_objects.push(obj);
        }
    }

    /// 清除内存池
    pub fn clear(&mut self) {
        self.free_objects.clear();
    }

    /// 获取内存池大小
    pub fn size(&self) -> usize {
        self.free_objects.len()
    }
}
