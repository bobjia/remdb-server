use remdb::{RemDb, Result};

fn main() -> Result<()> {
    // 创建内存数据库
    let mut db = RemDb::new_in_memory()?;
    
    println!("Creating time series tables...");
    
    // 创建带有不同配置的时序表
    let sql1 = "CREATE TIMESERIES TABLE test_ts1 (
        ts TIMESTAMP,
        value FLOAT64,
        tag1 VARCHAR(20),
        tag2 INT
    ) WITH COMPRESSION = (algorithm='delta-delta', enabled=true), WITH TTL = '30 days'";
    
    let sql2 = "CREATE TIMESERIES TABLE test_ts2 (
        timestamp TIMESTAMP,
        temperature FLOAT64,
        location VARCHAR(50)
    ) WITH COMPRESSION = (algorithm='delta', enabled=true), WITH TTL = '7 days'";
    
    let sql3 = "CREATE TIMESERIES TABLE test_ts3 (
        time TIMESTAMP,
        value DOUBLE,
        device_id VARCHAR(30),
        type VARCHAR(20)
    ) WITH COMPRESSION = (algorithm='runlength', enabled=true), WITH TTL = '1 day'";
    
    // 执行创建表语句
    db.execute_sql(sql1)?;
    db.execute_sql(sql2)?;
    db.execute_sql(sql3)?;
    
    println!("\nExporting DDL to 'test_ddl_export.sql'...");
    
    // 导出DDL
    db.export_ddl("test_ddl_export.sql")?;
    
    println!("\nDDL export completed. Let's check the content:");
    println!("=============================================");
    
    // 读取并显示导出的DDL
    use std::fs::File;
    use std::io::Read;
    
    let mut file = File::open("test_ddl_export.sql").map_err(|_| remdb::RemDbError::FileIoError)?;
    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|_| remdb::RemDbError::FileIoError)?;
    
    println!("{}", content);
    
    println!("=============================================");
    println!("DDL export test completed successfully!");
    
    Ok(())
}
