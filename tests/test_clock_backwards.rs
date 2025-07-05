use std::fs::OpenOptions;
use std::io::Write;

use snowflake_generator::{Snowflake, WorkerError};

fn main() -> Result<(), WorkerError> {
    println!("=== 时钟回拨测试 ===");
    
    // 1. 创建正常的配置文件
    let config_file = "config/test_worker.conf";
    let mut sf = Snowflake::new_with_config(config_file, 1)?;
    
    println!("✓ 生成第一个 ID");
    let id1 = sf.next_id()?;
    println!("ID: {}", id1);
    
    // 2. 手动修改配置文件，模拟时钟回拨
    println!("\n--- 模拟时钟回拨 ---");
    let future_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64 + 60000; // 未来1分钟
    
    // 修改配置文件中的时间戳
    let fake_content = format!("18\n1\n{}\n{}\n", future_timestamp, future_timestamp);
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(config_file)?;
    file.write_all(fake_content.as_bytes())?;
    drop(file);
    
    println!("已修改配置文件时间戳到未来");
    
    // 3. 尝试重新初始化，应该检测到时钟回拨
    println!("尝试重新初始化...");
    match Snowflake::new_with_config(config_file, 1) {
        Ok(_) => println!("⚠️  未检测到时钟回拨（不应该发生）"),
        Err(WorkerError::ClockBackwardsError(msg)) => {
            println!("✓ 成功检测到时钟回拨: {}", msg);
        },
        Err(e) => println!("❌ 其他错误: {}", e),
    }
    
    // 清理测试文件
    let _ = std::fs::remove_file(config_file);
    
    println!("\n=== 测试完成 ===");
    Ok(())
}
