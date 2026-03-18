//! 测试数据生成器模块

use rand::{RngExt, distr::Alphanumeric, prelude::IndexedRandom};
use wae_types::{WaeError, WaeErrorKind, WaeResult as TestingResult};

/// Fixture trait - 测试数据接口
pub trait Fixture: Sized {
    /// 生成测试数据
    fn generate() -> TestingResult<Self>;
}

/// Fixture 构建器 trait
pub trait FixtureBuilder<T>: Sized {
    /// 构建测试数据
    fn build(self) -> TestingResult<T>;
}

/// 随机字符串生成器
#[derive(Debug, Clone)]
pub struct RandomString {
    /// 字符串长度
    pub length: usize,
    /// 可选前缀
    pub prefix: Option<String>,
    /// 可选后缀
    pub suffix: Option<String>,
}

impl Default for RandomString {
    fn default() -> Self {
        Self { length: 10, prefix: None, suffix: None }
    }
}

impl RandomString {
    /// 创建新的随机字符串生成器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置字符串长度
    pub fn length(mut self, len: usize) -> Self {
        self.length = len;
        self
    }

    /// 设置前缀
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 设置后缀
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// 生成随机字符串
    pub fn generate(&self) -> TestingResult<String> {
        let mut rng = rand::rng();
        let random_part: String = (0..self.length).map(|_| rng.sample(Alphanumeric) as char).collect();

        let mut result = String::new();
        if let Some(ref prefix) = self.prefix {
            result.push_str(prefix);
        }
        result.push_str(&random_part);
        if let Some(ref suffix) = self.suffix {
            result.push_str(suffix);
        }

        Ok(result)
    }
}

impl Fixture for String {
    fn generate() -> TestingResult<Self> {
        RandomString::new().generate()
    }
}

/// 随机数字生成器
#[derive(Debug, Clone)]
pub struct RandomNumber<T> {
    /// 最小值
    pub min: T,
    /// 最大值
    pub max: T,
    _marker: std::marker::PhantomData<T>,
}

impl<T> RandomNumber<T> {
    /// 创建新的随机数字生成器
    pub fn new(min: T, max: T) -> Self {
        Self { min, max, _marker: std::marker::PhantomData }
    }
}

impl RandomNumber<i32> {
    /// 生成随机 i32
    pub fn generate(&self) -> TestingResult<i32> {
        let mut rng = rand::rng();
        Ok(rng.random_range(self.min..=self.max))
    }
}

impl RandomNumber<i64> {
    /// 生成随机 i64
    pub fn generate(&self) -> TestingResult<i64> {
        let mut rng = rand::rng();
        Ok(rng.random_range(self.min..=self.max))
    }
}

impl RandomNumber<u32> {
    /// 生成随机 u32
    pub fn generate(&self) -> TestingResult<u32> {
        let mut rng = rand::rng();
        Ok(rng.random_range(self.min..=self.max))
    }
}

impl RandomNumber<u64> {
    /// 生成随机 u64
    pub fn generate(&self) -> TestingResult<u64> {
        let mut rng = rand::rng();
        Ok(rng.random_range(self.min..=self.max))
    }
}

impl RandomNumber<f64> {
    /// 生成随机 f64
    pub fn generate(&self) -> TestingResult<f64> {
        let mut rng = rand::rng();
        Ok(rng.random_range(self.min..=self.max))
    }
}

impl Fixture for i32 {
    fn generate() -> TestingResult<Self> {
        let generator: RandomNumber<i32> = RandomNumber::new(0, 1000000);
        generator.generate()
    }
}

impl Fixture for u64 {
    fn generate() -> TestingResult<Self> {
        RandomNumber::new(0u64, 1000000u64).generate()
    }
}

/// 随机邮箱生成器
#[derive(Debug, Clone)]
pub struct RandomEmail {
    /// 邮箱域名
    pub domain: String,
}

impl Default for RandomEmail {
    fn default() -> Self {
        Self { domain: "test.example.com".to_string() }
    }
}

impl RandomEmail {
    /// 创建新的随机邮箱生成器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置域名
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = domain.into();
        self
    }

    /// 生成随机邮箱
    pub fn generate(&self) -> TestingResult<String> {
        let username = RandomString::new().length(8).generate()?;
        Ok(format!("{}@{}", username, self.domain))
    }
}

/// 随机 UUID 生成器
#[derive(Debug, Clone, Default)]
pub struct RandomUuid {
    /// 是否使用 v4 版本
    pub version4: bool,
}

