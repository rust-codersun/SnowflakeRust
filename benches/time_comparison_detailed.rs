use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

/// 雪花算法常量
const EPOCH: u64 = 1609459200000; // 2021-01-01 00:00:00 UTC
const WORKER_ID_BITS: u64 = 5;
const DATACENTER_ID_BITS: u64 = 5;
const SEQUENCE_BITS: u64 = 12;

const WORKER_ID_SHIFT: u64 = SEQUENCE_BITS;
const DATACENTER_ID_SHIFT: u64 = SEQUENCE_BITS + WORKER_ID_BITS;
const TIMESTAMP_SHIFT: u64 = SEQUENCE_BITS + WORKER_ID_BITS + DATACENTER_ID_BITS;
const SEQUENCE_MASK: u64 = (1 << SEQUENCE_BITS) - 1;

/// 系统时间版本的雪花算法
pub struct SystemTimeSnowflake {
    worker_id: u64,
    datacenter_id: u64,
    sequence: u64,
    last_timestamp: u64,
    lock: Mutex<()>,
}

impl SystemTimeSnowflake {
    pub fn new(worker_id: u64, datacenter_id: u64) -> Self {
        SystemTimeSnowflake {
            worker_id,
            datacenter_id,
            sequence: 0,
            last_timestamp: 0,
            lock: Mutex::new(()),
        }
    }

    fn current_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn til_next_millis(last_timestamp: u64) -> u64 {
        let mut ts = Self::current_millis();
        while ts <= last_timestamp {
            ts = Self::current_millis();
        }
        ts
    }

    pub fn next_id(&mut self) -> u64 {
        let _guard = self.lock.lock().unwrap();
        let mut timestamp = Self::current_millis();

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & SEQUENCE_MASK;
            if self.sequence == 0 {
                timestamp = Self::til_next_millis(self.last_timestamp);
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        ((timestamp - EPOCH) << TIMESTAMP_SHIFT)
            | (self.datacenter_id << DATACENTER_ID_SHIFT)
            | (self.worker_id << WORKER_ID_SHIFT)
            | self.sequence
    }
}

/// 相对时间版本的雪花算法
pub struct RelativeTimeSnowflake {
    worker_id: u64,
    datacenter_id: u64,
    sequence: u64,
    last_timestamp: u64,
    lock: Mutex<()>,
    start_time: Instant,
    base_timestamp: u64,
}

impl RelativeTimeSnowflake {
    pub fn new(worker_id: u64, datacenter_id: u64) -> Self {
        let start_time = Instant::now();
        let base_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        RelativeTimeSnowflake {
            worker_id,
            datacenter_id,
            sequence: 0,
            last_timestamp: 0,
            lock: Mutex::new(()),
            start_time,
            base_timestamp,
        }
    }

    fn current_millis(&self) -> u64 {
        self.base_timestamp + self.start_time.elapsed().as_millis() as u64
    }

    fn til_next_millis(&self, last_timestamp: u64) -> u64 {
        let mut ts = self.current_millis();
        while ts <= last_timestamp {
            ts = self.current_millis();
        }
        ts
    }

    pub fn next_id(&mut self) -> u64 {
        let _guard = self.lock.lock().unwrap();
        let mut timestamp = self.current_millis();

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & SEQUENCE_MASK;
            if self.sequence == 0 {
                timestamp = self.til_next_millis(self.last_timestamp);
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        ((timestamp - EPOCH) << TIMESTAMP_SHIFT)
            | (self.datacenter_id << DATACENTER_ID_SHIFT)
            | (self.worker_id << WORKER_ID_SHIFT)
            | self.sequence
    }
}

/// 缓存时间版本的雪花算法（使用项目中的CachedTimeProvider）
pub struct CachedTimeSnowflake {
    worker_id: u64,
    datacenter_id: u64,
    sequence: u64,
    last_timestamp: u64,
    lock: Mutex<()>,
    time_provider: Arc<CachedTimeProvider>,
}

/// 简化的缓存时间提供者
pub struct CachedTimeProvider {
    cached_millis: AtomicU64,
    running: AtomicU64,
}

impl CachedTimeProvider {
    pub fn new(update_interval_ms: u64) -> Arc<Self> {
        let provider = Arc::new(CachedTimeProvider {
            cached_millis: AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            ),
            running: AtomicU64::new(1),
        });

