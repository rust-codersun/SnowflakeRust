use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;
use snowflake_generator::CachedTimeProvider;
use snowflake_generator::TimeProvider;

fn main() {
    println!("=== 缓存时间提供者测试 ===");
    
    // 创建缓存时间提供者（每1毫秒更新一次）
    let time_provider = CachedTimeProvider::new(1);
    
    // 性能测试
    let iterations = 1_000_000;
    
    // 测试缓存方案性能
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = time_provider.current_millis();
    }
    let cached_duration = start.elapsed();
    
    // 测试原始系统时间方案
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    }
    let system_duration = start.elapsed();
    
    println!("缓存方案 ({} 次调用): {:?}", iterations, cached_duration);
    println!("系统时间方案 ({} 次调用): {:?}", iterations, system_duration);
    println!("性能提升: {:.2}x", 
        system_duration.as_nanos() as f64 / cached_duration.as_nanos() as f64);
    
    // 测试精度
    println!("\n=== 精度测试 ===");
    for i in 0..5 {
        let cached_time = time_provider.current_millis();
        let system_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let diff = if system_time > cached_time { 
            system_time - cached_time 
        } else { 
            cached_time - system_time 
        };
        
        println!("第{}次 - 缓存时间: {}, 系统时间: {}, 差异: {} ms", 
            i + 1, cached_time, system_time, diff);
        
        thread::sleep(Duration::from_millis(50));
    }
    
    time_provider.stop();
    thread::sleep(Duration::from_millis(10)); // 等待后台线程结束
}
