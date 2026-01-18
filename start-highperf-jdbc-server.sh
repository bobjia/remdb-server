#!/bin/bash
# 高性能JDBC服务器启动脚本

# 默认配置
JDBC_PORT=6666
MAX_CONNECTIONS=10
AUTH_ENABLED=false
USERNAME="root"
PASSWORD_HASH=""
LOG_LEVEL="info"

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port) JDBC_PORT="$2"; shift 2 ;;
        -c|--connections) MAX_CONNECTIONS="$2"; shift 2 ;;
        --auth-enabled) AUTH_ENABLED=true; shift 1 ;;
        --username) USERNAME="$2"; shift 2 ;;
        --password-hash) PASSWORD_HASH="$2"; shift 2 ;;
        -l|--log-level) LOG_LEVEL="$2"; shift 2 ;;
        -h|--help) 
            echo "Usage: $0 [OPTIONS]"
            echo "  -p, --port              JDBC服务器端口 (默认: $JDBC_PORT)"
            echo "  -c, --connections      最大连接数 (默认: $MAX_CONNECTIONS)"
            echo "  --auth-enabled        启用认证 (默认: $AUTH_ENABLED)"
            echo "  --username            认证用户名"
            echo "  --password-hash       认证密码哈希值 (SHA-256)"
            echo "  -l, --log-level       日志级别 (默认: $LOG_LEVEL)"
            echo "  -h, --help            显示帮助信息"
            exit 0 ;;
        *) echo "未知选项: $1"; exit 1 ;;
    esac
done

# 设置环境变量
export RUST_LOG="$LOG_LEVEL"
export RUST_BACKTRACE=1

# 构建项目
echo "正在构建高性能JDBC服务器..."
cargo build --release

# 获取构建结果
BUILD_RESULT=$?
if [ $BUILD_RESULT -ne 0 ]; then
    echo "构建失败，请检查错误信息"
    exit $BUILD_RESULT
fi

# 设置CPU亲和性（可选，仅在Linux上支持）
if [[ "$(uname)" == "Linux" ]]; then
    # 获取可用CPU核心列表
    CPU_LIST=$(seq -s, 0 $(($(nproc) - 1)))
    echo "使用CPU亲和性: $CPU_LIST"
    AFFINITY_CMD="taskset -c $CPU_LIST"
else
    AFFINITY_CMD=""
fi

# 构建启动命令
START_CMD="$AFFINITY_CMD ./target/release/remdb-server"

# 添加配置参数
START_CMD="$START_CMD --jdbc-enabled true --jdbc-port $JDBC_PORT --max-connections $MAX_CONNECTIONS"

if [ "$AUTH_ENABLED" = true ]; then
    START_CMD="$START_CMD --jdbc-auth-enabled true --jdbc-username $USERNAME --jdbc-password-hash $PASSWORD_HASH"
fi

# 显示配置信息
echo "========================================"
echo "高性能JDBC服务器配置"
echo "========================================"
echo "JDBC端口: $JDBC_PORT"
echo "最大连接数: $MAX_CONNECTIONS"
echo "认证启用: $AUTH_ENABLED"
echo "日志级别: $LOG_LEVEL"
echo "========================================"

# 启动服务器
echo "正在启动高性能JDBC服务器..."
echo "命令: $START_CMD"

# 执行启动命令
$START_CMD
