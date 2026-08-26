//! 向量操作性能基准测试
//!
//! 本模块对 SQL 向量操作（L2距离、内积、余弦相似度、表达式解析）进行性能测试。
//! 测试不同维度、不同批量下的吞吐量和延迟。

use std::time::Instant;

use remdb::sql::operations::vector::{
    calculate_vector_cosine_similarity, calculate_vector_inner_product,
    calculate_vector_l2_distance, parse_vector_distance_expression,
};

/// 性能测试结果
#[derive(Debug, Clone)]
pub struct VectorOpBenchResult {
    /// 操作名称
    pub op_name: &'static str,
    /// 向量维度
    pub dimension: u16,
    /// 测试次数
    pub iterations: usize,
    /// 总耗时(秒)
    pub total_time_secs: f64,
    /// 平均延迟(纳秒)
    pub avg_latency_ns: f64,
    /// 吞吐量(ops/sec)
    pub throughput: f64,
    /// 总耗时(纳秒)
    pub total_time_ns: u128,
    /// 最小延迟(纳秒)
    pub min_latency_ns: u128,
    /// 最大延迟(纳秒)
    pub max_latency_ns: u128,
}

impl VectorOpBenchResult {
    fn new(op_name: &'static str, dimension: u16, iterations: usize) -> Self {
        Self {
            op_name,
            dimension,
            iterations,
            total_time_secs: 0.0,
            avg_latency_ns: 0.0,
            throughput: 0.0,
            total_time_ns: 0,
            min_latency_ns: u128::MAX,
            max_latency_ns: 0,
        }
    }

    fn record(&mut self, latency_ns: u128) {
        self.total_time_ns += latency_ns;
        if latency_ns < self.min_latency_ns {
            self.min_latency_ns = latency_ns;
        }
        if latency_ns > self.max_latency_ns {
            self.max_latency_ns = latency_ns;
        }
    }

    fn finalize(&mut self) {
        self.total_time_secs = self.total_time_ns as f64 / 1_000_000_000.0;
        self.avg_latency_ns = self.total_time_ns as f64 / self.iterations as f64;
        self.throughput = if self.total_time_secs > 0.0 {
            self.iterations as f64 / self.total_time_secs
        } else {
            0.0
        };
    }
}

/// 向量操作性能测试套件
pub struct VectorOpsBenchmark {
    /// 测试维度列表
    dimensions: Vec<u16>,
    /// 每项测试迭代次数
    iterations: usize,
    /// 预热迭代次数
    warmup_iterations: usize,
}

impl VectorOpsBenchmark {
    /// 创建新的向量操作性能测试
    pub fn new() -> Self {
        Self {
            dimensions: vec![128, 256, 512, 1024, 2048],
            iterations: 100_000,
            warmup_iterations: 10_000,
        }
    }

    /// 设置测试维度
    pub fn set_dimensions(&mut self, dimensions: Vec<u16>) {
        self.dimensions = dimensions;
    }

    /// 设置迭代次数
    pub fn set_iterations(&mut self, iterations: usize) {
        self.iterations = iterations;
    }

    /// 设置预热次数
    pub fn set_warmup_iterations(&mut self, warmup: usize) {
        self.warmup_iterations = warmup;
    }

    /// 运行所有性能测试
    pub fn run_all(&self) -> Vec<VectorOpBenchResult> {
        let mut results = Vec::new();

        for &dim in &self.dimensions {
            // 为当前维度生成测试向量
            let vec1 = self.generate_test_vec_f32(dim);
            let vec2 = self.generate_test_vec_f64(dim);

            // L2距离测试
            results.push(self.bench_l2_distance(dim, &vec1, &vec2));
            // 内积测试
            results.push(self.bench_inner_product(dim, &vec1, &vec2));
            // 余弦相似度测试
            results.push(self.bench_cosine_similarity(dim, &vec1, &vec2));
        }

        // 表达式解析测试（不需要维度参数）
        results.push(self.bench_expression_parsing());

        results
    }

    /// 生成测试向量 (f32)
    fn generate_test_vec_f32(&self, dimension: u16) -> Vec<f32> {
        (0..dimension)
            .map(|i| {
                let i = i as f32;
                // 使用确定性数据，避免随机数生成的开销影响测试
                ((i * 1.1).sin() * 100.0).round() / 100.0
            })
            .collect()
    }

