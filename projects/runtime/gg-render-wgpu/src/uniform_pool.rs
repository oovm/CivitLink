//! Uniform 缓冲区池模块
//!
//! 管理 uniform 缓冲区的帧间复用，减少 GPU 内存分配开销。

use wgpu::util::DeviceExt;

/// Uniform 缓冲区池
///
/// 通过帧间复用减少 GPU 内存分配。
/// 每帧结束后将使用过的缓冲区回收到池中，下一帧优先复用。
///
/// # 工作原理
///
/// 1. 帧内通过 [`UniformPool::allocate`] 分配 uniform 缓冲区
/// 2. 优先从可用列表中取用已回收的缓冲区，通过 `queue.write_buffer()` 更新数据
/// 3. 若无可用缓冲区则创建新的
/// 4. 帧结束后通过 [`UniformPool::mark_used`] 标记缓冲区为待回收
/// 5. 下一帧开始时通过 [`UniformPool::recycle_frame`] 将待回收缓冲区移入可用列表
pub struct UniformPool {
    /// 可复用的缓冲区列表
    available: Vec<wgpu::Buffer>,
    /// 待回收的缓冲区列表
    pending_recycle: Vec<wgpu::Buffer>,
    /// 缓冲区的固定字节大小
    buffer_size: usize,
}

impl UniformPool {
    /// 创建新的 uniform 缓冲区池
    ///
    /// # 参数
    ///
    /// - `buffer_size` - 单个缓冲区的固定字节大小，
    ///   应取所有 uniform 类型的最大尺寸以确保复用兼容性
    pub fn new(buffer_size: usize) -> Self {
        Self { available: Vec::new(), pending_recycle: Vec::new(), buffer_size }
    }

    /// 分配一个 uniform 缓冲区并写入数据
    ///
    /// 优先从可用列表中取用缓冲区并通过 `write_buffer` 更新数据，
    /// 若无可用缓冲区则创建新的。
    /// 新创建的缓冲区大小为 [`UniformPool::buffer_size`]，
    /// 数据不足部分以零填充。
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `queue` - wgpu 命令队列
    /// - `data` - 要写入缓冲区的数据，长度不得超过 `buffer_size`
    ///
    /// # 崩溃
    ///
    /// 当 `data.len() > buffer_size` 时触发 panic
    pub fn allocate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) -> wgpu::Buffer {
        assert!(data.len() <= self.buffer_size, "数据大小超过缓冲区大小");
        if let Some(buf) = self.available.pop() {
            queue.write_buffer(&buf, 0, data);
            buf
        }
        else {
            let mut contents = data.to_vec();
            contents.resize(self.buffer_size, 0);
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform_buffer"),
                contents: &contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        }
    }

    /// 标记缓冲区为待回收
    ///
    /// 该缓冲区将在下一帧开始时被移入可用列表。
    ///
    /// # 参数
    ///
    /// - `buffer` - 要回收的缓冲区
    pub fn mark_used(&mut self, buffer: wgpu::Buffer) {
        self.pending_recycle.push(buffer);
    }

    /// 回收待回收的缓冲区
    ///
    /// 将所有待回收的缓冲区移入可用列表，供下一帧复用。
    /// 应在每帧开始时调用。
    pub fn recycle_frame(&mut self) {
        let recycled = std::mem::take(&mut self.pending_recycle);
        self.available.extend(recycled);
    }

    /// 获取池中可用缓冲区数量
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// 清空池中所有缓冲区
    pub fn clear(&mut self) {
        self.available.clear();
        self.pending_recycle.clear();
    }
}

impl Default for UniformPool {
    fn default() -> Self {
        Self::new(0)
    }
}
