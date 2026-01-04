use remdb::{RemDb, Result};

fn main() -> Result<()> {
    // 创建内存数据库
    let mut db = RemDb::new_in_memory()?;
    
    println!("Testing CREATE TIMESERIES TABLE with WITH COMPRESSION...");
    
    // 测试1：创建带有WITH COMPRESSION子句的时序表
    let sql1 = "CREATE TIMESERIES TABLE test_ts (
        ts TIMESTAMP,
        value FLOAT64,
        tag1 VARCHAR(20),
        tag2 INT
    ) WITH COMPRESSION = (algorithm='delta-delta', enabled=true)";
    
    match db.execute_sql(sql1) {
        Ok(result) => {
            println!("✓ Test 1 passed: Created timeseries table with delta-delta compression");
        },
        Err(err) => {
            println!("✗ Test 1 failed: {}", err);
            return Err(err);
        }
    }
    
    println!("\nTesting CREATE TIMESERIES TABLE with WITH TTL...");
    
    // 测试2：创建带有WITH TTL子句的时序表
    let sql2 = "CREATE TIMESERIES TABLE test_ts_ttl (
        ts TIMESTAMP,
        value FLOAT64,
        tag1 VARCHAR(20)
    ) WITH TTL = '30 days'";
    
    match db.execute_sql(sql2) {
        Ok(result) => {
            println!("✓ Test 2 passed: Created timeseries table with TTL=30 days");
        },
        Err(err) => {
            println!("✗ Test 2 failed: {}", err);
            return Err(err);
        }
    }
    
    println!("\nTesting CREATE TIMESERIES TABLE with both WITH COMPRESSION and WITH TTL...");
    
    // 测试3：创建同时带有WITH COMPRESSION和WITH TTL子句的时序表
    let sql3 = "CREATE TIMESERIES TABLE test_ts_both (
        ts TIMESTAMP,
        value FLOAT64,
        tag1 VARCHAR(20),
        tag2 INT,
        tag3 FLOAT
    ) WITH COMPRESSION = (algorithm='delta-delta', enabled=true), WITH TTL = '7 days'";
    
    match db.execute_sql(sql3) {
        Ok(result) => {
            println!("✓ Test 3 passed: Created timeseries table with both delta-delta compression and TTL=7 days");
        },
        Err(err) => {
            println!("✗ Test 3 failed: {}", err);
            return Err(err);
        }
    }
    
    println!("\nAll tests passed! ✓");
    
    Ok(())
}
