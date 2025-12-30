mod ddl_compiler;
mod snapshot_loader;
mod sql_engine;
mod cli;

use crate::ddl_compiler::{compile_ddl_file};
use crate::sql_engine::{execute_extended_sql};
use remdb::RemDb;

fn main() {
    println!("Testing export functionality...");
    
    // 编译DDL文件
    let tables = compile_ddl_file("schema.ddl").expect("Failed to compile DDL");
    
    // 创建默认内存分配器
    static mut DEFAULT_ALLOCATOR: remdb::config::DefaultMemoryAllocator = remdb::config::DefaultMemoryAllocator;
    
    // 使用非常小的默认最大记录数
    let small_max_records = 1;
    
    // 将tables向量泄漏到静态内存
    let static_tables = Box::leak(Box::new(tables));
    
    // 创建配置
    let config = Box::leak(Box::new(remdb::config::DbConfig {
        tables: static_tables,
        total_memory: 1024 * 1024 * 100, // 100MB
        default_max_records: small_max_records,
        low_power_mode_supported: true,
        low_power_max_records: Some(100),
        memory_allocator: unsafe {
            &*(&raw const DEFAULT_ALLOCATOR as *const _) as &'static dyn remdb::config::MemoryAllocator
        },
    }));
    
    // 初始化全局内存分配器
    let total_memory = config.total_memory;
    let memory_vec: Vec<u8> = Vec::with_capacity(total_memory);
    let memory_ptr = memory_vec.as_ptr() as *mut u8;
    std::mem::forget(memory_vec);
    
    unsafe {
        remdb::memory::allocator::init_global_allocator(memory_ptr, total_memory)
            .expect("Failed to initialize global allocator");
    }
    
    // 初始化数据库
    let mut db = unsafe {
        remdb::init_global_db(config)
            .expect("Failed to initialize database")
    };
    
    // 测试导出DDL
    println!("\n1. Testing EXPORT DDL...");
    let result = execute_extended_sql(&mut db, "export ddl exported_schema.ddl");
    match result {
        Ok(result_set) => {
            println!("✓ Success: Exported {} table(s) to exported_schema.ddl", result_set.affected_rows);
        },
        Err(err) => {
            println!("✗ Error: {}", err);
        }
    }
    
    // 测试导出数据
    println!("\n2. Testing EXPORT DATA...");
    let tables = ["users", "products", "orders"];
    for table in tables {
        let sql = format!("export data {} {}.csv", table, table);
        let result = execute_extended_sql(&mut db, &sql);
        match result {
            Ok(result_set) => {
                println!("✓ Success: Exported {} row(s) from {} to {}.csv", result_set.affected_rows, table, table);
            },
            Err(err) => {
                println!("✗ Error exporting {}: {}", table, err);
            }
        }
    }
    
    // 测试导出全部
    println!("\n3. Testing EXPORT ALL...");
    let result = execute_extended_sql(&mut db, "export all export_all");
    match result {
        Ok(result_set) => {
            println!("✓ Success: Exported all data for {} table(s) to export_all/", result_set.affected_rows);
        },
        Err(err) => {
            println!("✗ Error: {}", err);
        }
    }
    
    // 检查导出结果
    println!("\n4. Checking export results...");
    use std::fs;
    
    if fs::metadata("exported_schema.ddl").is_ok() {
        println!("✓ exported_schema.ddl created");
        let content = fs::read_to_string("exported_schema.ddl").unwrap();
        println!("   Content preview:");
        for (i, line) in content.lines().take(10).enumerate() {
            println!("   {:02}: {}", i+1, line);
        }
    } else {
        println!("✗ exported_schema.ddl not found");
    }
    
    if fs::metadata("export_all").is_ok() {
        println!("\n✓ export_all directory created");
        for entry in fs::read_dir("export_all").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                println!("   - {}", path.file_name().unwrap().to_str().unwrap());
            }
        }
    } else {
        println!("\n✗ export_all directory not found");
    }
    
    println!("\nTest completed!");
}