# remdb-server 打包指南

本目录包含 remdb-server 的安装包构建脚本。

## 目录结构

```
packaging/
├── build-all.sh              # 一键构建所有安装包
├── build-tarball.sh          # 构建 tar.gz 二进制安装包
├── build-deb.sh              # 构建 .deb 安装包
├── install.sh                # tar.gz 安装包内置安装脚本
├── remdb-server.service      # systemd 服务单元文件
├── remdb-server.default      # 环境变量默认配置
├── debian/                   # Debian 打包脚本
│   ├── control               # 包控制信息
│   ├── postinst              # 安装后配置
│   ├── prerm                 # 卸载前清理
│   ├── postrm                # 卸载后清理
│   └── copyright             # 版权信息
└── README.md                 # 本文件
```

## 前提条件

- Rust 工具链 (rustc, cargo)
- `cargo-deb` (构建 .deb 时需要)
  ```bash
  cargo install cargo-deb
  ```
- `strip` (用于减小二进制体积, 通常包含在 binutils 中)

## 构建安装包

### 构建所有安装包

```bash
./packaging/build-all.sh
```

### 仅构建 tar.gz 二进制安装包

```bash
./packaging/build-tarball.sh
```

### 仅构建 .deb 安装包

```bash
./packaging/build-deb.sh
```

## 安装方法

### 从 tar.gz 安装

```bash
# 解压
tar xzf remdb-server-<version>-linux-amd64.tar.gz

# 安装
cd remdb-server-<version>-linux-amd64
sudo ./install.sh
```

### 从 .deb 安装

```bash
sudo dpkg -i remdb-server_<version>_amd64.deb
# 或
sudo apt install ./remdb-server_<version>_amd64.deb
```

## 启动服务

### 使用 systemd

```bash
sudo systemctl start remdb-server
sudo systemctl enable remdb-server
sudo systemctl status remdb-server
```

### 手动启动

```bash
remdb-server --config /etc/remdb/remdb-master.toml
```

## 配置文件

- 主配置: `/etc/remdb/remdb-master.toml`
- 从配置: `/etc/remdb/remdb-slave.toml`
- 环境变量: `/etc/remdb/remdb-server.conf`

## 数据目录

- 数据: `/var/lib/remdb/`
- 日志: `/var/log/remdb/`