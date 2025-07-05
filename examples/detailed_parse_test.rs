use snowflake_generator::Snowflake;

fn main() {
    println!("=== 雪花ID解析功能测试 ===\n");
    
    // 创建一个雪花ID生成器
    let mut snowflake = Snowflake::new(5, 3);
    
    // 生成多个ID进行测试
    println!("生成测试ID:");
    for i in 0..5 {
        let id = snowflake.next_id().unwrap();
        println!("ID {}: {}", i + 1, id);
        
        // 解析每个ID
        let info = Snowflake::parse_id(id);
        println!("  解析结果:");
        println!("    时间戳: {} ({})", info.timestamp, info.timestamp_as_string());
        println!("    数据中心ID: {}", info.datacenter_id);
        println!("    工作ID: {}", info.worker_id);
        println!("    序列号: {}", info.sequence);
        println!("    十六进制: {}", info.id_as_hex());
        println!("    二进制: {}...{}", &info.id_as_binary()[..32], &info.id_as_binary()[32..]);
        println!();
    }
    
    // 测试一些特殊的ID
    println!("=== 测试特殊ID ===\n");
    
    // 测试最小ID（只有时间戳）
    let min_id = 1 << 22; // 最小的雪花ID
    println!("最小ID: {}", min_id);
    let info = Snowflake::parse_id(min_id);
    println!("  解析结果: 时间戳={}, 数据中心ID={}, 工作ID={}, 序列号={}", 
             info.timestamp, info.datacenter_id, info.worker_id, info.sequence);
    
    // 测试最大序列号
    let max_seq_id = (1 << 22) | 0xFFF; // 最大序列号
    println!("\n最大序列号ID: {}", max_seq_id);
    let info = Snowflake::parse_id(max_seq_id);
    println!("  解析结果: 时间戳={}, 数据中心ID={}, 工作ID={}, 序列号={}", 
             info.timestamp, info.datacenter_id, info.worker_id, info.sequence);
    
    // 测试最大worker和datacenter ID
    let max_worker_id = (1 << 22) | (31 << 17) | (31 << 12); // 最大worker和datacenter ID
    println!("\n最大worker/datacenter ID: {}", max_worker_id);
    let info = Snowflake::parse_id(max_worker_id);
    println!("  解析结果: 时间戳={}, 数据中心ID={}, 工作ID={}, 序列号={}", 
             info.timestamp, info.datacenter_id, info.worker_id, info.sequence);
    
    println!("\n=== 详细格式化显示 ===\n");
    let test_id = snowflake.next_id().unwrap();
    let info = Snowflake::parse_id(test_id);
    println!("{}", info.format_details());
}
