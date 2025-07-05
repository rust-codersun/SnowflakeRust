use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// 时间提供者 trait
pub trait TimeProvider {
    fn current_millis(&self) -> u64;
}

/// 系统时间提供者：直接获取系统时间
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn current_millis(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

/// 相对时间提供者：基于程序启动时的 Instant
pub struct RelativeTimeProvider {
    start_instant: Instant,
    start_millis: u64,
}

impl RelativeTimeProvider {
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
            start_millis: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

impl TimeProvider for RelativeTimeProvider {
    fn current_millis(&self) -> u64 {
        let elapsed = self.start_instant.elapsed().as_millis() as u64;
        self.start_millis + elapsed
    }
}

/// 缓存时间提供者：定期更新时间戳缓存
pub struct CachedTimeProvider {
    /// 缓存的时间戳
    cached_millis: AtomicU64,
    /// 是否正在运行
    running: AtomicU64, // 使用 AtomicU64 作为布尔值 (0=false, 1=true)
}

impl TimeProvider for CachedTimeProvider {
    fn current_millis(&self) -> u64 {
        self.cached_millis.load(Ordering::Relaxed)
    }
}

impl CachedTimeProvider {
    pub fn new(update_interval_ms: u64) -> Arc<Self> {
        let provider = Arc::new(CachedTimeProvider {
            cached_millis: AtomicU64::new(Self::get_system_millis()),
            running: AtomicU64::new(1),
        });
        
        // 启动后台线程定期更新时间戳
        let provider_clone = provider.clone();
        thread::spawn(move || {
            while provider_clone.running.load(Ordering::Relaxed) == 1 {
                let current_time = Self::get_system_millis();
                provider_clone.cached_millis.store(current_time, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(update_interval_ms));
            }
        });
        
        provider
    }
    
    /// 强制更新时间戳
    pub fn force_update(&self) {
        let current_time = Self::get_system_millis();
        self.cached_millis.store(current_time, Ordering::Relaxed);
    }
    
    /// 停止后台更新线程
    pub fn stop(&self) {
        self.running.store(0, Ordering::Relaxed);
    }
    
    fn get_system_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

impl Drop for CachedTimeProvider {
    fn drop(&mut self) {
        self.stop();
    }
}