impl RandomUuid {
    /// 创建新的随机 UUID 生成器
    pub fn new() -> Self {
        Self { version4: true }
    }

    /// 使用 v7 版本
    pub fn version7(mut self) -> Self {
        self.version4 = false;
        self
    }

    /// 生成随机 UUID
    pub fn generate(&self) -> TestingResult<uuid::Uuid> {
        if self.version4 { Ok(uuid::Uuid::new_v4()) } else { Ok(uuid::Uuid::now_v7()) }
    }

    /// 生成 UUID 字符串
    pub fn generate_string(&self) -> TestingResult<String> {
        Ok(self.generate()?.to_string())
    }
}

impl Fixture for uuid::Uuid {
    fn generate() -> TestingResult<Self> {
        RandomUuid::new().generate()
    }
}

/// 随机布尔值生成器
#[derive(Debug, Clone, Default)]
pub struct RandomBool {
    /// 为 true 的概率 (0.0 - 1.0)
    pub true_probability: f64,
}

impl RandomBool {
    /// 创建新的随机布尔值生成器
    pub fn new() -> Self {
        Self { true_probability: 0.5 }
    }

    /// 设置 true 的概率
    pub fn probability(mut self, prob: f64) -> Self {
        self.true_probability = prob.clamp(0.0, 1.0);
        self
    }

    /// 生成随机布尔值
    pub fn generate(&self) -> TestingResult<bool> {
        let mut rng = rand::rng();
        Ok(rng.random_bool(self.true_probability))
    }
}

impl Fixture for bool {
    fn generate() -> TestingResult<Self> {
        RandomBool::new().generate()
    }
}

/// 随机选择器 - 从列表中随机选择
#[derive(Debug, Clone)]
pub struct RandomChoice<T> {
    /// 可选项列表
    pub items: Vec<T>,
}

impl<T: Clone> RandomChoice<T> {
    /// 创建新的随机选择器
    pub fn new(items: Vec<T>) -> Self {
        Self { items }
    }

    /// 生成随机选择
    pub fn generate(&self) -> TestingResult<T> {
        let mut rng = rand::rng();
        self.items
            .choose(&mut rng)
            .cloned()
            .ok_or_else(|| WaeError::new(WaeErrorKind::FixtureError { reason: "No items to choose from".to_string() }))
    }
}

/// 随机日期时间生成器
#[derive(Debug, Clone)]
pub struct RandomDateTime {
    /// 开始时间戳 (秒)
    pub start_timestamp: i64,
    /// 结束时间戳 (秒)
    pub end_timestamp: i64,
}

impl Default for RandomDateTime {
    fn default() -> Self {
        Self { start_timestamp: 0, end_timestamp: 4102444800 }
    }
}

impl RandomDateTime {
    /// 创建新的随机日期时间生成器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置时间范围
    pub fn range(mut self, start: i64, end: i64) -> Self {
        self.start_timestamp = start;
        self.end_timestamp = end;
        self
    }

    /// 生成随机时间戳
    pub fn generate_timestamp(&self) -> TestingResult<i64> {
        let mut rng = rand::rng();
        Ok(rng.random_range(self.start_timestamp..=self.end_timestamp))
    }
}

/// 批量数据生成器
pub struct FixtureGenerator;

impl FixtureGenerator {
    /// 生成指定数量的随机字符串
    pub fn strings(count: usize, length: usize) -> TestingResult<Vec<String>> {
        (0..count).map(|_| RandomString::new().length(length).generate()).collect()
    }

    /// 生成指定数量的随机邮箱
    pub fn emails(count: usize, domain: &str) -> TestingResult<Vec<String>> {
        (0..count).map(|_| RandomEmail::new().domain(domain).generate()).collect()
    }

    /// 生成指定数量的随机 UUID
    pub fn uuids(count: usize) -> TestingResult<Vec<uuid::Uuid>> {
        (0..count).map(|_| RandomUuid::new().generate()).collect()
    }

    /// 生成指定数量的随机 i32 数字
    pub fn i32_numbers(count: usize, min: i32, max: i32) -> TestingResult<Vec<i32>> {
        let mut rng = rand::rng();
        (0..count).map(|_| Ok(rng.random_range(min..=max))).collect()
    }

    /// 生成指定数量的随机 u64 数字
    pub fn u64_numbers(count: usize, min: u64, max: u64) -> TestingResult<Vec<u64>> {
        let mut rng = rand::rng();
        (0..count).map(|_| Ok(rng.random_range(min..=max))).collect()
    }
}
