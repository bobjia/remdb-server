# remdb-server

remdb-server 是一个基于 remdb 库构建的基于全流程零拷贝的高并发高性能轻量级数据库服务器，支持 JDBC 连接、DDL 编译、快照管理、PubSub 功能、高可用配置、Milvus 兼容 RESTful API 和 ONNX 模型推理。

## 功能特性

### 核心功能
- **轻量级数据库服务器**：基于 remdb 库，提供高效的数据存储和检索
- **高性能 JDBC 支持**：提供优化的 JDBC 驱动和服务器，支持零拷贝网络传输和高并发连接
- **DDL 支持**：支持通过 DDL 文件定义数据库表结构
- **快照管理**：支持从快照目录加载数据，实现数据持久化
- **交互式 CLI**：提供命令行界面，方便直接操作数据库
- **数据导出**：支持导出 DDL、表数据和全部数据
- **Milvus 兼容 RESTful API**：兼容 Milvus v2.x RESTful API 协议，支持向量数据库操作
- **Python 绑定**：提供完整的 Python 绑定（remdb-python），支持 NumPy/Pandas 集成
- **ONNX 模型推理**：支持 ONNX 运行时模型加载和推理，可作为 SQL UDF 调用

### 高级功能
- **高可用配置**：支持主从复制，提供数据冗余和故障转移
- **PubSub 功能**：基于 UDP 的发布订阅机制，支持实时数据推送
- **多连接支持**：支持最大连接数配置，适应高并发场景
- **调试模式**：提供详细的调试日志，方便开发和故障排查
- **高性能架构**：零拷贝网络传输、高性能连接池、动态系统调优
- **直接内存管理**：优化的内存分配和缓冲区管理，减少 GC 压力
- **Lock-free 队列**：高效的请求处理队列，减少线程阻塞
- **动态系统调优**：根据 CPU/memory 负载自动调整系统参数
- **性能监控**：内置监控指标，支持 Prometheus 集成

## 技术栈

- **编程语言**：Rust 2024
- **异步运行时**：Tokio
- **命令行解析**：Clap
- **配置管理**：Toml + Serde
- **日志系统**：Log
- **网络通信**：Tokio + Tonic + Prost（高性能gRPC/protobuf支持）
- **序列化**：Serde + Bincode
- **高性能数据结构**：Crossbeam（Lock-free队列）+ Dashmap（高性能哈希表）
- **并发控制**：Parking_lot（高性能锁）
- **系统调优**：Sysinfo（系统资源监控）
- **直接内存管理**：Mmap2（内存映射）

## 安装和构建

### 前提条件
- Rust 编译器（版本 1.70+）
- Cargo 包管理器
- Python 3.8+（可选，用于 Python 绑定）
- Java 8+（可选，用于 JDBC 驱动）
- Maven 3.6+（可选，用于编译 JDBC 驱动）

### 构建步骤

1. 克隆项目仓库
2. 进入项目目录
3. 执行构建命令

```bash
git clone https://github.com/bobjia/remdb-server
cd remdb-server
cargo build --release
```

构建完成后，可执行文件将位于 `target/release/remdb-server` 和 `target/release/remdbcli`。

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
|      | `--milvus_enabled` | 是否启用 Milvus RESTful API 服务 |
|      | `--milvus_port` | Milvus RESTful API 监听端口（默认 19530） |
|      | `--milvus_api_key` | Milvus RESTful API 认证密钥 |
|      | `--non_interactive` | 非交互式模式（初始化后退出） |
|      | `--test_export` | 测试导出功能 |
|      | `--jdbc_port` | JDBC 监听端口 |
|      | `--jdbc_enabled` | 是否启用 JDBC 服务 |
|      | `--max_connections` | 最大允许的并发 JDBC 客户端连接数 |
|      | `--jdbc_timeout` | JDBC 执行超时时间（秒） |
|      | `--jdbc_auth_enabled` | 是否启用 JDBC 认证 |
|      | `--jdbc_username` | JDBC 认证用户名 |
|      | `--jdbc_password_hash` | JDBC 认证密码哈希值 |
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

# 是否启用JDBC服务
jdbc_enabled = true

# JDBC监听端口
jdbc_port = 5432

# 最大允许的并发jdbc客户端连接数
max_connections = 100

# JDBC执行超时时间（秒）
jdbc_timeout = 5

# JDBC认证配置
# 是否启用JDBC认证
jdbc_auth_enabled = false
# JDBC认证用户名
jdbc_username = "admin"
# JDBC认证密码哈希值
jdbc_password_hash = "8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918"

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

