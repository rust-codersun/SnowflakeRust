use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum WorkerError {
    IoError(std::io::Error),
    ParseError(String),
    ClockBackwardsError(String),
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WorkerError::IoError(err) => write!(f, "IO error: {}", err),
            WorkerError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            WorkerError::ClockBackwardsError(msg) => write!(f, "Clock backwards error: {}", msg),
        }
    }
}

impl Error for WorkerError {}

impl From<std::io::Error> for WorkerError {
    fn from(error: std::io::Error) -> Self {
        WorkerError::IoError(error)
    }
}

#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub worker_id: u64,
    pub datacenter_id: u64,
    pub last_timestamp: u64,
    pub creation_time: u64,
}

impl WorkerInfo {
    pub fn new(worker_id: u64, datacenter_id: u64) -> Self {
        let current_time = current_millis();
        WorkerInfo {
            worker_id,
            datacenter_id,
            last_timestamp: current_time,
            creation_time: current_time,
        }
    }

    pub fn from_file_content(content: &str) -> Result<Self, WorkerError> {
        let lines: Vec<&str> = content.trim().split('\n').collect();
        if lines.len() < 4 {
            return Err(WorkerError::ParseError(
                "Invalid file format: missing required fields".to_string()
            ));
        }

        let worker_id = lines[0].trim().parse::<u64>()
            .map_err(|_| WorkerError::ParseError("Invalid worker_id".to_string()))?;
        
        let datacenter_id = lines[1].trim().parse::<u64>()
            .map_err(|_| WorkerError::ParseError("Invalid datacenter_id".to_string()))?;
        
        let last_timestamp = lines[2].trim().parse::<u64>()
            .map_err(|_| WorkerError::ParseError("Invalid last_timestamp".to_string()))?;
        
        let creation_time = lines[3].trim().parse::<u64>()
            .map_err(|_| WorkerError::ParseError("Invalid creation_time".to_string()))?;

        Ok(WorkerInfo {
            worker_id,
            datacenter_id,
            last_timestamp,
            creation_time,
        })
    }

    pub fn to_file_content(&self) -> String {
        format!("{}\n{}\n{}\n{}\n", 
            self.worker_id, 
            self.datacenter_id, 
            self.last_timestamp, 
            self.creation_time
        )
    }

    pub fn update_timestamp(&mut self) {
        self.last_timestamp = current_millis();
    }

    pub fn check_clock_backwards(&self) -> Result<(), WorkerError> {
        let current_time = current_millis();
        if current_time < self.last_timestamp {
            let diff = self.last_timestamp - current_time;
            return Err(WorkerError::ClockBackwardsError(
                format!("Clock moved backwards by {} milliseconds. Last: {}, Current: {}", 
                    diff, self.last_timestamp, current_time)
            ));
        }
        Ok(())
    }
}

pub struct WorkerManager {
    file_path: String,
    worker_info: WorkerInfo,
}

impl WorkerManager {
    pub fn new(file_path: &str, default_datacenter_id: u64) -> Result<Self, WorkerError> {
        let worker_info = if Path::new(file_path).exists() {
            // 读取现有文件
            let mut file = File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            println!("Found existing worker config file: {}", file_path);
            let info = WorkerInfo::from_file_content(&contents)?;
            
            // 检查时钟回拨
            info.check_clock_backwards()?;
            
            println!("Worker ID: {}, Datacenter ID: {}", info.worker_id, info.datacenter_id);
            println!("Creation time: {}", format_timestamp(info.creation_time));
            println!("Last timestamp: {}", format_timestamp(info.last_timestamp));
            
            info
        } else {
            // 生成新的 worker ID
            let worker_id = generate_worker_id();
            let info = WorkerInfo::new(worker_id, default_datacenter_id);
            
            println!("Creating new worker config file: {}", file_path);
            println!("Generated Worker ID: {}, Datacenter ID: {}", info.worker_id, info.datacenter_id);
            println!("Creation time: {}", format_timestamp(info.creation_time));
            
            info
        };

        let manager = WorkerManager {
            file_path: file_path.to_string(),
            worker_info,
        };

        // 保存当前状态到文件
        manager.save_to_file()?;
        
        Ok(manager)
    }

    pub fn get_worker_info(&self) -> &WorkerInfo {
        &self.worker_info
    }

    pub fn update_and_save(&mut self) -> Result<(), WorkerError> {
        // 再次检查时钟回拨
        self.worker_info.check_clock_backwards()?;
        
        // 更新时间戳
        self.worker_info.update_timestamp();
        
        // 保存到文件
        self.save_to_file()?;
        
        Ok(())
    }

    fn save_to_file(&self) -> Result<(), WorkerError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;
        
        file.write_all(self.worker_info.to_file_content().as_bytes())?;
        Ok(())
    }

    pub fn get_worker_id(&self) -> u64 {
        self.worker_info.worker_id
    }

    pub fn get_datacenter_id(&self) -> u64 {
        self.worker_info.datacenter_id
    }
}

fn current_millis() -> u64 {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    dur.as_millis() as u64
}

fn generate_worker_id() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // 基于机器名和当前时间生成 worker ID
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    
    let mut hasher = DefaultHasher::new();
    hostname.hash(&mut hasher);
    current_millis().hash(&mut hasher);
    
    // 确保 worker ID 在有效范围内 (0-31)
    (hasher.finish() % 32) as u64
}

fn format_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    
    let datetime = UNIX_EPOCH + Duration::from_millis(timestamp);
    format!("{:?}", datetime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_worker_info_serialization() {
        let info = WorkerInfo::new(1, 2);
        let content = info.to_file_content();
        let parsed_info = WorkerInfo::from_file_content(&content).unwrap();
        
        assert_eq!(info.worker_id, parsed_info.worker_id);
        assert_eq!(info.datacenter_id, parsed_info.datacenter_id);
    }

    #[test]
    fn test_clock_backwards_detection() {
        let mut info = WorkerInfo::new(1, 2);
        // 模拟时钟回拨
        info.last_timestamp = current_millis() + 10000; // 未来时间
        
        assert!(info.check_clock_backwards().is_err());
    }

    #[test]
    fn test_worker_manager_creation() {
        let test_file = "test_worker.conf";
        
        // 清理测试文件
        let _ = fs::remove_file(test_file);
        
        // 创建新的 WorkerManager
        let _manager = WorkerManager::new(test_file, 1).unwrap();
        assert!(Path::new(test_file).exists());
        
        // 清理测试文件
        let _ = fs::remove_file(test_file);
    }
}