    /// 生成测试向量 (f64)
    fn generate_test_vec_f64(&self, dimension: u16) -> Vec<f64> {
        (0..dimension)
            .map(|i| {
                let i = i as f64;
                // 使用确定性数据，避免随机数生成的开销影响测试
                ((i * 1.1).sin() * 100.0).round() / 100.0
            })
            .collect()
    }

    /// 预热：执行多次迭代使CPU缓存和分支预测器稳定
    fn warmup<F>(&self, mut f: F)
    where
        F: FnMut(),
    {
        for _ in 0..self.warmup_iterations {
            f();
        }
    }

    /// 基准测试 L2 距离
    fn bench_l2_distance(
        &self,
        dimension: u16,
        vec1: &[f32],
        vec2: &[f64],
    ) -> VectorOpBenchResult {
        let mut result = VectorOpBenchResult::new("L2 Distance", dimension, self.iterations);

        // 预热
        self.warmup(|| {
            let _ = unsafe {
                calculate_vector_l2_distance(vec1.as_ptr(), vec2, dimension)
            };
        });

        // 测试
        for _ in 0..self.iterations {
            let start = Instant::now();
            let _dist = unsafe {
                calculate_vector_l2_distance(vec1.as_ptr(), vec2, dimension)
            };
            let elapsed = start.elapsed().as_nanos();
            result.record(elapsed);
        }

        result.finalize();
        result
    }

    /// 基准测试内积
    fn bench_inner_product(
        &self,
        dimension: u16,
        vec1: &[f32],
        vec2: &[f64],
    ) -> VectorOpBenchResult {
        let mut result = VectorOpBenchResult::new("Inner Product", dimension, self.iterations);

        // 预热
        self.warmup(|| {
            let _ = unsafe {
                calculate_vector_inner_product(vec1.as_ptr(), vec2, dimension)
            };
        });

        // 测试
        for _ in 0..self.iterations {
            let start = Instant::now();
            let _prod = unsafe {
                calculate_vector_inner_product(vec1.as_ptr(), vec2, dimension)
            };
            let elapsed = start.elapsed().as_nanos();
            result.record(elapsed);
        }

        result.finalize();
        result
    }

    /// 基准测试余弦相似度
    fn bench_cosine_similarity(
        &self,
        dimension: u16,
        vec1: &[f32],
        vec2: &[f64],
    ) -> VectorOpBenchResult {
        let mut result = VectorOpBenchResult::new("Cosine Similarity", dimension, self.iterations);

        // 预热
        self.warmup(|| {
            let _ = unsafe {
                calculate_vector_cosine_similarity(vec1.as_ptr(), vec2, dimension)
            };
        });

        // 测试
        for _ in 0..self.iterations {
            let start = Instant::now();
            let _sim = unsafe {
                calculate_vector_cosine_similarity(vec1.as_ptr(), vec2, dimension)
            };
            let elapsed = start.elapsed().as_nanos();
            result.record(elapsed);
        }

        result.finalize();
        result
    }

    /// 基准测试表达式解析
    fn bench_expression_parsing(&self) -> VectorOpBenchResult {
        let mut result = VectorOpBenchResult::new("Expression Parsing", 0, self.iterations);

        // 测试用例
        let test_cases = vec![
            "vector_field <-> [1.0, 2.0, 3.0, 4.0, 5.0]",
            "embedding <#> [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]",
            "vec_col <=> [10.5, 20.3, 30.7, 40.1, 50.9]",
            "long_field_name_123 <-> [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]",
            "a <=> [0.0]",
        ];

        // 预热
        let cases = test_cases.clone();
        self.warmup(move || {
            for case in &cases {
                let _ = parse_vector_distance_expression(case);
            }
        });

        // 测试
        for _ in 0..self.iterations {
            let start = Instant::now();
            for case in &test_cases {
                let _parsed = parse_vector_distance_expression(case);
            }
            let elapsed = start.elapsed().as_nanos();
            result.record(elapsed);
        }

        result.finalize();
        // 每次迭代处理5个表达式
        let total_ops = self.iterations * test_cases.len();
        result.throughput = total_ops as f64 / result.total_time_secs;
        result
    }

