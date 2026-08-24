#!/bin/bash
#
# remdb-server 安装脚本
# 用于从 tar.gz 安装包中安装 remdb-server
#

set -euo pipefail

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 安装路径
PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="${PREFIX}/bin"
ETC_DIR="/etc/remdb"
SYSTEMD_DIR="/lib/systemd/system"
SHARE_DIR="${PREFIX}/share/remdb"
DOC_DIR="${PREFIX}/share/doc/remdb-server"

echo "========================================"
echo "remdb-server 安装程序"
echo "========================================"
echo ""

# 检查 root 权限
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}错误: 安装需要 root 权限。请使用 sudo 运行。${NC}"
    exit 1
fi

# 创建用户
if ! id -u remdb &>/dev/null; then
    echo "创建 remdb 系统用户..."
    useradd --system --no-create-home --shell /usr/sbin/nologin remdb
fi

# 创建目录
echo "创建目录..."
mkdir -p "$BIN_DIR"
mkdir -p "$ETC_DIR"
mkdir -p "$SHARE_DIR/schema"
mkdir -p "$SHARE_DIR/examples"
mkdir -p "$DOC_DIR"

# 复制二进制文件
echo "安装二进制文件..."
cp bin/remdb-server "$BIN_DIR/"
cp bin/remdbcli "$BIN_DIR/"
chmod 755 "$BIN_DIR/remdb-server" "$BIN_DIR/remdbcli"

# 复制配置文件
echo "安装配置文件..."
if [ ! -f "$ETC_DIR/remdb-master.toml" ]; then
    cp etc/remdb/remdb-master.toml "$ETC_DIR/"
    echo -e "${YELLOW}  已创建 $ETC_DIR/remdb-master.toml${NC}"
else
    echo "  跳过 $ETC_DIR/remdb-master.toml (已存在)"
fi

if [ ! -f "$ETC_DIR/remdb-slave.toml" ]; then
    cp etc/remdb/remdb-slave.toml "$ETC_DIR/"
fi

# 默认配置
if [ ! -f "$ETC_DIR/remdb-server.conf" ]; then
    cp etc/remdb/remdb-server.conf "$ETC_DIR/"
    echo "  已创建 $ETC_DIR/remdb-server.conf"
fi

# systemd 服务文件
echo "安装 systemd 服务..."
cp lib/systemd/system/remdb-server.service "$SYSTEMD_DIR/"
chmod 644 "$SYSTEMD_DIR/remdb-server.service"

# 文档和示例
echo "安装文档和示例..."
cp -r share/remdb/schema/* "$SHARE_DIR/schema/" 2>/dev/null || true
cp -r share/remdb/examples/* "$SHARE_DIR/examples/" 2>/dev/null || true
cp -r share/doc/remdb-server/* "$DOC_DIR/"

# 创建数据目录
echo "创建数据目录..."
mkdir -p /var/lib/remdb
mkdir -p /var/log/remdb
chown -R remdb:remdb /var/lib/remdb /var/log/remdb

# 重新加载 systemd
echo "重新加载 systemd..."
systemctl daemon-reload

echo ""
echo "========================================"
echo -e "${GREEN}安装完成!${NC}"
echo "========================================"
echo ""
echo "快速开始:"
echo "  1. 编辑配置文件: vi $ETC_DIR/remdb-master.toml"
echo "  2. 启动服务:     sudo systemctl start remdb-server"
echo "  3. 设置开机启动: sudo systemctl enable remdb-server"
echo "  4. 查看状态:     sudo systemctl status remdb-server"
echo "  5. CLI 连接:     remdbcli"
echo ""
echo "日志文件: /var/log/remdb/"
echo "数据目录: /var/lib/remdb/"
echo "配置文件: $ETC_DIR/"
echo ""