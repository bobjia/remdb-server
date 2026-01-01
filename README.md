# remdb-server

remdb-server 是一个基于 remdb 库构建的轻量级数据库服务器，支持 JDBC 连接、DDL 编译、快照管理、PubSub 功能和高可用配置。

## 功能特性

### 核心功能
- **轻量级数据库服务器**：基于 remdb 库，提供高效的数据存储和检索
- **JDBC 支持**：提供 JDBC 接口，方便 Java 应用程序连接
- **DDL 支持**：支持通过 DDL 文件定义数据库表结构
- **快照管理**：支持从快照目录加载数据，实现数据持久化
- **交互式 CLI**：提供命令行界面，方便直接操作数据库
- **数据导出**：支持导出 DDL、表数据和全部数据

### 高级功能
- **高可用配置**：支持主从复制，提供数据冗余和故障转移
- **PubSub 功能**：基于 UDP 的发布订阅机制，支持实时数据推送
- **多连接支持**：支持最大连接数配置，适应高并发场景
- **调试模式**：提供详细的调试日志，方便开发和故障排查

## 技术栈

- **编程语言**：Rust 2024
- **异步运行时**：Tokio
- **命令行解析**：Clap
- **配置管理**：Toml + Serde
- **日志系统**：Log
- **网络通信**：Tokio + Tokio-native-tls
- **序列化**：Serde

## 安装和构建

### 前提条件
- Rust 编译器（版本 1.70+）
- Cargo 包管理器

### 构建步骤

1. 克隆项目仓库
2. 进入项目目录
3. 执行构建命令

```bash
git clone <repository-url>
cd remdb-server
cargo build --release
```

构建完成后，可执行文件将位于 `target/release/remdb-server`。

## 运行和配置

### 基本运行

```bash
# 使用默认配置运行
./remdb-server

# 指定配置文件
./remdb-server --config config.toml

# 启用调试模式
./remdb-server --debug
```

### 命令行参数

| 参数 | 长格式 | 描述 |
|------|--------|------|
| `-c` | `--config` | 配置文件路径 |
|      | `--ddl` | DDL 文件路径 |
|      | `--snapshot_dir` | 快照存储目录 |
|      | `--full_image` | 全量镜像文件路径 |
|      | `--total_memory` | 数据库总内存大小（字节） |
|      | `--default_max_records` | 默认最大记录数 |
|      | `--low_power_mode_supported` | 是否支持低功耗模式 |
|      | `--low_power_max_records` | 低功耗模式下的最大记录数 |
|      | `--snapshot_interval` | 增量快照周期（秒） |
|      | `--max_incremental_snapshots` | 最大增量快照数量 |
| `-d` | `--debug` | 是否开启调试模式 |
|      | `--non_interactive` | 非交互式模式（初始化后退出） |
|      | `--test_export` | 测试导出功能 |
|      | `--jdbc_port` | JDBC 监听端口 |
|      | `--max_connections` | 最大允许的并发 JDBC 客户端连接数 |
|      | `--pubsub_enabled` | 是否启用 PubSub 功能 |
|      | `--pubsub_udp_bind` | UDP 绑定地址 |
|      | `--pubsub_heartbeat` | 心跳间隔（毫秒） |
|      | `--pubsub_retrans_timeout` | 重传超时（毫秒） |
|      | `--pubsub_max_retrans` | 最大重传次数 |
|      | `--ha_enabled` | 是否启用高可用功能 |
|      | `--ha_role` | 节点角色（master/slave） |
|      | `--ha_replication_mode` | 复制模式（async/sync） |
|      | `--ha_heartbeat_interval` | 心跳间隔（毫秒） |
|      | `--ha_failure_detection_ms` | 故障检测时间（毫秒） |
|      | `--ha_sync_timeout_ms` | 同步超时时间（毫秒） |
|      | `--ha_master_address` | 主节点地址（仅 slave 节点需要） |
|      | `--ha_master_port` | 主节点端口（仅 slave 节点需要） |

### 配置文件

配置文件使用 TOML 格式，示例配置：

```toml
# DDL 文件路径
ddl = "schema.ddl"

# 快照存储目录
snapshot_dir = "./snapshots"

# 数据库总内存大小（字节）
total_memory = 104857600  # 100MB

# 默认最大记录数
default_max_records = 1000

# 是否支持低功耗模式
low_power_mode_supported = true

# 低功耗模式下的最大记录数
low_power_max_records = 100

# 增量快照周期（秒）
snapshot_interval = 3600

# 最大增量快照数量
max_incremental_snapshots = 5

# 是否开启debug模式
debug = false

# JDBC监听端口
jdbc_port = 5432

# 最大允许的并发jdbc客户端连接数
max_connections = 100

# pubsub配置
[pubsub]
# 是否启用pubsub功能
enabled = true
# UDP绑定地址
udp_bind_address = "0.0.0.0:9000"
# 心跳间隔（毫秒）
heartbeat_interval = 1000
# 重传超时（毫秒）
retransmission_timeout = 500
# 最大重传次数
max_retransmissions = 3

# 高可用配置
[ha]
# 是否启用高可用功能
enabled = true
# 节点角色（master/slave）
role = "master"
# 复制模式（async/sync）
replication_mode = "async"
# 心跳间隔（毫秒）
heartbeat_interval = 1000
# 故障检测时间（毫秒）
failure_detection_ms = 5000
# 同步超时时间（毫秒）
sync_timeout_ms = 2000
```

