use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
struct CachedValue<T> {
    value: T,
    expires_at: Option<Instant>,
}

struct MockCacheService {
    cache: HashMap<String, CachedValue<String>>,
    default_ttl: Duration,
}

impl MockCacheService {
    fn new(default_ttl: Duration) -> Self {
        Self { cache: HashMap::new(), default_ttl }
    }

    fn set(&mut self, key: &str, value: String, ttl: Option<Duration>) {
        let expires_at = ttl
            .map(|ttl| Instant::now() + ttl)
            .or_else(|| if self.default_ttl > Duration::ZERO { Some(Instant::now() + self.default_ttl) } else { None });

        self.cache.insert(key.to_string(), CachedValue { value, expires_at });
    }

    fn get(&self, key: &str) -> Option<String> {
        if let Some(cached) = self.cache.get(key) {
            if let Some(expires_at) = cached.expires_at {
                if Instant::now() > expires_at {
                    return None;
                }
            }
            return Some(cached.value.clone());
        }
        None
    }

    fn delete(&mut self, key: &str) -> bool {
        self.cache.remove(key).is_some()
    }

    fn exists(&self, key: &str) -> bool {
        if let Some(cached) = self.cache.get(key) {
            if let Some(expires_at) = cached.expires_at {
                if Instant::now() > expires_at {
                    return false;
                }
            }
            return true;
        }
        false
    }

    fn get_ttl(&self, key: &str) -> Option<Duration> {
        if let Some(cached) = self.cache.get(key) {
            if let Some(expires_at) = cached.expires_at {
                let now = Instant::now();
                if expires_at > now {
                    return Some(expires_at - now);
                }
            }
        }
        None
    }

    fn clear(&mut self) {
        self.cache.clear();
    }

    fn size(&self) -> usize {
        self.cache.len()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAE Cache 基础示例 ===\n");

    println!("1. 初始化缓存服务:");
    let mut cache = MockCacheService::new(Duration::from_secs(60));
    println!("   缓存服务初始化完成 (默认 TTL: 60 秒)\n");

    println!("2. 设置缓存项 (无 TTL):");
    cache.set("user:1001", "张三".to_string(), None);
    cache.set("user:1002", "李四".to_string(), None);
    println!("   设置缓存: user:1001 = \"张三\"");
    println!("   设置缓存: user:1002 = \"李四\"");
    println!("   当前缓存数量: {}\n", cache.size());

    println!("3. 获取缓存项:");
    if let Some(value) = cache.get("user:1001") {
        println!("   获取 user:1001 = \"{}\"", value);
    }
    if let Some(value) = cache.get("user:1002") {
        println!("   获取 user:1002 = \"{}\"", value);
    }
    println!();

    println!("4. 检查缓存是否存在:");
    println!("   user:1001 是否存在: {}", cache.exists("user:1001"));
    println!("   user:9999 是否存在: {}", cache.exists("user:9999"));
    println!();

    println!("5. 设置缓存项 (带 TTL):");
    cache.set("temp:session", "临时会话数据".to_string(), Some(Duration::from_secs(5)));
    println!("   设置缓存: temp:session = \"临时会话数据\" (TTL: 5 秒)");
    if let Some(ttl) = cache.get_ttl("temp:session") {
        println!("   剩余过期时间: {} 秒", ttl.as_secs());
    }
    println!();

    println!("6. 更新缓存项:");
    cache.set("user:1001", "张三（已更新）".to_string(), None);
    println!("   更新缓存: user:1001 = \"张三（已更新）\"");
    if let Some(value) = cache.get("user:1001") {
        println!("   获取 user:1001 = \"{}\"", value);
    }
    println!();

    println!("7. 删除缓存项:");
    let deleted = cache.delete("user:1002");
    println!("   删除 user:1002: {}", deleted);
    println!("   user:1002 是否存在: {}", cache.exists("user:1002"));
    println!("   当前缓存数量: {}\n", cache.size());

    println!("8. 获取不存在的缓存:");
    if let Some(value) = cache.get("user:9999") {
        println!("   获取 user:9999 = \"{}\"", value);
    }
    else {
        println!("   获取 user:9999: 缓存不存在 (预期行为)");
    }
    println!();

    println!("9. 清空所有缓存:");
    cache.clear();
    println!("   已清空所有缓存");
    println!("   当前缓存数量: {}\n", cache.size());

    println!("=== 示例总结 ===");
    println!("本示例展示了基本的缓存操作:");
    println!("  - 设置缓存 (SET): 存储键值对");
    println!("  - 获取缓存 (GET): 读取键值对");
    println!("  - 删除缓存 (DELETE): 移除键值对");
    println!("  - 检查存在 (EXISTS): 验证键是否存在");
    println!("  - TTL 管理: 设置和查询过期时间");
    println!("  - 清空缓存 (CLEAR): 移除所有键值对");
    println!();
    println!("注意: 本示例使用内存数据结构模拟缓存服务");
    println!("实际使用时需要配置 Redis 或其他缓存后端");
    println!();

    Ok(())
}
