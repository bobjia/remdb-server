# remdb-server

remdb-server是一个基于remdb库构建的轻量级内存数据库服务器，提供SQL查询支持和交互式命令行界面。

## 功能特性

- ✅ 基于remdb库的高性能内存数据库
- ✅ 支持DDL文件编译和表定义
- ✅ 快照加载和保存功能
- ✅ 交互式命令行界面
- ✅ SQL查询支持
- ✅ 配置文件支持
- ✅ 低功耗模式支持

## 安装和构建

### 前提条件

- Rust 1.70+（推荐使用rustup安装）
- Windows或Linux操作系统

### 构建步骤

1. 克隆仓库：
   ```bash
   git clone https://gitee.com/bobjia/remdb-server.git
   cd remdb-server
   ```

2. 构建项目：
   ```bash
   cargo build
   ```

3. 构建发布版本：
   ```bash
   cargo build --release
   ```

4. 运行测试：
   ```bash
   cargo test
   ```

## 使用方法

### 基本用法

```bash
# 运行服务器（交互式模式）
./target/debug/remdb-server

# 使用配置文件运行
./target/debug/remdb-server --config remdb.toml

# 使用DDL文件运行
./target/debug/remdb-server --ddl schema.ddl

# 非交互式模式运行
./target/debug/remdb-server --non-interactive
```

### 命令行参数

| 参数 | 简写 | 描述 |
|------|------|------|
| `--config <FILE>` | `-c` | 配置文件路径 |
| `--ddl <FILE>` | | DDL文件路径 |
| `--snapshot <FILE>` | | 快照文件路径 |
| `--total_memory <BYTES>` | | 数据库总内存大小（字节） |
| `--default_max_records <N>` | | 默认最大记录数 |
| `--low_power_mode_supported <BOOLEAN>` | | 是否支持低功耗模式 |
| `--low_power_max_records <N>` | | 低功耗模式下的最大记录数 |
| `--non_interactive` | | 非交互式模式（初始化后退出） |
| `--help` | `-h` | 显示帮助信息 |
| `--version` | `-V` | 显示版本信息 |

## 配置文件

remdb-server支持使用TOML格式的配置文件。示例配置文件（`remdb.toml.example`）：

```toml
# DDL文件路径
ddl = "schema.ddl"

# 快照文件路径
snapshot = "snapshot.dat"

# 数据库总内存大小（字节）
total_memory = 104857600  # 100MB

# 默认最大记录数
default_max_records = 1

# 是否支持低功耗模式
low_power_mode_supported = true

# 低功耗模式下的最大记录数
low_power_max_records = 100
```

## DDL语法

remdb-server支持基本的DDL语法，用于定义表结构：

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    age INTEGER
);

CREATE INDEX idx_users_email ON users (email);
```

## 交互式命令行

启动服务器后，会进入交互式命令行界面，可以执行以下命令：

- SQL查询：直接输入SQL语句执行
- `tables`：查看所有表
- `exit`或`quit`：退出服务器

## 架构设计

remdb-server采用模块化设计，主要组件包括：

1. **主程序**（main.rs）：负责初始化、配置加载和启动服务器
2. **DDL编译器**（ddl_compiler.rs）：解析DDL文件，生成表定义
3. **快照加载器**（snapshot_loader.rs）：处理数据库快照的加载和保存
4. **SQL引擎**（sql_engine.rs）：执行SQL查询和命令
5. **命令行界面**（cli.rs）：提供交互式命令行体验

## 项目结构

```
remdb-server/
├── src/
│   ├── main.rs          # 主入口文件
│   ├── ddl_compiler.rs  # DDL文件编译器
│   ├── snapshot_loader.rs # 快照加载器
│   ├── sql_engine.rs    # SQL引擎
│   └── cli.rs           # 命令行界面
├── tests/               # 测试目录
├── Cargo.toml           # 项目配置文件
├── SPEC.md              # 规格说明文件
├── remdb.toml.example   # 配置文件示例
└── schema.ddl           # DDL示例文件
```

## 依赖库

| 依赖 | 版本 | 用途 |
|------|------|------|
| remdb | 0.1.15 | 核心数据库库 |
| remdb-macros | 0.1.15 | 数据库宏定义 |
| rustyline | 12.0.0 | 交互式命令行 |
| rustyline-derive | 0.11.0 | rustyline派生宏 |
| clap | 4.5.0 | 命令行参数解析 |
| thiserror | 1.0.58 | 错误处理 |
| toml | 0.8.10 | TOML配置文件解析 |
| serde | 1.0.197 | 序列化和反序列化 |

## 开发指南

### 代码风格

项目使用Rust的标准代码风格，建议使用以下工具进行代码检查：

```bash
# 运行rustfmt格式化代码
cargo fmt

# 运行clippy检查代码质量
cargo clippy
```

### 调试模式

使用调试模式构建可以获得更详细的日志信息：

```bash
cargo build
```

### 发布模式

发布模式构建会进行优化，获得更好的性能：

```bash
cargo build --release
```

## 许可证

[MIT License](LICENSE)

## 贡献指南

欢迎提交Issue和Pull Request！

### 提交Pull Request的步骤

1. Fork仓库
2. 创建特性分支（`git checkout -b feature/AmazingFeature`）
3. 提交更改（`git commit -m 'Add some AmazingFeature'`）
4. 推送到分支（`git push origin feature/AmazingFeature`）
5. 打开Pull Request

## 联系方式

如有问题或建议，请通过以下方式联系：

- 提交Issue：[GitHub Issues](<repository-issues-url>)

## 版本历史

### v0.1.0（初始版本）

- ✅ 基本数据库服务器功能
- ✅ DDL文件支持
- ✅ 快照加载和保存
- ✅ 交互式命令行界面
- ✅ SQL查询支持
- ✅ 配置文件支持

---

**remdb-server** - 轻量级、高性能的内存数据库服务器