# Milvus 兼容 RESTful API 配置
[milvus]
# 是否启用 Milvus RESTful API 服务
enabled = false
# 监听端口（默认 19530）
port = 19530
# 认证密钥（可选）
api_key = "your-api-key"
```

## 使用示例

### 1. 基本运行

```bash
# 启动服务器，使用默认配置
./remdb-server

# 启动服务器，指定JDBC端口
./remdb-server --jdbc_port 5432

# 启用调试模式
./remdb-server --debug
```

### 2. 使用DDL文件

```bash
# 编译DDL文件并启动服务器
./remdb-server --ddl schema.ddl
```

### 3. 加载快照和全量镜像

```bash
# 从快照目录加载数据
./remdb-server --snapshot_dir ./snapshots

# 从全量镜像文件加载数据
./remdb-server --full_image ./full_image.remdb
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

### 6. JDBC服务配置

```bash
# 启用JDBC服务
./remdb-server --jdbc_enabled true --jdbc_port 6666

# 配置JDBC超时时间和最大连接数
./remdb-server --jdbc_port 6666 --jdbc_timeout 10 --max_connections 200

# 启用JDBC认证
./remdb-server --jdbc_port 6666 --jdbc_auth_enabled true --jdbc_username admin --jdbc_password_hash "8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918"
```

### 7. 高性能JDBC服务器

remdb-server提供了高性能JDBC服务器模式，采用零拷贝网络传输、高性能连接池和动态系统调优，支持百万级QPS处理能力。

#### 启动高性能JDBC服务器

使用提供的启动脚本：

```bash
# Linux/MacOS
chmod +x start-highperf-jdbc-server.sh
./start-highperf-jdbc-server.sh

# Windows
start-highperf-jdbc-server.sh
```

#### 高性能模式配置选项

| 参数 | 描述 | 建议值 |
|------|------|--------|
| `--port` | JDBC服务器端口 | 默认：6666 |
| `--admin-port` | 管理API端口 | 默认：9090 |
| `--threads` | 工作线程数 | 建议：CPU核心数 |
| `--connections` | 最大连接数 | 建议：10000-50000 |
| `--memory` | 内存限制 | 建议：系统内存的70-80% |
| `--auth-enabled` | 启用认证 | 根据需求选择 |
| `--log-level` | 日志级别 | 生产环境：info，调试：debug |

#### 环境变量调优

```bash
# 设置日志级别
export RUST_LOG=info

# 启用backtrace
export RUST_BACKTRACE=1

# 优化内存分配
export MALLOC_ARENA_MAX=4

# 禁用信号处理优化
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
```

#### 性能监控

高性能JDBC服务器提供了内置的Prometheus监控指标，访问 `http://localhost:9090/metrics` 可以获取以下指标：

- 请求处理延迟分布
- 每秒查询数(QPS)
- 活跃连接数
- 内存使用情况
- CPU使用率
- 工作线程状态

#### 性能调优指南

详细的性能调优指南请参考 [PERFORMANCE_TUNING.md](PERFORMANCE_TUNING.md) 文件，包括：

- Linux内核参数调优
- 资源限制调优
- CPU和内存调优
- 应用程序调优
- 客户端调优
- 性能测试方法
- 常见性能问题排查

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

## 基准测试

remdb-server 提供了内置的基准测试功能，可以测试不同负载下的系统性能，包括只读、只写和混合读写负载。

### 支持的测试类型

- **query**: 只读测试，只执行 SELECT 查询
- **write**: 只写测试，只执行 INSERT/UPDATE 操作
- **mix**: 混合读写测试，根据配置的读写比例执行不同类型的操作

### 基准测试命令行参数

| 参数 | 长格式 | 描述 | 默认值 |
|------|--------|------|--------|
|      | `--query_count` | 测试执行的查询总数 | 100000 |
|      | `--connections` | 并发连接数 | 16 |
|      | `--query_template` | 查询模板，使用 `{}` 作为参数占位符 | `SELECT * FROM test_table WHERE id = {}` |
|      | `--write_template` | 写入模板，使用 `{}` 作为参数占位符 | `INSERT INTO test_table (id, value) VALUES ({}, {}) ON DUPLICATE KEY UPDATE value = {}` |
|      | `--test_type` | 测试类型：`query`、`write` 或 `mix` | `query` |
|      | `--read_write_ratio` | 混合读写测试的读写比例，格式为 `8:2` | `8:2` |
|      | `--server_url` | 服务器 JDBC URL | `jdbc:remdb://localhost:6666` |

### 基准测试示例

#### 1. 只读测试

```bash
cargo run --bin remdb-server -- benchmark --test-type query --query-count 100000 --connections 16
```

#### 2. 只写测试

```bash
cargo run --bin remdb-server -- benchmark --test-type write --query-count 100000 --connections 16
```

#### 3. 混合读写测试（8:2比例）