    /// 打印结果表格
    pub fn print_results(&self, results: &[VectorOpBenchResult]) {
        println!("\n═══════════════════════════════════════════════════════════════════");
        println!("          向量操作性能基准测试报告");
        println!("═══════════════════════════════════════════════════════════════════");
        println!(
            " 迭代次数: {} (预热: {})",
            self.iterations, self.warmup_iterations
        );
        println!("─────────────────────────────────────────────────────────────────────");
        println!(
            " {:<20} {:<8} {:<14} {:<14} {:<14}",
            "操作", "维度", "平均延迟(ns)", "吞吐量(ops/s)", "总时间(ms)"
        );
        println!("─────────────────────────────────────────────────────────────────────");

        for r in results {
            let total_ms = r.total_time_ns as f64 / 1_000_000.0;
            println!(
                " {:<20} {:<8} {:>8.2} ns  {:>12.2}  {:>10.2}",
                r.op_name,
                if r.dimension > 0 {
                    format!("{}", r.dimension)
                } else {
                    "N/A".to_string()
                },
                r.avg_latency_ns,
                r.throughput,
                total_ms,
            );
        }

        println!("─────────────────────────────────────────────────────────────────────");
        println!();

        // 打印详细延迟信息
        println!("═══════════════════════════════════════════════════════════════════");
        println!("          详细延迟统计 (纳秒)");
        println!("═══════════════════════════════════════════════════════════════════");
        println!(
            " {:<20} {:<8} {:<12} {:<12} {:<12}",
            "操作", "维度", "最小", "平均", "最大"
        );
        println!("─────────────────────────────────────────────────────────────────────");

        for r in results {
            println!(
                " {:<20} {:<8} {:>8}   {:>8.1}   {:>8}",
                r.op_name,
                if r.dimension > 0 {
                    format!("{}", r.dimension)
                } else {
                    "N/A".to_string()
                },
                r.min_latency_ns,
                r.avg_latency_ns,
                r.max_latency_ns,
            );
        }

        println!("─────────────────────────────────────────────────────────────────────");
        println!();
    }

    /// 生成 Markdown 格式报告
    pub fn generate_markdown_report(&self, results: &[VectorOpBenchResult]) -> String {
        let mut md = String::new();

        md.push_str("# 向量操作性能基准测试报告\n\n");
        md.push_str(&format!(
            "- **迭代次数**: {} (预热: {})\n",
            self.iterations, self.warmup_iterations
        ));
        md.push_str(&format!(
            "- **测试维度**: {:?}\n\n",
            self.dimensions
        ));

        // 吞吐量表格
        md.push_str("## 吞吐量 (ops/s)\n\n");
        md.push_str("| 操作 |");
        for &dim in &self.dimensions {
            md.push_str(&format!(" {} |", dim));
        }
        md.push_str(" 表达式解析 |\n");

        md.push_str("|");
        for _ in 0..=self.dimensions.len() {
            md.push_str(" --- |");
        }
        md.push_str(" --- |\n");

        let ops = ["L2 Distance", "Inner Product", "Cosine Similarity"];
        for op_name in &ops {
            md.push_str(&format!("| {} |", op_name));
            for &dim in &self.dimensions {
                if let Some(r) = results.iter().find(|r| r.op_name == *op_name && r.dimension == dim) {
                    md.push_str(&format!(" {:.0} |", r.throughput));
                } else {
                    md.push_str(" N/A |");
                }
            }
            // 表达式解析
            if let Some(r) = results.iter().find(|r| r.op_name == "Expression Parsing") {
                md.push_str(&format!(" {:.0} |", r.throughput));
            }
            md.push('\n');
        }

        // 平均延迟表格
        md.push_str("\n## 平均延迟 (ns)\n\n");
        md.push_str("| 操作 |");
        for &dim in &self.dimensions {
            md.push_str(&format!(" {} |", dim));
        }
        md.push_str(" 表达式解析 |\n");

        md.push_str("|");
        for _ in 0..=self.dimensions.len() {
            md.push_str(" --- |");
        }
        md.push_str(" --- |\n");

        for op_name in &ops {
            md.push_str(&format!("| {} |", op_name));
            for &dim in &self.dimensions {
                if let Some(r) = results.iter().find(|r| r.op_name == *op_name && r.dimension == dim) {
                    md.push_str(&format!(" {:.1} |", r.avg_latency_ns));
                } else {
                    md.push_str(" N/A |");
                }
            }
            if let Some(r) = results.iter().find(|r| r.op_name == "Expression Parsing") {
                md.push_str(&format!(" {:.1} |", r.avg_latency_ns));
            }
            md.push('\n');
        }

        md
    }
}

