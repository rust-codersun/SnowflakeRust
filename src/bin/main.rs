use snowflake_generator::{Snowflake, WorkerError};

fn main() -> Result<(), WorkerError> {
    println!("=== Snowflake ID Generator with Worker Management ===");
    
    // 使用配置文件创建 Snowflake 实例
    let config_file = "config/worker.conf";
    let default_datacenter_id = 1;
    
    let mut sf = match Snowflake::new_with_config(config_file, default_datacenter_id) {
        Ok(sf) => {
            println!("✓ Snowflake initialized successfully");
            sf
        },
        Err(e) => {
            eprintln!("✗ Failed to initialize Snowflake: {}", e);
            return Err(e);
        }
    };
    
    println!("Worker ID: {}, Datacenter ID: {}", sf.get_worker_id(), sf.get_datacenter_id());
    println!("\nGenerating 10 unique IDs:");
    
    for i in 1..=10 {
        match sf.next_id() {
            Ok(id) => {
                println!("ID {}: {}", i, id);
            },
            Err(e) => {
                eprintln!("✗ Error generating ID {}: {}", i, e);
                return Err(e);
            }
        }
        
        // 添加小延迟以展示不同的时间戳
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    
    println!("\n=== Test completed successfully ===");
    Ok(())
}
