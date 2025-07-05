// Snowflake ID Generator Example
// This example demonstrates how to use the snowflake generator

use snowflake_generator::Snowflake;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Snowflake ID Generator Examples ===\n");

    // 1. Basic usage with default configuration
    println!("1. Basic Snowflake (with CachedTimeProvider):");
    let mut basic_snowflake = Snowflake::new(1, 1);
    for i in 0..5 {
        let id = basic_snowflake.next_id()?;
        println!("  ID {}: {}", i + 1, id);
    }

    // 2. Using with configuration file
    println!("\n2. Snowflake with configuration file:");
    match Snowflake::new_with_config("config/worker.conf", 1) {
        Ok(mut config_snowflake) => {
            for i in 0..3 {
                let id = config_snowflake.next_id()?;
                println!("  Config ID {}: {}", i + 1, id);
            }
        }
        Err(e) => {
            println!("  Config file not found or invalid: {}", e);
            println!("  (This is expected if config/worker.conf doesn't exist)");
        }
    }

    // 3. Multiple workers demonstration
    println!("\n3. Multiple workers demonstration:");
    let mut workers = Vec::new();
    for worker_id in 1..=3 {
        let snowflake = Snowflake::new(worker_id, 1);
        workers.push(snowflake);
    }

    for (i, worker) in workers.iter_mut().enumerate() {
        println!("  Worker {}:", i + 1);
        for j in 0..3 {
            let id = worker.next_id()?;
            println!("    ID {}: {}", j + 1, id);
        }
    }

    // 4. Performance test
    println!("\n4. Performance test:");
    let iterations = 10_000;
    let mut snowflake = Snowflake::new(5, 1);
    
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = snowflake.next_id()?;
    }
    let duration = start.elapsed();

    println!("  {} IDs generation:", iterations);
    println!("    Time: {:?} ({:.0} IDs/sec)", 
        duration, 
        iterations as f64 / duration.as_secs_f64()
    );

    // 5. Show ID structure
    println!("\n5. ID Structure Analysis:");
    let mut basic_snowflake = Snowflake::new(7, 2);
    let id = basic_snowflake.next_id()?;
    
    println!("  Generated ID: {}", id);
    println!("  Worker ID: {}", basic_snowflake.get_worker_id());
    println!("  Datacenter ID: {}", basic_snowflake.get_datacenter_id());
    println!("  Binary representation: {:064b}", id);
    
    // Extract components (demonstration purposes)
    let timestamp_part = id >> 22;
    let datacenter_part = (id >> 17) & 0x1F;
    let worker_part = (id >> 12) & 0x1F;
    let sequence_part = id & 0xFFF;
    
    println!("  Timestamp part: {}", timestamp_part);
    println!("  Datacenter part: {}", datacenter_part);
    println!("  Worker part: {}", worker_part);
    println!("  Sequence part: {}", sequence_part);

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