impl Default for VectorOpsBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：向量操作的功能正确性
    #[test]
    fn test_vector_ops_correctness() {
        let dim = 4u16;
        let vec1 = vec![1.0f32, 2.0, 3.0, 4.0];
        let vec2 = vec![4.0f64, 3.0, 2.0, 1.0];

        // L2距离: sqrt((1-4)^2 + (2-3)^2 + (3-2)^2 + (4-1)^2)
        // = sqrt(9 + 1 + 1 + 9) = sqrt(20) ≈ 4.4721
        let dist = unsafe { calculate_vector_l2_distance(vec1.as_ptr(), &vec2, dim) };
        let expected_dist = (20.0f64).sqrt();
        assert!(
            (dist - expected_dist).abs() < 1e-10,
            "L2 distance mismatch: got {}, expected {}",
            dist,
            expected_dist
        );

        // 内积: 1*4 + 2*3 + 3*2 + 4*1 = 4 + 6 + 6 + 4 = 20
        let prod = unsafe { calculate_vector_inner_product(vec1.as_ptr(), &vec2, dim) };
        assert!(
            (prod - 20.0).abs() < 1e-10,
            "Inner product mismatch: got {}, expected {}",
            prod,
            20.0
        );

        // 余弦相似度: dot/(|v1|*|v2|)
        // |v1| = sqrt(1+4+9+16) = sqrt(30)
        // |v2| = sqrt(16+9+4+1) = sqrt(30)
        // cos = 20/(sqrt(30)*sqrt(30)) = 20/30 = 2/3 ≈ 0.6667
        let cos = unsafe { calculate_vector_cosine_similarity(vec1.as_ptr(), &vec2, dim) };
        let expected_cos = 20.0 / 30.0;
        assert!(
            (cos - expected_cos).abs() < 1e-10,
            "Cosine similarity mismatch: got {}, expected {}",
            cos,
            expected_cos
        );
    }

    /// 测试：表达式解析的正确性
    #[test]
    fn test_expression_parsing_correctness() {
        // 测试 L2 距离解析
        let result = parse_vector_distance_expression("vec <-> [1.0, 2.0, 3.0]");
        assert!(result.is_some());
        let (field, op, values) = result.unwrap();
        assert_eq!(field, "vec");
        assert_eq!(op, "<->");
        assert_eq!(values, vec![1.0, 2.0, 3.0]);

        // 测试内积解析
        let result = parse_vector_distance_expression("embedding <#> [0.5, 0.5]");
        assert!(result.is_some());
        let (field, op, values) = result.unwrap();
        assert_eq!(field, "embedding");
        assert_eq!(op, "<#>");
        assert_eq!(values, vec![0.5, 0.5]);

        // 测试余弦相似度解析
        let result = parse_vector_distance_expression("v <=> [1.0, 2.0]");
        assert!(result.is_some());
        let (field, op, values) = result.unwrap();
        assert_eq!(field, "v");
        assert_eq!(op, "<=>");
        assert_eq!(values, vec![1.0, 2.0]);

        // 测试无效表达式
        let result = parse_vector_distance_expression("invalid");
        assert!(result.is_none());
    }

    /// 测试：向量操作性能基准测试
    /// 注意：这是一个性能测试，通过标准测试框架运行。
    /// 使用 `cargo test -- --nocapture vector_ops_bench` 查看详细输出。
    #[test]
    fn test_vector_ops_bench() {
        let mut bench = VectorOpsBenchmark::new();
        // 使用较小的迭代次数，确保测试快速完成
        bench.set_iterations(10_000);
        bench.set_warmup_iterations(1_000);
        // 使用较小的维度集
        bench.set_dimensions(vec![128, 256, 512]);

        let results = bench.run_all();
        bench.print_results(&results);

        // 生成 Markdown 报告
        let md_report = bench.generate_markdown_report(&results);
        // 写入文件
        let report_path = "vector_ops_benchmark_report.md";
        match std::fs::write(report_path, &md_report) {
            Ok(_) => println!("\nMarkdown 报告已生成: {}", report_path),
            Err(e) => eprintln!("警告: 无法写入报告文件: {}", e),
        }

        // 验证所有操作都有结果
        assert!(!results.is_empty(), "应该产生至少一个测试结果");

        // 验证 L2 距离操作的结果
        let l2_results: Vec<_> = results
            .iter()
            .filter(|r| r.op_name == "L2 Distance")
            .collect();
        assert!(
            !l2_results.is_empty(),
            "应该包含 L2 Distance 测试结果"
        );

        // 验证吞吐量大于0
        for r in &results {
            assert!(
                r.throughput > 0.0,
                "{} (dim={}) 的吞吐量应该大于0",
                r.op_name,
                r.dimension
            );
        }
    }

    /// 测试：大规模向量操作性能测试
    #[test]
    fn test_vector_ops_large_scale() {
        let mut bench = VectorOpsBenchmark::new();
        // 大规模测试：使用完整的维度集和合理的迭代次数
        bench.set_iterations(50_000);
        bench.set_warmup_iterations(5_000);
        bench.set_dimensions(vec![128, 256, 512, 1024, 2048]);

        let results = bench.run_all();

        // 验证高维向量操作的结果
        let high_dim_results: Vec<_> = results
            .iter()
            .filter(|r| r.dimension == 1024 || r.dimension == 2048)
            .collect();
        assert!(
            !high_dim_results.is_empty(),
            "应该包含高维向量测试结果"
        );

        // 验证延迟随维度增加而增加
        for op_name in &["L2 Distance", "Inner Product", "Cosine Similarity"] {
            let dim_results: Vec<_> = results
                .iter()
                .filter(|r| r.op_name == *op_name)
                .collect();

            if dim_results.len() >= 2 {
                // 确保维度排序正确
                for i in 1..dim_results.len() {
                    assert!(
                        dim_results[i].avg_latency_ns >= dim_results[i - 1].avg_latency_ns,
                        "{} 的延迟应随维度增加而增加: dim={} ({}ns) < dim={} ({}ns)",
                        op_name,
                        dim_results[i - 1].dimension,
                        dim_results[i - 1].avg_latency_ns,
                        dim_results[i].dimension,
                        dim_results[i].avg_latency_ns,
                    );
                }
            }
        }
    }

    /// 测试：向量操作与表达式解析的混合性能
    #[test]
    fn test_vector_ops_mixed_workload() {
        let bench = VectorOpsBenchmark::new();
        let dim = 512u16;
        let iterations = 20_000;

        let vec1 = bench.generate_test_vec_f32(dim);
        let vec2 = bench.generate_test_vec_f64(dim);

        let mut l2_result = VectorOpBenchResult::new("Mixed L2", dim, iterations);
        let mut cos_result = VectorOpBenchResult::new("Mixed Cosine", dim, iterations);
        let mut parse_result = VectorOpBenchResult::new("Mixed Parse", 0, iterations);

        let expressions = vec![
            "v1 <-> [1.0, 2.0, 3.0, 4.0]",
            "v2 <#> [0.5, 0.5, 0.5, 0.5]",
            "v3 <=> [0.1, 0.2, 0.3, 0.4]",
        ];

        // 模拟混合工作负载：交替执行不同类型的操作
        for i in 0..iterations {
            let idx = i % 3;

            // 操作1: L2距离
            let start = Instant::now();
            let _ = unsafe { calculate_vector_l2_distance(vec1.as_ptr(), &vec2, dim) };
            l2_result.record(start.elapsed().as_nanos());

            // 操作2: 余弦相似度
            let start = Instant::now();
            let _ = unsafe { calculate_vector_cosine_similarity(vec1.as_ptr(), &vec2, dim) };
            cos_result.record(start.elapsed().as_nanos());

            // 操作3: 表达式解析
            let start = Instant::now();
            let _ = parse_vector_distance_expression(expressions[idx]);
            parse_result.record(start.elapsed().as_nanos());
        }

        l2_result.finalize();
        cos_result.finalize();
        parse_result.finalize();

        println!("\n═══════════════════════════════════════════════════════════════════");
        println!("          混合工作负载性能测试 (dim=512, iter={})", iterations);
        println!("═══════════════════════════════════════════════════════════════════");
        println!(
            " {:<20} {:<12} {:<14}",
            "操作", "平均延迟(ns)", "吞吐量(ops/s)"
        );
        println!("─────────────────────────────────────────────────────────────────────");
        println!(
            " {:<20} {:>8.1} ns  {:>12.0}",
            "L2 Distance", l2_result.avg_latency_ns, l2_result.throughput
        );
        println!(
            " {:<20} {:>8.1} ns  {:>12.0}",
            "Cosine Similarity", cos_result.avg_latency_ns, cos_result.throughput
        );
        println!(
            " {:<20} {:>8.1} ns  {:>12.0}",
            "Expression Parsing", parse_result.avg_latency_ns, parse_result.throughput
        );
        println!("─────────────────────────────────────────────────────────────────────\n");

        // 验证结果
        assert!(l2_result.throughput > 0.0);
        assert!(cos_result.throughput > 0.0);
        assert!(parse_result.throughput > 0.0);
    }
}