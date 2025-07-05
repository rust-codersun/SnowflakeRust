use snowflake_generator::Snowflake;

fn main() {
    // 创建一个雪花ID生成器
    let mut snowflake = Snowflake::new(1, 1);
    
    // 生成一个ID
    let id = snowflake.next_id().unwrap();
    println!("Generated snowflake ID: {}", id);
    
    // 解析这个ID
    let info = Snowflake::parse_id(id);
    println!("\nParsed information:");
    println!("ID: {}", info.id);
    println!("Hex: {}", info.id_as_hex());
    println!("Timestamp: {}", info.timestamp);
    println!("Timestamp formatted: {}", info.timestamp_as_string());
    println!("Datacenter ID: {}", info.datacenter_id);
    println!("Worker ID: {}", info.worker_id);
    println!("Sequence: {}", info.sequence);
    
    println!("\nDetailed format:");
    println!("{}", info.format_details());
}
