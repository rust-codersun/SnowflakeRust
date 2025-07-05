use std::time::Instant;
use snowflake_generator::Snowflake;

fn main() {
    println!("=== 雪花算法性能测试（使用CachedTimeProvider）===");
    
    // 创建使用缓存时间提供者的雪花算法实例
    let mut snowflake = Snowflake::new(1, 1);
    
    // 性能测试参数
    let test_counts = vec![1_000, 10_000, 100_000, 500_000];
    
    for count in test_counts {
        println!("\n--- 生成 {} 个ID的性能测试 ---", count);
        
        let start = Instant::now();
        let mut generated_ids = Vec::with_capacity(count);
        
        for _ in 0..count {
            match snowflake.next_id() {
                Ok(id) => generated_ids.push(id),
                Err(e) => {
                    println!("错误: {:?}", e);
                    break;
                }
            }
        }
        
        let duration = start.elapsed();
        let ids_per_second = count as f64 / duration.as_secs_f64();
        
        println!("生成了 {} 个ID", generated_ids.len());
        println!("耗时: {:?}", duration);
        println!("性能: {:.0} IDs/秒", ids_per_second);
        
        // 验证ID唯一性
        let mut sorted_ids = generated_ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();
        
        if sorted_ids.len() == generated_ids.len() {
            println!("✓ 所有ID都是唯一的");
        } else {
            println!("✗ 检测到重复ID! 唯一ID数量: {}, 总数量: {}", 
                sorted_ids.len(), generated_ids.len());
        }
        
        // 显示前几个和后几个ID作为示例
        if generated_ids.len() >= 10 {
            println!("前5个ID: {:?}", &generated_ids[0..5]);
            println!("后5个ID: {:?}", &generated_ids[generated_ids.len()-5..]);
        }
    }
    
    println!("\n=== 性能测试完成 ===");
}
