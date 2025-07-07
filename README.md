# Snowflake ID Generator

一个高性能的雪花算法ID生成器，支持RESTful API服务、ID解析功能和多种时间优化方案。

## 特性

- 🚀 **高性能**: ID生成性能可达千万级/秒，缓存时间提供20-25倍性能提升
- 🔧 **Worker管理**: 自动生成和持久化worker ID，配置文件支持
- ⏰ **时钟回拨检测**: 检测并处理系统时钟回拨
- 🌐 **HTTP服务**: 内置RESTful API，支持生成、解析、批量生成和统计
- 🔍 **ID解析**: 完整的雪花ID反解析功能，支持时间戳、工作ID等信息提取
- 🧵 **线程安全**: 支持多线程并发环境
- 📊 **完整测试**: 单元测试、性能基准测试、示例程序
- 🐳 **Docker支持**: 一键容器化部署

## 项目结构

```
├── Cargo.toml              # 项目配置文件
├── README.md               # 项目说明文档
├── PARSE_GUIDE.md          # ID解析功能使用指南
├── Dockerfile              # Docker构建文件
├── docker-compose.yml      # Docker编排文件
├── src/                    # 源代码目录
│   ├── lib.rs              # 库入口文件
│   ├── snowflake.rs        # 雪花算法实现
│   ├── snowflake_core.rs   # 核心常量定义
│   ├── worker_manager.rs   # Worker管理器
│   ├── time_provider.rs    # 时间提供者
│   └── bin/                # 可执行文件
│       ├── main.rs         # 主程序演示
│       ├── snowflake_server.rs  # HTTP服务器
│       └── test_clock_backwards.rs  # 时钟回拨测试
├── examples/               # 示例代码
│   ├── basic_usage.rs      # 基本使用示例
│   ├── parse_id_example.rs # ID解析示例
│   └── detailed_parse_test.rs  # 详细解析测试
├── benches/                # 性能基准测试
├── tests/                  # 集成测试
├── config/                 # 配置文件
│   └── worker.conf         # Worker配置文件
└── .vscode/                # VS Code配置
    ├── launch.json         # 调试配置
    ├── tasks.json          # 任务配置
    └── settings.json       # 工作区设置
```

## 快速开始

### 1. 构建项目

```bash
# 构建release版本
cargo build --release

# 运行测试
cargo test

# 运行基准测试
cargo bench
```

### 2. 运行HTTP服务器

```bash
# 使用默认参数启动服务器
cargo run --bin snowflake_server

# 指定端口和ID参数
cargo run --bin snowflake_server -- --port 8080 --worker-id 1 --datacenter-id 1

# 使用配置文件
cargo run --bin snowflake_server -- --config-file config/worker.conf
```

### 3. 运行示例

```bash
# 基本使用示例
cargo run --example basic_usage

# ID解析示例
cargo run --example parse_id_example

# 详细解析测试
cargo run --example detailed_parse_test
```

## 使用方法

### 1. 程序库调用

```rust
use snowflake_generator::Snowflake;

// 创建雪花ID生成器
let mut snowflake = Snowflake::new(1, 1); // worker_id=1, datacenter_id=1

// 生成ID
let id = snowflake.next_id().unwrap();
println!("Generated ID: {}", id);

// 解析ID
let info = Snowflake::parse_id(id);
println!("Timestamp: {}", info.timestamp);
println!("Worker ID: {}", info.worker_id);
println!("Datacenter ID: {}", info.datacenter_id);
println!("Sequence: {}", info.sequence);
```

### 2. 使用配置文件

```rust
use snowflake_generator::{Snowflake, WorkerError};

fn main() -> Result<(), WorkerError> {
    // 使用配置文件创建Snowflake实例
    let mut snowflake = Snowflake::new_with_config("config/worker.conf", 1)?;
    
    // 生成ID
    let id = snowflake.next_id()?;
    println!("Generated ID: {}", id);
    
    Ok(())
}
```

## 配置文件

Worker配置文件 (`config/worker.conf`) 格式：

```
1          # worker_id
1          # datacenter_id
1751213037258  # last_timestamp
1751213037258  # creation_time
```

## Docker部署

```bash
# 构建Docker镜像
docker build -t snowflake-generator .

# 运行容器
docker run -d -p 8080:8080 snowflake-generator

# 使用docker-compose
docker-compose up -d
```

## 性能说明

- **时间获取优化**: 缓存时间方案提供20-25倍性能提升
- **ID生成性能**: 单核可达千万级/秒
- **并发支持**: 线程安全，支持多核并发
- **内存占用**: 极低内存占用
- **时间精度**: 毫秒级精度，满足雪花算法要求

## 雪花ID结构

```
| 1位符号位 | 41位时间戳 | 5位数据中心ID | 5位工作ID | 12位序列号 |
|    0     |  timestamp | datacenter_id | worker_id | sequence  |
```

- **时间戳**: 相对于EPOCH (2021-01-01 00:00:00 UTC)的毫秒数
- **数据中心ID**: 0-31，标识数据中心
- **工作ID**: 0-31，标识工作节点  
- **序列号**: 0-4095，同一毫秒内的序列号

## HTTP API

### 启动服务器

```bash
cargo run --bin snowflake_server -- --port 8080 --worker-id 1 --datacenter-id 1
```

### 主要端点

| 端点 | 方法 | 描述 | 示例 |
|------|------|------|------|
| `/health` | GET | 健康检查 | `curl http://localhost:8080/health` |
| `/id` | GET | 生成单个雪花ID | `curl http://localhost:8080/id` |
| `/batch` | GET | 批量生成ID | `curl http://localhost:8080/batch?count=10` |
| `/parse/{id}` | GET | 解析雪花ID | `curl http://localhost:8080/parse/1234567890` |
| `/stats` | GET | 服务器统计信息 | `curl http://localhost:8080/stats` |

演示地址(2c2g小水管）： http://id.demo.codersun.cn/id

### 响应示例

**生成ID** (`/id`):
```json
{
  "id": 596623079686410240,
  "worker_id": 1,
  "datacenter_id": 1,
  "timestamp": 1751705226918
}
```

**解析ID** (`/parse/{id}`):
```json
{
  "id": 596623079686410240,
  "id_hex": "0x0847a187a9821000",
  "timestamp": 1751705226918,
  "datacenter_id": 1,
  "worker_id": 1,
  "sequence": 0,
  "details": "完整格式化信息..."
}
```

## 开发说明

### 调试环境

项目配置了VS Code调试环境：

1. 打开VS Code
2. 按F5启动调试
3. 选择要调试的程序（main 或 snowflake_server）

### 测试覆盖

```bash
# 运行单元测试
cargo test

# 运行基准测试
cargo bench

# 运行时钟回拨测试
cargo run --bin test_clock_backwards
```

## 贡献

欢迎提交Issue和Pull Request来改进这个项目。

## 许可证

MIT License
