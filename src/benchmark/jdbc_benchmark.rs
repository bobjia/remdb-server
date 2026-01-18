use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::task;

/// 基准测试类型
#[derive(Clone)]
enum BenchmarkType {
    Query,
    Write,
    Mix,
}

/// 基准测试结果
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub total_queries: usize,
    pub total_time_secs: f64,
    pub avg_latency_ns: u128,
    pub p95_latency_ns: u128,
    pub p99_latency_ns: u128,
    pub throughput_qps: f64,
    pub successful_queries: usize,
    pub failed_queries: usize,
    pub min_latency_ns: u128,
    pub max_latency_ns: u128,
    pub test_type: String,
}

/// JDBC基准测试
pub struct JdbcBenchmark {
    server_url: String,
    connection_count: usize,
    query_count: usize,
    query_template: String,
    test_type: BenchmarkType,
    write_template: String,
    read_write_ratio: (usize, usize), // (read, write) ratio, e.g., (8, 2) for 80% read, 20% write
}

impl JdbcBenchmark {
    /// 创建新的JDBC基准测试
    pub fn new(server_url: String, connection_count: usize, query_count: usize) -> Self {
        Self {
            server_url,
            connection_count,
            query_count,
            query_template: "SELECT * FROM test_table WHERE id = {}".to_string(),
            test_type: BenchmarkType::Query,
            write_template: "INSERT INTO test_table (id, value) VALUES ({}, {}) ON DUPLICATE KEY UPDATE value = {}".to_string(),
            read_write_ratio: (8, 2), // 默认8:2读写比
        }
    }

    /// 设置查询模板
    pub fn set_query_template(&mut self, template: String) {
        self.query_template = template;
    }

    /// 设置为查询测试
    pub fn set_query_test(&mut self) {
        self.test_type = BenchmarkType::Query;
    }

    /// 设置为写入测试
    pub fn set_write_test(&mut self) {
        self.test_type = BenchmarkType::Write;
    }

    /// 设置为混合测试
    pub fn set_mix_test(&mut self) {
        self.test_type = BenchmarkType::Mix;
    }

    /// 设置写入模板
    pub fn set_write_template(&mut self, template: String) {
        self.write_template = template;
    }

    /// 设置读写比例
    pub fn set_read_write_ratio(&mut self, read_ratio: usize, write_ratio: usize) {
        self.read_write_ratio = (read_ratio, write_ratio);
    }