## 使用示例

### 1. 基本运行

```bash
# 启动服务器，使用默认配置
./remdb-server

# 启动服务器，指定JDBC端口
./remdb-server --jdbc_port 5432
```

### 2. 使用DDL文件

```bash
# 编译DDL文件并启动服务器
./remdb-server --ddl schema.ddl
```

### 3. 加载快照

```bash
# 从快照目录加载数据
./remdb-server --snapshot_dir ./snapshots
```

### 4. 非交互式模式

```bash
# 初始化数据库后退出
./remdb-server --ddl schema.ddl --non_interactive
```

### 5. 测试导出功能

```bash
# 测试数据导出功能
./remdb-server --ddl schema.ddl --test_export
```

## 高可用配置

### 主节点配置

```bash
./remdb-server --ha_enabled true --ha_role master --ha_replication_mode async
```

### 从节点配置

```bash
./remdb-server --ha_enabled true --ha_role slave --ha_master_address 127.0.0.1 --ha_master_port 5432
```

## PubSub 功能

### 启用 PubSub

```bash
./remdb-server --pubsub_enabled true --pubsub_udp_bind 0.0.0.0:9000
```

### PubSub 配置项

| 配置项 | 描述 | 默认值 |
|--------|------|--------|
| `enabled` | 是否启用 PubSub 功能 | false |
| `udp_bind_address` | UDP 绑定地址 | 0.0.0.0:9000 |
| `heartbeat_interval` | 心跳间隔（毫秒） | 1000 |
| `retransmission_timeout` | 重传超时（毫秒） | 500 |
| `max_retransmissions` | 最大重传次数 | 3 |

## 导出功能

remdb-server 支持多种导出方式：

### 1. 导出 DDL

```sql
export ddl exported_schema.ddl
```

### 2. 导出表数据

```sql
export data users users.csv
export data products products.csv
export data orders orders.csv
```

### 3. 导出全部数据

```sql
export all export_all
```

## 交互式 CLI 命令

启动服务器后，可以使用以下 CLI 命令：

- `tables`：查看所有表
- `stat`：查看监控指标
- `healthcheck`：查看健康状态
- `export ddl <file>`：导出 DDL
- `export data <table> <file>`：导出表数据
- `export all <dir>`：导出全部数据
- SQL 查询命令：执行 SQL 查询

## 技术架构

### 主要模块

1. **ddl_compiler**：DDL 文件编译，将 DDL 定义转换为内部表结构
2. **snapshot_loader**：快照加载，从文件系统加载数据快照
3. **sql_engine**：SQL 引擎，执行 SQL 查询和命令
4. **cli**：命令行界面，提供交互式操作
5. **jdbc_server**：JDBC 服务器，处理 JDBC 连接请求
6. **udp_transport**：UDP 传输层，支持 PubSub 功能
7. **pubsub_server**：PubSub 服务器，提供发布订阅机制

### 数据流

1. 服务器启动，初始化平台和内存分配器
2. 编译 DDL 文件（如果提供），创建表结构
3. 加载快照数据（如果提供）
4. 启动 JDBC 服务器（如果配置）
5. 启动 PubSub 服务器（如果配置）
6. 进入交互式 CLI 或保持后台运行

## 性能特性

- **低延迟**：基于 Tokio 异步运行时，提供高效的 I/O 操作
- **高并发**：支持大量并发 JDBC 连接
- **内存高效**：优化的内存管理，支持低功耗模式
- **快速启动**：轻量级设计，启动时间短

## 开发和贡献

### 构建和测试

```bash
# 构建项目
cargo build

# 构建发布版本
cargo build --release

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 依赖管理

项目使用 Cargo 管理依赖，主要依赖包括：
- remdb：核心数据库库
- tokio：异步运行时
- clap：命令行解析
- toml：配置文件解析
- serde：序列化
- log：日志库

## 许可证

MIT License

## 联系方式

如有问题或建议，欢迎提交 Issue 或 Pull Request。

## 版本历史

### v0.1.0
- 初始版本
- 支持 JDBC 连接
- 支持 DDL 编译
- 支持快照管理
- 支持 PubSub 功能
- 支持高可用配置
- 提供交互式 CLI

## 致谢

感谢 remdb 库的开发者，以及所有为项目做出贡献的开发者。