/// 雪花算法核心常量和共享逻辑
/// 
/// 这个模块包含了雪花算法的所有常量定义和一些共享的辅助函数。

pub const EPOCH: u64 = 1609459200000; // 2021-01-01 00:00:00 UTC
pub const WORKER_ID_BITS: u64 = 5;
pub const DATACENTER_ID_BITS: u64 = 5;
pub const SEQUENCE_BITS: u64 = 12;

pub const MAX_WORKER_ID: u64 = (1 << WORKER_ID_BITS) - 1;
pub const MAX_DATACENTER_ID: u64 = (1 << DATACENTER_ID_BITS) - 1;

pub const WORKER_ID_SHIFT: u64 = SEQUENCE_BITS;
pub const DATACENTER_ID_SHIFT: u64 = SEQUENCE_BITS + WORKER_ID_BITS;
pub const TIMESTAMP_SHIFT: u64 = SEQUENCE_BITS + WORKER_ID_BITS + DATACENTER_ID_BITS;
pub const SEQUENCE_MASK: u64 = (1 << SEQUENCE_BITS) - 1;

/// 从雪花ID中提取时间戳
pub fn extract_timestamp(id: u64) -> u64 {
    (id >> TIMESTAMP_SHIFT) + EPOCH
}

/// 从雪花ID中提取worker_id
pub fn extract_worker_id(id: u64) -> u64 {
    (id >> WORKER_ID_SHIFT) & ((1 << WORKER_ID_BITS) - 1)
}

/// 从雪花ID中提取datacenter_id
pub fn extract_datacenter_id(id: u64) -> u64 {
    (id >> DATACENTER_ID_SHIFT) & ((1 << DATACENTER_ID_BITS) - 1)
}

/// 从雪花ID中提取序列号
pub fn extract_sequence(id: u64) -> u64 {
    id & SEQUENCE_MASK
}

/// 构建雪花ID
pub fn build_snowflake_id(timestamp: u64, datacenter_id: u64, worker_id: u64, sequence: u64) -> u64 {
    ((timestamp - EPOCH) << TIMESTAMP_SHIFT)
        | (datacenter_id << DATACENTER_ID_SHIFT)
        | (worker_id << WORKER_ID_SHIFT)
        | sequence
}

/// 验证worker_id和datacenter_id的有效性
pub fn validate_ids(worker_id: u64, datacenter_id: u64) -> Result<(), String> {
    if worker_id > MAX_WORKER_ID {
        return Err(format!("worker_id {} exceeds maximum {}", worker_id, MAX_WORKER_ID));
    }
    if datacenter_id > MAX_DATACENTER_ID {
        return Err(format!("datacenter_id {} exceeds maximum {}", datacenter_id, MAX_DATACENTER_ID));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_extraction() {
        let timestamp = 1640995200000; // 2022-01-01 00:00:00 UTC
        let datacenter_id = 3;
        let worker_id = 5;
        let sequence = 100;

        let id = build_snowflake_id(timestamp, datacenter_id, worker_id, sequence);
        
        assert_eq!(extract_timestamp(id), timestamp);
        assert_eq!(extract_datacenter_id(id), datacenter_id);
        assert_eq!(extract_worker_id(id), worker_id);
        assert_eq!(extract_sequence(id), sequence);
    }

    #[test]
    fn test_validation() {
        assert!(validate_ids(31, 31).is_ok());
        assert!(validate_ids(32, 31).is_err());
        assert!(validate_ids(31, 32).is_err());
    }
}