    /// 运行基准测试
    pub async fn run(&self) -> BenchmarkResult {
        let start_time = Instant::now();

        // 原子计数器，用于跟踪成功和失败的查询
        let successful_queries = Arc::new(AtomicU64::new(0));
        let failed_queries = Arc::new(AtomicU64::new(0));

        // 用于收集延迟数据
        let latencies = Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(
            self.query_count,
        )));

        // 创建连接池（模拟，实际应该使用真实的连接池）
        // 注意：这里只是模拟，实际实现需要连接到真实的JDBC服务器

        // 并行执行测试
        let mut tasks = Vec::with_capacity(self.query_count);
        for _ in 0..self.query_count {
            let successful = successful_queries.clone();
            let failed = failed_queries.clone();
            let latencies = latencies.clone();

            // 在spawn之前生成所有需要的随机数
            let mut rng = rand::thread_rng();
            let delay_nanos = rng.gen_range(1000..10000);
            let should_fail = rng.gen_bool(0.01);

            // 根据测试类型生成不同的测试数据
            let test_type = self.test_type.clone();
            let (read_ratio, write_ratio) = self.read_write_ratio;
            let total_ratio = read_ratio + write_ratio;

            tasks.push(task::spawn(async move {
                let query_start = Instant::now();

                // 决定当前任务的实际操作类型
                let actual_op_type = match test_type {
                    BenchmarkType::Query => BenchmarkType::Query,
                    BenchmarkType::Write => BenchmarkType::Write,
                    BenchmarkType::Mix => {
                        // 根据读写比例随机决定操作类型
                        let mut rng = rand::thread_rng();
                        let rand_val = rng.gen_range(0..total_ratio);
                        if rand_val < read_ratio {
                            BenchmarkType::Query
                        } else {
                            BenchmarkType::Write
                        }
                    }
                };

                // 模拟测试执行
                // 在实际实现中，这里应该是真实的JDBC操作
                match actual_op_type {
                    BenchmarkType::Query => {
                        // 模拟查询执行
                        // 例如：
                        // let conn = pool.get_connection().await;
                        // let stmt = conn.create_statement().await;
                        // let result = stmt.execute_query(&query).await;
                    }
                    BenchmarkType::Write => {
                        // 模拟写入执行
                        // 例如：
                        // let conn = pool.get_connection().await;
                        // let stmt = conn.create_statement().await;
                        // let result = stmt.execute_update(&query).await;
                    }
                    BenchmarkType::Mix => {
                        // 混合模式下不会直接进入这里，因为已经在上面转换为具体操作类型
                        unreachable!()
                    }
                }

                // 模拟操作延迟（1-10微秒）
                tokio::time::sleep(tokio::time::Duration::from_nanos(delay_nanos)).await;

                let query_duration = query_start.elapsed();
                let latency_ns = query_duration.as_nanos();

                // 记录延迟
                latencies.lock().await.push(latency_ns);

                // 随机模拟失败
                if should_fail {
                    // 1%失败率
                    failed.fetch_add(1, Ordering::SeqCst);
                } else {
                    successful.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        // 等待所有任务完成
        for task in tasks {
            task.await.unwrap();
        }

        let total_time = start_time.elapsed();
        let total_time_secs = total_time.as_secs_f64();

        // 计算结果
        let mut latencies = latencies.lock().await;
        let total_queries = latencies.len();
        let successful = successful_queries.load(Ordering::SeqCst) as usize;
        let failed = failed_queries.load(Ordering::SeqCst) as usize;

        // 计算统计信息
        let (avg_latency, min_latency, max_latency, p95_latency, p99_latency) =
            self.calculate_statistics(&mut latencies);

        // 计算吞吐量
        let throughput_qps = total_queries as f64 / total_time_secs;

        // 确定测试类型字符串
        let test_type_str = match self.test_type {
            BenchmarkType::Query => "query",
            BenchmarkType::Write => "write",
            BenchmarkType::Mix => "mix",
        };

        BenchmarkResult {
            total_queries,
            total_time_secs,
            avg_latency_ns: avg_latency,
            p95_latency_ns: p95_latency,
            p99_latency_ns: p99_latency,
            throughput_qps,
            successful_queries: successful,
            failed_queries: failed,
            min_latency_ns: min_latency,
            max_latency_ns: max_latency,
            test_type: test_type_str.to_string(),
        }
    }

    /// 计算延迟统计信息
    fn calculate_statistics(&self, latencies: &mut Vec<u128>) -> (u128, u128, u128, u128, u128) {
        if latencies.is_empty() {
            return (0, 0, 0, 0, 0);
        }

        // 排序延迟数据
        latencies.sort();

        // 计算最小值和最大值
        let min_latency = latencies[0];
        let max_latency = latencies[latencies.len() - 1];

        // 计算平均值
        let sum: u128 = latencies.iter().sum();
        let avg_latency = sum / latencies.len() as u128;

        // 计算百分位数
        let p95_index = (latencies.len() as f64 * 0.95).floor() as usize;
        let p99_index = (latencies.len() as f64 * 0.99).floor() as usize;

        let p95_latency = latencies[p95_index];
        let p99_latency = latencies[p99_index];

        (
            avg_latency,
            min_latency,
            max_latency,
            p95_latency,
            p99_latency,
        )
    }

    /// 生成HTML报告
    pub fn generate_html_report(
        &self,
        result: &BenchmarkResult,
        output_path: &str,
    ) -> std::io::Result<()> {
        // 先格式化浮点数
        let total_time_str = format!("{:.2}", result.total_time_secs);
        let throughput_str = format!("{:.2}", result.throughput_qps);
        let success_rate_str = format!(
            "{:.2}",
            (result.successful_queries as f64 / result.total_queries as f64) * 100.0
        );

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>JDBC Benchmark Report</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            text-align: center;
        }}
        .stats {{ 
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 30px 0;
        }}
        .stat-card {{
            background-color: #f0f0f0;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
        }}
        .stat-label {{
            font-size: 14px;
            color: #666;
            margin-bottom: 5px;
        }}
        .stat-value {{
            font-size: 24px;
            font-weight: bold;
            color: #333;
        }}
        .chart-container {{
            margin: 30px 0;
            height: 400px;
        }}
        .success {{ color: #4CAF50; }}
        .error {{ color: #f44336; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }}
        th {{
            background-color: #f2f2f2;
        }}
    </style>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <div class="container">
        <h1>JDBC Benchmark Report</h1>
        
        <div class="stats">
            <div class="stat-card">
                <div class="stat-label">Total Queries</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Time (sec)</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Throughput (QPS)</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Average Latency (ns)</div>
                <div class="stat-value">{}</div>
            </div>
        </div>
        
        <div class="stats">
            <div class="stat-card">
                <div class="stat-label">P95 Latency (ns)</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">P99 Latency (ns)</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card success">
                <div class="stat-label">Successful Queries</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card error">
                <div class="stat-label">Failed Queries</div>
                <div class="stat-value">{}</div>
            </div>
        </div>
        
        <div class="chart-container">
            <canvas id="latencyChart"></canvas>
        </div>
        
        <h2>Detailed Results</h2>
        <table>
            <tr>
                <th>Metric</th>
                <th>Value</th>
            </tr>
            <tr>
                <td>Minimum Latency (ns)</td>
                <td>{}</td>
            </tr>
            <tr>
                <td>Maximum Latency (ns)</td>
                <td>{}</td>
            </tr>
            <tr>
                <td>Success Rate</td>
                <td>{}%</td>
            </tr>
        </table>
    </div>
    
    <script>
        const ctx = document.getElementById('latencyChart').getContext('2d');
        const chart = new Chart(ctx, {{
            type: 'bar',
            data: {{
                labels: ['Min', 'Avg', 'P95', 'P99', 'Max'],
                datasets: [{{
                    label: 'Latency (ns)',
                    data: [{}, {}, {}, {}, {}],
                    backgroundColor: [
                        'rgba(75, 192, 192, 0.6)',
                        'rgba(54, 162, 235, 0.6)',
                        'rgba(255, 206, 86, 0.6)',
                        'rgba(255, 99, 132, 0.6)',
                        'rgba(153, 102, 255, 0.6)'
                    ],
                    borderColor: [
                        'rgba(75, 192, 192, 1)',
                        'rgba(54, 162, 235, 1)',
                        'rgba(255, 206, 86, 1)',
                        'rgba(255, 99, 132, 1)',
                        'rgba(153, 102, 255, 1)'
                    ],
                    borderWidth: 1
                }}]
            }},
            options: {{
                scales: {{
                    y: {{
                        beginAtZero: true,
                        title: {{
                            display: true,
                            text: 'Latency (ns)'
                        }}
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>"#,
            result.total_queries,
            total_time_str,
            throughput_str,
            result.avg_latency_ns,
            result.p95_latency_ns,
            result.p99_latency_ns,
            result.successful_queries,
            result.failed_queries,
            result.min_latency_ns,
            result.max_latency_ns,
            success_rate_str,
            result.min_latency_ns,
            result.avg_latency_ns,
            result.p95_latency_ns,
            result.p99_latency_ns,
            result.max_latency_ns
        );

        let mut file = File::create(output_path)?;
        file.write_all(html.as_bytes())?;

        Ok(())
    }

    /// 生成JSON报告
    pub fn generate_json_report(
        &self,
        result: &BenchmarkResult,
        output_path: &str,
    ) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(result)?;
        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}

/// 基准测试配置
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub server_url: String,
    pub connection_count: usize,
    pub query_count: usize,
    pub query_template: String,
    pub run_duration_secs: Option<u64>,
    pub test_type: String,
    pub write_template: String,
    pub read_write_ratio: (usize, usize),
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            server_url: "jdbc:remdb://localhost:6666".to_string(),
            connection_count: 16,
            query_count: 100000,
            query_template: "SELECT * FROM test_table WHERE id = {}".to_string(),
            run_duration_secs: None,
            test_type: "query".to_string(),
            write_template: "INSERT INTO test_table (id, value) VALUES ({}, {}) ON DUPLICATE KEY UPDATE value = {}".to_string(),
            read_write_ratio: (8, 2),
        }
    }
}

/// 运行基准测试的命令行工具
pub async fn run_benchmark(config: BenchmarkConfig) -> std::io::Result<BenchmarkResult> {
    println!("Starting JDBC benchmark with configuration:");
    println!("  Server URL: {}", config.server_url);
    println!("  Connection Count: {}", config.connection_count);
    println!("  Query Count: {}", config.query_count);
    println!("  Test Type: {}", config.test_type);
    println!("  Query Template: {}", config.query_template);
    println!("  Write Template: {}", config.write_template);
    println!(
        "  Read-Write Ratio: {}:{}",
        config.read_write_ratio.0, config.read_write_ratio.1
    );
    println!("  Run Duration: {:?}", config.run_duration_secs);
    println!();

    let mut benchmark = JdbcBenchmark::new(
        config.server_url,
        config.connection_count,
        config.query_count,
    );
    benchmark.set_query_template(config.query_template);
    benchmark.set_write_template(config.write_template);
    benchmark.set_read_write_ratio(config.read_write_ratio.0, config.read_write_ratio.1);

    // 设置测试类型
    match config.test_type.to_lowercase().as_str() {
        "write" => {
            benchmark.set_write_test();
            println!("Running write benchmark...");
        }
        "mix" => {
            benchmark.set_mix_test();
            println!(
                "Running mixed read-write benchmark with ratio {}:{}",
                config.read_write_ratio.0, config.read_write_ratio.1
            );
        }
        _ => {
            benchmark.set_query_test();
            println!("Running query benchmark...");
        }
    }

    let result = benchmark.run().await;

    println!("\nBenchmark Results:");
    println!("================");
    println!("Total Queries: {}", result.total_queries);
    println!("Total Time: {:.2} seconds", result.total_time_secs);
    println!("Throughput: {:.2} QPS", result.throughput_qps);
    println!("Average Latency: {} ns", result.avg_latency_ns);
    println!("P95 Latency: {} ns", result.p95_latency_ns);
    println!("P99 Latency: {} ns", result.p99_latency_ns);
    println!("Min Latency: {} ns", result.min_latency_ns);
    println!("Max Latency: {} ns", result.max_latency_ns);
    println!(
        "Successful Queries: {} ({:.2}%)",
        result.successful_queries,
        (result.successful_queries as f64 / result.total_queries as f64) * 100.0
    );
    println!(
        "Failed Queries: {} ({:.2}%)",
        result.failed_queries,
        (result.failed_queries as f64 / result.total_queries as f64) * 100.0
    );

    // 生成HTML报告
    let html_path = "benchmark_report.html";
    benchmark.generate_html_report(&result, html_path)?;
    println!("\nHTML report generated: {}", html_path);

    // 生成JSON报告
    let json_path = "benchmark_results.json";
    benchmark.generate_json_report(&result, json_path)?;
    println!("JSON report generated: {}", json_path);

    Ok(result)
}
