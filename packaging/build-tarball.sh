#!/bin/bash
#
# remdb-server 二进制安装包构建脚本
# 构建一个自包含的 .tar.gz 安装包，包含预编译的二进制文件、配置文件和安装脚本
#
# 用法: ./build-tarball.sh [version]
#   如果未指定版本，则从 Cargo.toml 中读取

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# 版本号
if [ $# -ge 1 ]; then
    VERSION="$1"
else
    VERSION=$(grep '^version = ' "$PROJECT_DIR/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
fi

PACKAGE_NAME="remdb-server-${VERSION}-linux-amd64"
BUILD_DIR="$SCRIPT_DIR/build/$PACKAGE_NAME"
TARBALL="$SCRIPT_DIR/$PACKAGE_NAME.tar.gz"

echo "========================================"
echo "remdb-server v${VERSION} 二进制安装包构建"
echo "========================================"

# 1. 构建 release 二进制文件
echo ""
echo "[1/4] 构建 release 二进制文件..."
cd "$PROJECT_DIR"
cargo build --release

# 2. 创建安装包目录结构
echo "[2/4] 创建安装包目录结构..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/bin"
mkdir -p "$BUILD_DIR/etc/remdb"
mkdir -p "$BUILD_DIR/lib/systemd/system"
mkdir -p "$BUILD_DIR/share/remdb/examples"
mkdir -p "$BUILD_DIR/share/remdb/schema"
mkdir -p "$BUILD_DIR/share/doc/remdb-server"

# 3. 复制文件
echo "[3/4] 复制文件..."

# 二进制文件
cp "$PROJECT_DIR/target/release/remdb-server" "$BUILD_DIR/bin/"
cp "$PROJECT_DIR/target/release/remdbcli" "$BUILD_DIR/bin/"

# 使用 strip 减小体积
strip "$BUILD_DIR/bin/remdb-server"
strip "$BUILD_DIR/bin/remdbcli"

# 配置文件
cp "$PROJECT_DIR/remdb-master.toml" "$BUILD_DIR/etc/remdb/remdb-master.toml"
cp "$PROJECT_DIR/remdb-slave.toml" "$BUILD_DIR/etc/remdb/remdb-slave.toml"

# 默认配置
cp "$SCRIPT_DIR/remdb-server.default" "$BUILD_DIR/etc/remdb/remdb-server.conf"

# systemd 服务文件
cp "$SCRIPT_DIR/remdb-server.service" "$BUILD_DIR/lib/systemd/system/"

# 示例 schema
cp "$PROJECT_DIR/schema.ddl" "$BUILD_DIR/share/remdb/schema/"

# 文档
cp "$PROJECT_DIR/README.md" "$BUILD_DIR/share/doc/remdb-server/"
cp "$PROJECT_DIR/LICENSE" "$BUILD_DIR/share/doc/remdb-server/"
cp "$PROJECT_DIR/SPEC.md" "$BUILD_DIR/share/doc/remdb-server/"
cp "$PROJECT_DIR/PERFORMANCE_TUNING.md" "$BUILD_DIR/share/doc/remdb-server/"

# 启动脚本
cp "$PROJECT_DIR/start-highperf-jdbc-server.sh" "$BUILD_DIR/share/remdb/examples/"
cp "$PROJECT_DIR/start_remdbcli.sh" "$BUILD_DIR/share/remdb/examples/"

# 安装脚本
cp "$SCRIPT_DIR/install.sh" "$BUILD_DIR/"
chmod +x "$BUILD_DIR/install.sh"

# 4. 打包
echo "[4/4] 打包为 tar.gz..."
cd "$SCRIPT_DIR/build"
tar czf "$TARBALL" "$PACKAGE_NAME/"
rm -rf "$PACKAGE_NAME"

echo ""
echo "========================================"
echo "构建完成!"
echo "  安装包: $TARBALL"
echo "  版本:   v${VERSION}"
echo "  大小:   $(du -h "$TARBALL" | cut -f1)"
echo "========================================"
echo ""
echo "安装方法:"
echo "  tar xzf $TARBALL"
echo "  cd $PACKAGE_NAME"
echo "  sudo ./install.sh"
echo ""