```bash
cargo run --bin remdb-server -- benchmark --test-type mix --read-write-ratio 8:2 --query-count 100000 --connections 16
```

#### 4. 自定义读写比例（7:3）

```bash
cargo run --bin remdb-server -- benchmark --test-type mix --read-write-ratio 7:3 --query-count 100000 --connections 16
```

### 测试结果

基准测试运行完成后，会输出详细的性能指标，包括：

- **Total Queries**: 执行的查询总数
- **Total Time**: 总执行时间（秒）
- **Throughput**: 吞吐量（QPS，每秒查询数）
- **Average Latency**: 平均延迟（纳秒）
- **P95 Latency**: 95% 延迟（纳秒）
- **P99 Latency**: 99% 延迟（纳秒）
- **Min/Max Latency**: 最小/最大延迟（纳秒）
- **Successful/Failed Queries**: 成功/失败查询数

此外，测试结果还会生成 HTML 和 JSON 格式的报告文件：
- `benchmark_report.html`: 可视化的 HTML 报告
- `benchmark_results.json`: 结构化的 JSON 数据

### 报告生成

基准测试会自动生成两种格式的报告：

#### HTML报告

HTML 报告提供了可视化的性能指标展示，包括：
- 主要性能指标卡片
- 延迟分布柱状图
- 详细的结果表格

通过浏览器打开 `benchmark_report.html` 文件即可查看。

#### JSON报告

JSON 报告包含了结构化的性能数据，方便后续分析和处理。报告格式如下：

```json
{
  "total_queries": 100000,
  "total_time_secs": 0.95,
  "avg_latency_ns": 15245,
  "p95_latency_ns": 45890,
  "p99_latency_ns": 89760,
  "throughput_qps": 105263.16,
  "successful_queries": 99980,
  "failed_queries": 20,
  "min_latency_ns": 5678,
  "max_latency_ns": 123456,
  "test_type": "mix"
}
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

- **低延迟**：基于 Tokio 异步运行时，提供高效的 I/O 操作，平均延迟 < 100微秒
- **高并发**：支持 10,000+ 并发 JDBC 连接，吞吐量可达 100,000+ QPS
- **内存高效**：优化的内存管理，支持低功耗模式和直接内存缓冲区
- **快速启动**：轻量级设计，启动时间短
- **零拷贝网络传输**：减少数据复制开销，提高网络传输效率
- **高性能连接池**：优化的连接管理，减少连接创建和销毁开销
- **Lock-free 设计**：减少线程阻塞，提高并发性能
- **动态系统调优**：根据负载自动调整系统参数，优化资源利用
- **直接内存访问**：减少 GC 压力，提高内存访问效率

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

### v0.3.0
- 新增 Milvus 兼容 RESTful API 服务（v2.x 风格）
- 新增 Python 绑定（remdb-python）支持
- 新增 ONNX 模型推理支持
- 新增零拷贝网络传输层
- 新增高性能连接池
- 新增动态系统调优
- 新增 Prometheus 性能监控
- 大幅提升 JDBC 服务性能和并发能力

### v0.2.0
- 新增 JDBC 认证支持
- 新增快照导出功能
- 新增基准测试框架
- 优化 WAL 恢复机制
- 改进错误处理和日志记录

### v0.1.0
- 初始版本
- 支持 JDBC 连接
- 支持 DDL 编译
- 支持快照管理
- 支持 PubSub 功能
- 支持高可用配置
- 提供交互式 CLI

## JDBC 驱动

remdb-server 提供了完整的 JDBC 驱动，方便 Java 应用程序连接和操作数据库。更多详细信息请查看 [jdbc-driver/README.md](jdbc-driver/README.md)。

### 功能特性

- 支持基本的 JDBC 连接管理
- 支持 SQL 查询、插入、更新和删除操作
- 支持事务处理（自动提交模式）
- 支持结果集处理
- 支持预编译语句（PreparedStatement）

### 系统要求

- Java 8 或更高版本
- RemDb Server 0.1.0 或更高版本

### 安装方法

#### 使用 Maven 依赖

在你的 Maven 项目中添加以下依赖：

```xml
<dependency>
    <groupId>cn.totaltrust.remdb</groupId>
    <artifactId>remdb-jdbc-driver</artifactId>
    <version>0.1.0</version>
</dependency>
```

#### 手动编译 JAR 文件

```bash
# 使用 Maven 编译
cd jdbc-driver
mvn clean package

# 编译后的 JAR 文件将位于 target/remdb-jdbc-driver-0.1.0.jar
```

### 使用示例

```java
import java.sql.*;

