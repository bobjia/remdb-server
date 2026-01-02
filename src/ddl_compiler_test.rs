use remdb::{types::{TableDef, FieldDef, DataType}, Result as RemResult};
use crate::ddl_compiler::parse_ddl_content;

#[test]
fn test_create_table_with_auto_increment() -> RemResult<()> {
    // Test CREATE TABLE statement with AUTO_INCREMENT and VARCHAR(n)
    let create_table_sql = "CREATE TABLE iot_devices (id INT AUTO_INCREMENT PRIMARY KEY,  device_id VARCHAR(50),  timestamp BIGINT,  temperature DOUBLE,  humidity DOUBLE,  pressure DOUBLE,  battery_level INT);";
    
    // Parse the CREATE TABLE statement
    let tables = parse_ddl_content(create_table_sql)?;
    
    // Verify that one table was created
    assert_eq!(tables.len(), 1, "Expected 1 table, got {}", tables.len());
    
    let table = &tables[0];
    assert_eq!(table.name, "iot_devices", "Expected table name 'iot_devices', got '{}'", table.name);
    
    // Verify that 7 fields were created
    assert_eq!(table.fields.len(), 7, "Expected 7 fields, got {}", table.fields.len());
    
    // Verify field properties
    let fields = &table.fields;
    
    // Check id field (AUTO_INCREMENT PRIMARY KEY)
    assert_eq!(fields[0].name, "id", "Expected field name 'id', got '{}'", fields[0].name);
    assert_eq!(fields[0].data_type, DataType::Int32, "Expected INT data type for id, got {:?}", fields[0].data_type);
    assert_eq!(fields[0].size, 4, "Expected size 4 for INT, got {}", fields[0].size);
    assert_eq!(fields[0].primary_key, true, "Expected id to be primary key, got false");
    assert_eq!(fields[0].auto_increment, true, "Expected id to be AUTO_INCREMENT, got false");
    
    // Check device_id field (VARCHAR(50))
    assert_eq!(fields[1].name, "device_id", "Expected field name 'device_id', got '{}'", fields[1].name);
    assert_eq!(fields[1].data_type, DataType::String, "Expected String data type for device_id, got {:?}", fields[1].data_type);
    assert_eq!(fields[1].size, 50, "Expected size 50 for VARCHAR(50), got {}", fields[1].size);
    
    // Check timestamp field (BIGINT)
    assert_eq!(fields[2].name, "timestamp", "Expected field name 'timestamp', got '{}'", fields[2].name);
    assert_eq!(fields[2].data_type, DataType::Int64, "Expected BIGINT data type for timestamp, got {:?}", fields[2].data_type);
    assert_eq!(fields[2].size, 8, "Expected size 8 for BIGINT, got {}", fields[2].size);
    
    // Check temperature field (DOUBLE)
    assert_eq!(fields[3].name, "temperature", "Expected field name 'temperature', got '{}'", fields[3].name);
    assert_eq!(fields[3].data_type, DataType::Float64, "Expected DOUBLE data type for temperature, got {:?}", fields[3].data_type);
    assert_eq!(fields[3].size, 8, "Expected size 8 for DOUBLE, got {}", fields[3].size);
    
    // Check humidity field (DOUBLE)
    assert_eq!(fields[4].name, "humidity", "Expected field name 'humidity', got '{}'", fields[4].name);
    assert_eq!(fields[4].data_type, DataType::Float64, "Expected DOUBLE data type for humidity, got {:?}", fields[4].data_type);
    assert_eq!(fields[4].size, 8, "Expected size 8 for DOUBLE, got {}", fields[4].size);
    
    // Check pressure field (DOUBLE)
    assert_eq!(fields[5].name, "pressure", "Expected field name 'pressure', got '{}'", fields[5].name);
    assert_eq!(fields[5].data_type, DataType::Float64, "Expected DOUBLE data type for pressure, got {:?}", fields[5].data_type);
    assert_eq!(fields[5].size, 8, "Expected size 8 for DOUBLE, got {}", fields[5].size);
    
    // Check battery_level field (INT)
    assert_eq!(fields[6].name, "battery_level", "Expected field name 'battery_level', got '{}'", fields[6].name);
    assert_eq!(fields[6].data_type, DataType::Int32, "Expected INT data type for battery_level, got {:?}", fields[6].data_type);
    assert_eq!(fields[6].size, 4, "Expected size 4 for INT, got {}", fields[6].size);
    
    println!("✓ CREATE TABLE with AUTO_INCREMENT and VARCHAR(n) parsed successfully!");
    
    Ok(())
}