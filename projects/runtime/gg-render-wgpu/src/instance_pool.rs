use std::collections::HashMap;

use bytemuck::Pod;

/// 实例缓冲区池
///
/// 缓存 wgpu 实例缓冲区，按容量分组管理，
/// 实现帧间缓冲区复用，减少每帧 GPU 内存分配。
///
/// # 工作原理
///
/// 1. 每帧结束后通过 [`InstanceBufferPool::return_buffer`] 归还缓冲区
/// 2. 下一帧通过 [`InstanceBufferPool::acquire_or_create`] 获取缓冲区
/// 3. 优先复用已有缓冲区，容量不足时创建新缓冲区
/// 4. 连续多帧使用率低于 50% 时自动缩小缓冲区
pub struct InstanceBufferPool {
    /// 按容量分组的可用缓冲区
    available: HashMap<u64, Vec<wgpu::Buffer>>,
    /// 上一帧使用的缓冲区及其使用率
    last_frame_usage: Vec<(wgpu::Buffer, u64, u64)>,
    /// 连续低使用率帧数
    low_usage_frames: u32,
}

impl InstanceBufferPool {
    /// 创建空的实例缓冲区池
    pub fn new() -> Self {
        Self { available: HashMap::new(), last_frame_usage: Vec::new(), low_usage_frames: 0 }
    }

    /// 获取或创建指定容量的实例缓冲区
    ///
    /// 优先从池中获取匹配容量的缓冲区，
    /// 如果没有可用缓冲区则创建新的。
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `capacity` - 所需容量（实例数量）
    /// - `label` - 缓冲区标签，用于调试
    pub fn acquire_or_create<T: Pod>(&mut self, device: &wgpu::Device, capacity: u64, label: &str) -> wgpu::Buffer {
        let size = std::mem::size_of::<T>() as u64 * capacity;

        if let Some(buffers) = self.available.get_mut(&size) {
            if let Some(buffer) = buffers.pop() {
                return buffer;
            }
        }

        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// 归还缓冲区到池中
    ///
    /// 将缓冲区归还到复用池，同时记录使用率用于自动缩放。
    ///
    /// # 参数
    ///
    /// - `buffer` - 要归还的缓冲区
    /// - `capacity` - 缓冲区总容量（实例数量）
    /// - `used` - 实际使用的实例数量
    pub fn return_buffer(&mut self, buffer: wgpu::Buffer, capacity: u64, used: u64) {
        self.last_frame_usage.push((buffer, capacity, used));
    }

    /// 回收上一帧的所有缓冲区
    ///
    /// 根据使用率决定是否保留缓冲区。
    /// 连续 10 帧使用率低于 50% 时，丢弃缓冲区以释放 GPU 内存。
    pub fn recycle_frame(&mut self) {
        let all_low = !self.last_frame_usage.is_empty() && self.last_frame_usage.iter().all(|(_, cap, used)| *used * 2 < *cap);

        if all_low {
            self.low_usage_frames += 1;
        }
        else {
            self.low_usage_frames = 0;
        }

        for (buffer, capacity, _used) in self.last_frame_usage.drain(..) {
            if self.low_usage_frames >= 10 {
                continue;
            }
            let size = capacity;
            self.available.entry(size).or_default().push(buffer);
        }
    }

    /// 清空池中所有缓冲区
    pub fn clear(&mut self) {
        self.available.clear();
        self.last_frame_usage.clear();
        self.low_usage_frames = 0;
    }
}

impl Default for InstanceBufferPool {
    fn default() -> Self {
        Self::new()
    }
}
