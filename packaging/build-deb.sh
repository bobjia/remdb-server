#!/bin/bash
#
# remdb-server .deb 包构建脚本
# 使用 cargo-deb 生成 Debian 安装包
#
# 用法: ./build-deb.sh [version]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# 版本号
if [ $# -ge 1 ]; then
    VERSION="$1"
else
    VERSION=$(grep '^version = ' "$PROJECT_DIR/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
fi

echo "========================================"
echo "remdb-server v${VERSION} .deb 包构建"
echo "========================================"

# 1. 复制 Debian 打包文件到项目根目录（cargo-deb 需要）
echo "[1/3] 准备 Debian 打包文件..."
mkdir -p "$PROJECT_DIR/debian"
cp "$SCRIPT_DIR/debian/control" "$PROJECT_DIR/debian/"
cp "$SCRIPT_DIR/debian/postinst" "$PROJECT_DIR/debian/"
cp "$SCRIPT_DIR/debian/prerm" "$PROJECT_DIR/debian/"
cp "$SCRIPT_DIR/debian/postrm" "$PROJECT_DIR/debian/"
cp "$SCRIPT_DIR/debian/copyright" "$PROJECT_DIR/debian/"

# 2. 构建 .deb 包
echo "[2/3] 构建 .deb 包..."
cd "$PROJECT_DIR"
cargo deb

# 3. 清理临时文件
echo "[3/3] 清理..."
rm -rf "$PROJECT_DIR/debian"

# 查找生成的 .deb 文件
DEB_FILE=$(ls -t target/debian/remdb-server_*.deb 2>/dev/null | head -1)

if [ -n "$DEB_FILE" ]; then
    # 复制到 packaging 目录
    cp "$DEB_FILE" "$SCRIPT_DIR/"
    echo ""
    echo "========================================"
    echo "构建完成!"
    echo "  .deb 包: $SCRIPT_DIR/$(basename "$DEB_FILE")"
    echo "版本:    v${VERSION}"
    echo "大小:    $(du -h "$DEB_FILE" | cut -f1)"
    echo "========================================"
    echo ""
    echo "安装方法:"
    echo "  sudo dpkg -i $(basename "$DEB_FILE")"
    echo "  或"
    echo "  sudo apt install ./$(basename "$DEB_FILE")"
    echo ""
else
    echo "错误: 未找到生成的 .deb 文件"
    exit 1
fi