        let provider_clone = provider.clone();
        thread::spawn(move || {
            while provider_clone.running.load(Ordering::Relaxed) == 1 {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                provider_clone.cached_millis.store(current_time, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(update_interval_ms));
            }
        });

        provider
    }

    pub fn current_millis(&self) -> u64 {
        self.cached_millis.load(Ordering::Relaxed)
    }
}

impl Drop for CachedTimeProvider {
    fn drop(&mut self) {
        self.running.store(0, Ordering::Relaxed);
    }
}

impl CachedTimeSnowflake {
    pub fn new(worker_id: u64, datacenter_id: u64) -> Self {
        let time_provider = CachedTimeProvider::new(1);

        CachedTimeSnowflake {
            worker_id,
            datacenter_id,
            sequence: 0,
            last_timestamp: 0,
            lock: Mutex::new(()),
            time_provider,
        }
    }

    fn current_millis(&self) -> u64 {
        self.time_provider.current_millis()
    }

    fn til_next_millis(&self, last_timestamp: u64) -> u64 {
        let mut ts = self.current_millis();
        while ts <= last_timestamp {
            ts = self.current_millis();
        }
        ts
    }

    pub fn next_id(&mut self) -> u64 {
        let _guard = self.lock.lock().unwrap();
        let mut timestamp = self.current_millis();

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & SEQUENCE_MASK;
            if self.sequence == 0 {
                timestamp = self.til_next_millis(self.last_timestamp);
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        ((timestamp - EPOCH) << TIMESTAMP_SHIFT)
            | (self.datacenter_id << DATACENTER_ID_SHIFT)
            | (self.worker_id << WORKER_ID_SHIFT)
            | self.sequence
    }
}

fn main() {
    println!("=== 雪花算法时间获取方案性能对比测试 ===");
    
    let test_counts = vec![1_000, 5_000, 10_000];
    
    for &count in &test_counts {
        println!("\n{}", "=".repeat(70));
        println!("测试规模: {} 个ID", count);
        
        // 预热阶段 - 生成少量ID以确保公平测试
        println!("\n--- 预热阶段 ---");
        let mut warmup_system = SystemTimeSnowflake::new(1, 1);
        let mut warmup_relative = RelativeTimeSnowflake::new(1, 1);
        let mut warmup_cached = CachedTimeSnowflake::new(1, 1);
        
        for _ in 0..10 {
            let _ = warmup_system.next_id();
            let _ = warmup_relative.next_id();
            let _ = warmup_cached.next_id();
        }
        println!("预热完成");
        
        // 1. 系统时间版本
        println!("\n--- 系统时间版本 ---");
        let mut system_snowflake = SystemTimeSnowflake::new(1, 1);
        let start = Instant::now();
        
        for _ in 0..count {
            let _ = system_snowflake.next_id(); // 直接丢弃，不存储
        }
        
        let system_duration = start.elapsed();
        let system_ids_per_sec = count as f64 / system_duration.as_secs_f64();
        
        println!("耗时: {:?}", system_duration);
        println!("性能: {:.0} IDs/秒", system_ids_per_sec);
        
        // 2. 相对时间版本
        println!("\n--- 相对时间版本 ---");
        let mut relative_snowflake = RelativeTimeSnowflake::new(1, 1);
        let start = Instant::now();
        
        for _ in 0..count {
            let _ = relative_snowflake.next_id(); // 直接丢弃，不存储
        }
        
        let relative_duration = start.elapsed();
        let relative_ids_per_sec = count as f64 / relative_duration.as_secs_f64();
        
        println!("耗时: {:?}", relative_duration);
        println!("性能: {:.0} IDs/秒", relative_ids_per_sec);
        
        // 3. 缓存时间版本
        println!("\n--- 缓存时间版本 ---");
        let mut cached_snowflake = CachedTimeSnowflake::new(1, 1);
        // 让缓存时间提供者预热
        thread::sleep(Duration::from_millis(10));
        
        let start = Instant::now();
        
        for _ in 0..count {
            let _ = cached_snowflake.next_id(); // 直接丢弃，不存储
        }
        
        let cached_duration = start.elapsed();
        let cached_ids_per_sec = count as f64 / cached_duration.as_secs_f64();
        
        println!("耗时: {:?}", cached_duration);
        println!("性能: {:.0} IDs/秒", cached_ids_per_sec);
        
        // 性能对比
        println!("\n--- 性能对比 ---");
        let system_vs_relative = system_ids_per_sec / relative_ids_per_sec;
        let system_vs_cached = system_ids_per_sec / cached_ids_per_sec;
        let relative_vs_cached = relative_ids_per_sec / cached_ids_per_sec;
        
        println!("系统时间 vs 相对时间: {:.2}x", system_vs_relative);
        println!("系统时间 vs 缓存时间: {:.2}x", system_vs_cached);
        println!("相对时间 vs 缓存时间: {:.2}x", relative_vs_cached);
        
        // 最佳方案
        let max_perf = system_ids_per_sec.max(relative_ids_per_sec).max(cached_ids_per_sec);
        if system_ids_per_sec == max_perf {
            println!("🏆 最佳方案: 系统时间");
        } else if relative_ids_per_sec == max_perf {
            println!("🏆 最佳方案: 相对时间");
        } else {
            println!("🏆 最佳方案: 缓存时间");
        }
        
        // 简化的ID唯一性验证（生成少量ID进行验证）
        println!("\n--- ID唯一性验证 (测试100个ID) ---");
        
        let mut system_test = SystemTimeSnowflake::new(1, 1);
        let mut relative_test = RelativeTimeSnowflake::new(1, 1);
        let mut cached_test = CachedTimeSnowflake::new(1, 1);
        
        let test_size = 100;
        let mut system_test_ids = Vec::with_capacity(test_size);
        let mut relative_test_ids = Vec::with_capacity(test_size);
        let mut cached_test_ids = Vec::with_capacity(test_size);
        
        for _ in 0..test_size {
            system_test_ids.push(system_test.next_id());
            relative_test_ids.push(relative_test.next_id());
            cached_test_ids.push(cached_test.next_id());
        }
        
        let all_unique = |ids: &Vec<u64>| {
            let mut sorted = ids.clone();
            sorted.sort();
            sorted.dedup();
            sorted.len() == ids.len()
        };
        
        println!("系统时间版本: {} ({}个ID)", if all_unique(&system_test_ids) { "✓" } else { "✗" }, system_test_ids.len());
        println!("相对时间版本: {} ({}个ID)", if all_unique(&relative_test_ids) { "✓" } else { "✗" }, relative_test_ids.len());
        println!("缓存时间版本: {} ({}个ID)", if all_unique(&cached_test_ids) { "✓" } else { "✗" }, cached_test_ids.len());
    }
    
    println!("\n{}", "=".repeat(70));
    println!("🎯 结论:");
    println!("1. 缓存时间方案通常在大量ID生成时性能最优");
    println!("2. 系统时间方案稳定可靠，适合中等负载场景");
    println!("3. 相对时间方案避免系统调用，在某些情况下表现良好");
    println!("4. 实际应用中建议根据负载特点选择合适的时间获取方案");
    println!("5. 本测试已优化资源使用，减少了内存分配和测试规模");
}