public class RemDbExample {
    public static void main(String[] args) {
        String url = "jdbc:remdb://localhost:6666";
        String user = "";
        String password = "";

        try (Connection conn = DriverManager.getConnection(url, user, password);
             Statement stmt = conn.createStatement()) {

            // 创建表
            String createTableSQL = "CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY, name VARCHAR(50), age INT)";
            stmt.executeUpdate(createTableSQL);

            // 插入数据
            String insertSQL = "INSERT INTO users (id, name, age) VALUES (1, 'Alice', 25)";
            stmt.executeUpdate(insertSQL);

            // 查询数据
            String selectSQL = "SELECT id, name, age FROM users";
            try (ResultSet rs = stmt.executeQuery(selectSQL)) {
                while (rs.next()) {
                    int id = rs.getInt("id");
                    String name = rs.getString("name");
                    int age = rs.getInt("age");
                    System.out.printf("ID: %d, Name: %s, Age: %d%n", id, name, age);
                }
            }

        } catch (SQLException e) {
            e.printStackTrace();
        }
    }
}
```

### 使用JDBC认证的示例

```java
import java.sql.*;

public class RemDbAuthExample {
    public static void main(String[] args) {
        String url = "jdbc:remdb://localhost:6666";
        String user = "admin";
        String password = "password";

        try (Connection conn = DriverManager.getConnection(url, user, password);
             Statement stmt = conn.createStatement()) {

            // 执行查询
            String selectSQL = "SELECT id, name, age FROM users";
            try (ResultSet rs = stmt.executeQuery(selectSQL)) {
                while (rs.next()) {
                    int id = rs.getInt("id");
                    String name = rs.getString("name");
                    int age = rs.getInt("age");
                    System.out.printf("ID: %d, Name: %s, Age: %d%n", id, name, age);
                }
            }

        } catch (SQLException e) {
            System.err.println("Authentication failed or connection error: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
```

### JDBC URL 格式

```
jdbc:remdb://host:port
```

- `host`: RemDb Server 的主机名或 IP 地址，默认为 `localhost`
- `port`: RemDb Server 的 JDBC 监听端口，默认为 `6666`

### 支持的 SQL 语句

- `CREATE TABLE` - 创建表
- `INSERT` - 插入数据
- `SELECT` - 查询数据
- `UPDATE` - 更新数据
- `DELETE` - 删除数据
- `DROP TABLE` - 删除表

### JDBC认证

当JDBC认证启用时，客户端必须提供有效的用户名和密码才能连接到服务器。认证流程如下：
1. 服务器配置了用户名和密码哈希值
2. 客户端连接时提供用户名和明文密码
3. 服务器将客户端提供的密码进行哈希计算
4. 比较计算结果与配置的哈希值，匹配则认证成功

**注意**：密码哈希使用SHA-256算法生成，示例中的默认密码为"password"，其SHA-256哈希值为"8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918"

### 8. Milvus 兼容 RESTful API 服务

remdb-server 提供了兼容 Milvus v2.x RESTful API 协议的向量数据库服务，支持向量数据的管理和检索。

```bash
# 启动 Milvus RESTful API 服务
./remdb-server --milvus_enabled true --milvus_port 19530

# 启动 Milvus 服务并配置认证密钥
./remdb-server --milvus_enabled true --milvus_api_key "my-api-key"
```

Milvus 兼容 API 端点：

| 端点 | 方法 | 描述 |
|------|------|------|
| `/v2/vectordb/collections/create` | POST | 创建集合 |
| `/v2/vectordb/collections/drop` | POST | 删除集合 |
| `/v2/vectordb/collections/describe` | POST | 获取集合详情 |
| `/v2/vectordb/collections/list` | POST | 列出所有集合 |
| `/v2/vectordb/entities/insert` | POST | 插入实体 |
| `/v2/vectordb/entities/delete` | POST | 删除实体 |
| `/v2/vectordb/entities/query` | POST | 查询实体 |
| `/v2/vectordb/entities/search` | POST | 向量搜索 |
| `/v2/vectordb/entities/get` | POST | 获取实体 |

### 9. 使用 remdbcli（独立 CLI 模式）

remdb-server 还提供了独立的 CLI 二进制文件 `remdbcli`，适合不需要 JDBC 服务的场景：

```bash
# 启动独立 CLI 模式
cargo run --bin remdbcli

# 在 CLI 中执行 SQL 查询
> CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(50), age INT);
> INSERT INTO users (id, name, age) VALUES (1, 'Alice', 25);
> SELECT * FROM users;
```

## 相关项目

- [remdb-python](remdb-python/)：Python 绑定库，支持 NumPy/Pandas 集成
- [jdbc-driver](jdbc-driver/)：Java JDBC 驱动
- [SPEC.md](SPEC.md)：项目设计规格文档