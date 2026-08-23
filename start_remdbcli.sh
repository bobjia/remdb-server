#!/bin/bash
# remdb CLI 启动脚本

# 默认配置
LOG_LEVEL="info"
DEBUG=false

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--debug) DEBUG=true; shift 1 ;;
        -l|--log-level) LOG_LEVEL="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "  -d, --debug          启用调试模式 (默认: $DEBUG)"
            echo "  -l, --log-level      日志级别 (默认: $LOG_LEVEL)"
            echo "  -h, --help           显示帮助信息"
            exit 0 ;;
        *) echo "未知选项: $1"; exit 1 ;;
    esac
done

# 设置环境变量
export RUST_LOG="$LOG_LEVEL"
export RUST_BACKTRACE=1

# 构建项目
echo "正在构建 remdb CLI..."
cargo build --release

# 检查构建结果
BUILD_RESULT=$?
if [ $BUILD_RESULT -ne 0 ]; then
    echo "构建失败，请检查错误信息"
    exit $BUILD_RESULT
fi

# 构建启动命令
START_CMD="./target/release/remdbcli"

if [ "$DEBUG" = true ]; then
    START_CMD="$START_CMD --debug"
fi

# 显示配置信息
echo "========================================"
echo "remdb CLI 配置"
echo "========================================"
echo "调试模式: $DEBUG"
echo "日志级别: $LOG_LEVEL"
echo "========================================"

# 启动 CLI
echo "正在启动 remdb CLI..."
echo "命令: $START_CMD"
echo ""

$START_CMD