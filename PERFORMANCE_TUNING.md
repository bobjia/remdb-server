# 高性能JDBC服务器性能调优指南

## 1. 系统调优

### 1.1 Linux内核参数调优

将以下配置添加到 `/etc/sysctl.conf` 文件中，然后运行 `sysctl -p` 生效：

```bash
# 调整TCP参数
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 87380 134217728
net.ipv4.tcp_wmem = 4096 65536 134217728

# 启用TCP快速打开
net.ipv4.tcp_fastopen = 3

# 增加TCP最大连接数
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535

# 减少TCP超时时间
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 3
net.ipv4.tcp_keepalive_intvl = 15

# 禁用透明大页（对内存数据库更好）
vm.nr_hugepages = 0

# 增加文件描述符限制
fs.file-max = 1000000

# 优化内存管理
vm.swappiness = 1
vm.dirty_background_ratio = 5
vm.dirty_ratio = 10
```

### 1.2 资源限制调优

修改 `/etc/security/limits.conf` 文件，增加资源限制：

```bash
* soft nofile 1000000
* hard nofile 1000000
* soft nproc 1000000
* hard nproc 1000000
```

### 1.3 CPU和内存调优

- **CPU亲和性**：使用 `taskset` 命令将进程绑定到特定CPU核心，减少上下文切换
- **NUMA优化**：确保进程和内存分配在同一NUMA节点
- **大页内存**：对于内存密集型工作负载，考虑使用大页内存

## 2. 应用程序调优

### 2.1 启动参数调优

使用提供的启动脚本 `start-highperf-jdbc-server.sh`，可以调整以下参数：

| 参数 | 描述 | 建议值 |
|------|------|--------|
| `--port` | JDBC服务器端口 | 默认：6666 |
| `--admin-port` | 管理API端口 | 默认：9090 |
| `--threads` | 工作线程数 | 建议：CPU核心数 |
| `--connections` | 最大连接数 | 建议：10000-50000 |
| `--memory` | 内存限制 | 建议：系统内存的70-80% |
| `--auth-enabled` | 启用认证 | 根据需求选择 |
| `--log-level` | 日志级别 | 生产环境：info，调试：debug |

### 2.2 环境变量调优

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

## 3. 性能监控

### 3.1 内置监控

高性能JDBC服务器提供了内置的Prometheus监控指标，访问 `http://localhost:9090/metrics` 可以获取以下指标：

- 请求处理延迟分布
- 每秒查询数(QPS)
- 活跃连接数
- 内存使用情况
- CPU使用率
- 工作线程状态

### 3.2 推荐监控工具

- **Prometheus + Grafana**：用于长期监控和可视化
- **htop**：实时CPU和内存监控
- **iostat**：磁盘I/O监控
- **netstat/ss**：网络连接监控
- **tcpdump**：网络流量分析

## 4. 客户端调优

### 4.1 JDBC驱动调优

- 使用直接内存缓冲池（已在优化后的JDBC驱动中实现）
- 调整连接池大小，建议：CPU核心数的1-2倍
- 启用TCP_NODELAY
- 调整fetchSize参数，建议：100-1000

### 4.2 应用程序优化

- 使用批量操作减少网络往返
- 避免在循环中执行SQL查询
- 使用预编译语句
- 合理设置事务边界
- 减少大结果集查询

## 5. 性能测试

### 5.1 使用内置基准测试工具

```bash
# 运行基准测试
cargo run --release --bin remdb-server -- benchmark --query-count 100000 --connections 16
```

### 5.2 测试场景

- **高并发测试**：测试最大连接数下的性能
- **低延迟测试**：测试单个查询的延迟
- **混合负载测试**：同时测试读和写操作
- **长期稳定性测试**：连续运行数小时或数天

### 5.3 性能目标

| 指标 | 目标值 |
|------|--------|
| 吞吐量 | 100,000+ QPS |
| 平均延迟 | < 100微秒 |
| P95延迟 | < 500微秒 |
| P99延迟 | < 1毫秒 |
| 最大连接数 | 10,000+ |

## 6. 常见性能问题排查

### 6.1 高CPU使用率

- 检查是否有慢查询
- 检查工作线程数是否合适
- 使用火焰图分析热点函数

### 6.2 高内存使用率

- 检查是否有内存泄漏
- 调整缓冲区池大小
- 检查是否有大结果集查询

### 6.3 高延迟

- 检查网络连接是否正常
- 检查服务器负载
- 检查查询复杂度

### 6.4 连接拒绝

- 检查最大连接数设置
- 检查文件描述符限制
- 检查TCP backlog设置

## 7. 最佳实践

1. **定期监控**：设置监控告警，及时发现性能问题
2. **逐步调优**：一次只修改一个参数，观察效果
3. **基准测试**：在生产环境部署前进行充分的基准测试
4. **持续优化**：根据实际工作负载调整配置
5. **合理规划**：根据业务需求规划资源配置

## 8. 升级建议

- 定期更新依赖库
- 关注Rust版本更新，利用新的性能特性
- 定期进行性能测试，确保性能不退化
- 根据业务增长调整资源配置

## 9. 故障恢复

- 确保有完善的监控告警机制
- 定期备份数据
- 准备应急预案
- 定期进行故障演练

---

通过以上调优建议，可以充分发挥高性能JDBC服务器的潜力，实现百万级QPS处理能力和微秒级响应延迟。
