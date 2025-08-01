# 使用官方 Rust 镜像作为构建环境
FROM rust:1.80 AS builder

# 设置工作目录
WORKDIR /usr/src/app

# 复制 Cargo.toml 和 Cargo.lock（如果存在）
COPY Cargo.toml Cargo.lock ./

# 复制源代码
COPY src ./src
COPY benches ./benches
COPY config ./config

# 构建应用程序
RUN cargo build --release --bin snowflake_server

# 使用更小的基础镜像来运行应用程序
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -r -s /bin/false snowflake

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /usr/src/app/target/release/snowflake_server ./snowflake_server

# 复制配置文件
COPY --from=builder /usr/src/app/config ./config

# 设置权限
RUN chown -R snowflake:snowflake /app

# 切换到非 root 用户
USER snowflake

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# 运行应用程序
CMD ["./snowflake_server"]
