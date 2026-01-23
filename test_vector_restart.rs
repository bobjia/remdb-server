use remdb::{RemDb, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    // 清理旧的数据库文件
    let db_path = Path::new("./test_vector_restart_db");
    if db_path.exists() {
        fs::remove_dir_all(db_path)?;
    }

    // 1. 创建数据库实例
    let mut db = RemDb::create("test_vector_restart_db", None, None)?;
    println!("Database created successfully");

    // 2. 创建包含向量字段的表
    let create_table_sql = "CREATE TABLE test_table (
        id VARCHAR(64) PRIMARY KEY,
        name VARCHAR(64),
        embedding VECTOR(128)
    )";
    db.sql_query(create_table_sql)?;
    println!("Table created successfully");

    // 3. 插入一条数据
    let insert_sql = "INSERT INTO test_table (id, name, embedding) VALUES ('1', 'test', '[1.0, 2.0, 3.0, ...]')";
    db.sql_query(insert_sql)?;
    println!("Data inserted successfully");

    // 4. 执行DESCRIBE命令，查看向量类型显示
    let describe_sql = "DESCRIBE test_table";
    let result = db.sql_query(describe_sql)?;
    println!("DESCRIBE output before restart:");
    print_result(&result);

    // 5. 关闭数据库
    drop(db);
    println!("Database closed successfully");

    // 6. 重新打开数据库
    let mut db = RemDb::open("test_vector_restart_db", None)?;
    println!("Database reopened successfully");

    // 7. 再次执行DESCRIBE命令，查看向量类型显示
    let result = db.sql_query(describe_sql)?;
    println!("DESCRIBE output after restart:");
    print_result(&result);

    Ok(())
}

fn print_result(result: &remdb::sql::ResultSet) {
    // 打印列名
    for (i, col) in result.columns.iter().enumerate() {
        if i > 0 {
            print!(" | ");
        }
        print!("{:15}", col);
    }
    println!();

    // 打印分隔线
    for (i, _) in result.columns.iter().enumerate() {
        if i > 0 {
            print!("+-");
        } else {
            print!("+- ");
        }
        print!("{:13}-+", "-");
    }
    println!();

    // 打印行数据
    for row in &result.rows {
        for (i, val) in row.values.iter().enumerate() {
            let val_str = match val.value_type {
                remdb::types::DataType::String => {
                    let s = unsafe { &val.value.string };
                    std::str::from_utf8(s)
                        .unwrap_or("")
                        .trim_end_matches(char::from(0))
                        .to_string()
                }
                _ => "".to_string(),
            };
            if i > 0 {
                print!(" | ");
            }
            print!("{:15}", val_str);
        }
        println!();
    }
    println!();
}