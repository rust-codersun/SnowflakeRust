#!/bin/bash

# 雪花算法 ID 生成器简单启动脚本

# 设置颜色
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查是否已经在运行
check_running() {
    if pgrep -f "snowflake_server" > /dev/null; then
        log_info "雪花算法服务已在运行"
        return 0
    else
        return 1
    fi
}

# 构建项目
build_project() {
    log_info "构建雪花算法 ID 生成器..."
    cd "$PROJECT_ROOT"
    
    if cargo build --release --bin snowflake_server; then
        log_info "构建成功"
        return 0
    else
        log_error "构建失败"
        return 1
    fi
}

# 启动服务
start_server() {
    log_info "启动雪花算法 ID 生成器服务..."
    cd "$PROJECT_ROOT"
    
    # 检查是否已经在运行
    if check_running; then
        log_warn "服务已在运行，请先停止服务"
        return 1
    fi
    
    # 启动服务
    nohup ./target/release/snowflake_server \
        --host 0.0.0.0 \
        --port 8080 \
        --worker-id 1 \
        --datacenter-id 1 \
        --time-provider cached \
        > snowflake_server.log 2>&1 &
    
    local server_pid=$!
    echo $server_pid > snowflake_server.pid
    
    # 等待服务启动
    sleep 3
    
    # 检查服务是否启动成功
    if check_running; then
        log_info "服务启动成功"
        log_info "服务 PID: $server_pid"
        log_info "服务地址: http://localhost:8080"
        log_info "日志文件: $PROJECT_ROOT/snowflake_server.log"
        return 0
    else
        log_error "服务启动失败"
        return 1
    fi
}

# 停止服务
stop_server() {
    log_info "停止雪花算法 ID 生成器服务..."
    
    if [ -f snowflake_server.pid ]; then
        local pid=$(cat snowflake_server.pid)
        if kill -0 $pid 2>/dev/null; then
            kill $pid
            log_info "服务已停止 (PID: $pid)"
            rm -f snowflake_server.pid
        else
            log_warn "服务进程不存在"
            rm -f snowflake_server.pid
        fi
    else
        # 尝试通过进程名停止
        if pgrep -f "snowflake_server" > /dev/null; then
            pkill -f "snowflake_server"
            log_info "服务已停止"
        else
            log_warn "服务未运行"
        fi
    fi
}

# 查看服务状态
check_status() {
    log_info "检查服务状态..."
    
    if check_running; then
        log_info "✓ 服务正在运行"
        
        # 测试健康检查
        if curl -f http://localhost:8080/health > /dev/null 2>&1; then
            log_info "✓ 健康检查通过"
        else
            log_warn "✗ 健康检查失败"
        fi
        
        # 显示进程信息
        ps aux | grep "[s]nowflake_server"
        
        return 0
    else
        log_info "✗ 服务未运行"
        return 1
    fi
}

# 查看日志
view_logs() {
    log_info "查看服务日志..."
    
    if [ -f snowflake_server.log ]; then
        tail -f snowflake_server.log
    else
        log_warn "日志文件不存在"
    fi
}

# 测试服务
test_service() {
    log_info "测试雪花算法 ID 生成器服务..."
    
    # 检查服务是否运行
    if ! check_running; then
        log_error "服务未运行，请先启动服务"
        return 1
    fi
    
    # 测试健康检查
    log_info "测试健康检查..."
    if curl -f http://localhost:8080/health; then
        log_info "✓ 健康检查通过"
    else
        log_error "✗ 健康检查失败"
        return 1
    fi
    
    # 测试生成单个 ID
    log_info "测试生成单个 ID..."
    ID_RESPONSE=$(curl -s http://localhost:8080/id)
    if [[ $? -eq 0 ]]; then
        log_info "✓ 单个 ID 生成成功: $ID_RESPONSE"
    else
        log_error "✗ 单个 ID 生成失败"
        return 1
    fi
    
    # 测试批量生成 ID
    log_info "测试批量生成 ID..."
    BATCH_RESPONSE=$(curl -s "http://localhost:8080/batch?count=5")
    if [[ $? -eq 0 ]]; then
        log_info "✓ 批量 ID 生成成功: $BATCH_RESPONSE"
    else
        log_error "✗ 批量 ID 生成失败"
        return 1
    fi
    
    # 测试统计信息
    log_info "测试统计信息..."
    STATS_RESPONSE=$(curl -s http://localhost:8080/stats)
    if [[ $? -eq 0 ]]; then
        log_info "✓ 统计信息获取成功: $STATS_RESPONSE"
    else
        log_error "✗ 统计信息获取失败"
        return 1
    fi
    
    log_info "所有测试通过！"
}

# 显示帮助信息
show_help() {
    echo "雪花算法 ID 生成器简单启动脚本"
    echo ""
    echo "用法:"
    echo "  $0 [选项]"
    echo ""
    echo "选项:"
    echo "  build      构建项目"
    echo "  start      启动服务"
    echo "  stop       停止服务"
    echo "  restart    重启服务"
    echo "  status     查看状态"
    echo "  logs       查看日志"
    echo "  test       测试服务"
    echo "  help       显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 build && $0 start"
    echo "  $0 test"
    echo "  $0 stop"
}

# 主函数
main() {
    case "${1:-help}" in
        build)
            build_project
            ;;
        start)
            start_server
            ;;
        stop)
            stop_server
            ;;
        restart)
            stop_server
            sleep 2
            start_server
            ;;
        status)
            check_status
            ;;
        logs)
            view_logs
            ;;
        test)
            test_service
            ;;
        help|*)
            show_help
            ;;
    esac
}

# 运行主函数
main "$@"
