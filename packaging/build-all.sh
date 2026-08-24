#!/bin/bash
#
# remdb-server 一键构建所有安装包
#
# 用法: ./build-all.sh [version]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "========================================"
echo "remdb-server 综合安装包构建"
echo "========================================"
echo ""

# 构建 tar.gz
echo ">>> 构建 tar.gz 二进制安装包..."
bash "$SCRIPT_DIR/build-tarball.sh" "$@"
echo ""

# 构建 .deb
echo ">>> 构建 .deb 安装包..."
bash "$SCRIPT_DIR/build-deb.sh" "$@"
echo ""

echo "========================================"
echo "所有安装包构建完成!"
echo "========================================"
ls -lh "$SCRIPT_DIR"/*.tar.gz "$SCRIPT_DIR"/*.deb 2>/dev/null
echo ""