use std::sync::{Mutex, Arc};

use crate::worker_manager::{WorkerManager, WorkerError};
use crate::time_provider::{CachedTimeProvider, TimeProvider};
use crate::snowflake_core::*;

/// 生产级雪花算法ID生成器
/// 
/// 这是主要的雪花算法实现，集成了：
/// - Worker ID 持久化管理
/// - 缓存时间提供者（性能优化）
/// - 时钟回拨检测
/// - 高并发支持
pub struct Snowflake {
    worker_id: u64,
    datacenter_id: u64,
    sequence: u64,
    last_timestamp: u64,
    lock: Mutex<()>,
    worker_manager: Option<WorkerManager>,
    time_provider: Arc<CachedTimeProvider>,
}

impl Snowflake {
    /// 创建新的雪花算法生成器
    /// 
    /// # 参数
    /// - `worker_id`: Worker ID (0-31)
    /// - `datacenter_id`: Datacenter ID (0-31)
    pub fn new(worker_id: u64, datacenter_id: u64) -> Self {
        validate_ids(worker_id, datacenter_id).expect("Invalid worker_id or datacenter_id");
        
        // 创建缓存时间提供者（每1毫秒更新一次）
        let time_provider = CachedTimeProvider::new(1);
        
        Snowflake {
            worker_id,
            datacenter_id,
            sequence: 0,
            last_timestamp: 0,
            lock: Mutex::new(()),
            worker_manager: None,
            time_provider,
        }
    }

    /// 使用配置文件创建雪花算法生成器
    /// 
    /// # 参数
    /// - `config_file`: 配置文件路径
    /// - `default_datacenter_id`: 默认数据中心ID
    pub fn new_with_config(config_file: &str, default_datacenter_id: u64) -> Result<Self, WorkerError> {
        let worker_manager = WorkerManager::new(config_file, default_datacenter_id)?;
        let worker_info = worker_manager.get_worker_info();
        
        // 创建缓存时间提供者（每1毫秒更新一次）
        let time_provider = CachedTimeProvider::new(1);
        
        let mut snowflake = Snowflake {
            worker_id: worker_info.worker_id,
            datacenter_id: worker_info.datacenter_id,
            sequence: 0,
            last_timestamp: worker_info.last_timestamp,
            lock: Mutex::new(()),
            worker_manager: Some(worker_manager),
            time_provider,
        };

        // 更新 worker manager 的时间戳
        if let Some(ref mut manager) = snowflake.worker_manager {
            manager.update_and_save()?;
        }

        Ok(snowflake)
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

    /// 生成下一个雪花ID
    /// 
    /// # 返回值
    /// - `Ok(u64)`: 生成的雪花ID
    /// - `Err(WorkerError)`: 时钟回拨或其他错误
    pub fn next_id(&mut self) -> Result<u64, WorkerError> {
        let _guard = self.lock.lock().unwrap();
        let mut timestamp = self.current_millis();
        
        // 检查时钟回拨
        if timestamp < self.last_timestamp {
            return Err(WorkerError::ClockBackwardsError(
                format!("Clock moved backwards. Last: {}, Current: {}", 
                    self.last_timestamp, timestamp)
            ));
        }
        
        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & SEQUENCE_MASK;
            if self.sequence == 0 {
                timestamp = self.til_next_millis(self.last_timestamp);
            }
        } else {
            self.sequence = 0;
        }
        
        self.last_timestamp = timestamp;
        
        // 更新 worker manager 的时间戳（降低频率，避免频繁IO）
        if let Some(ref mut manager) = self.worker_manager {
            // 每1000个ID更新一次，减少IO操作
            if self.sequence % 1000 == 0 {
                manager.update_and_save()?;
            }
        }
        
        Ok(build_snowflake_id(timestamp, self.datacenter_id, self.worker_id, self.sequence))
    }
    
    pub fn get_worker_id(&self) -> u64 {
        self.worker_id
    }
    
    pub fn get_datacenter_id(&self) -> u64 {
        self.datacenter_id
    }
    
    pub fn get_last_timestamp(&self) -> u64 {
        self.last_timestamp
    }

    /// 解析雪花ID，返回其各个组成部分的信息
    /// 
    /// # 参数
    /// - `id`: 要解析的雪花ID
    /// 
    /// # 返回值
    /// 返回包含时间戳、数据中心ID、工作ID和序列号的元组
    pub fn parse_id(id: u64) -> SnowflakeInfo {
        SnowflakeInfo {
            id,
            timestamp: extract_timestamp(id),
            datacenter_id: extract_datacenter_id(id),
            worker_id: extract_worker_id(id),
            sequence: extract_sequence(id),
        }
    }
}

/// 雪花ID解析信息结构体
#[derive(Debug, Clone)]
pub struct SnowflakeInfo {
    pub id: u64,
    pub timestamp: u64,
    pub datacenter_id: u64,
    pub worker_id: u64,
    pub sequence: u64,
}

impl SnowflakeInfo {
    /// 获取可读的时间戳字符串
    pub fn timestamp_as_string(&self) -> String {
        use std::time::{SystemTime, Duration};
        
        let timestamp_secs = self.timestamp / 1000;
        let timestamp_millis = self.timestamp % 1000;
        
        match SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(timestamp_secs)) {
            Some(time) => {
                format!("{:?}.{:03}", time, timestamp_millis)
            }
            None => format!("Invalid timestamp: {}", self.timestamp)
        }
    }
    
    /// 获取ID的十六进制表示
    pub fn id_as_hex(&self) -> String {
        format!("0x{:016x}", self.id)
    }
    
    /// 获取ID的二进制表示（带分隔符）
    pub fn id_as_binary(&self) -> String {
        format!("{:064b}", self.id)
    }
    
    /// 获取详细的格式化信息
    pub fn format_details(&self) -> String {
        format!(
            "Snowflake ID: {}\n\
             Hex: {}\n\
             Binary: {}\n\
             Timestamp: {} ({})\n\
             Datacenter ID: {}\n\
             Worker ID: {}\n\
             Sequence: {}",
            self.id,
            self.id_as_hex(),
            self.id_as_binary(),
            self.timestamp,
            self.timestamp_as_string(),
            self.datacenter_id,
            self.worker_id,
            self.sequence
        )
    }
}

// 示例用法和测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snowflake_creation() {
        let sf = Snowflake::new(1, 1);
        assert_eq!(sf.get_worker_id(), 1);
        assert_eq!(sf.get_datacenter_id(), 1);
    }
    
    #[test]
    fn test_id_generation() {
        let mut sf = Snowflake::new(1, 1);
        let id1 = sf.next_id().unwrap();
        let id2 = sf.next_id().unwrap();
        assert_ne!(id1, id2);
    }
}