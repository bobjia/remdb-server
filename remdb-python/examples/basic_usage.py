"""Basic usage example of RemDB Python bindings"""

import remdb
import tempfile
import os

# 创建临时文件作为数据库路径
temp_file = tempfile.NamedTemporaryFile(delete=False)
db_path = temp_file.name
temp_file.close()

try:
    # 连接到数据库（使用上下文管理器）
    print("Connecting to database...")
    with remdb.connect(db_path) as db:
        print("Connected successfully!")
        
        # 尝试获取表（这里假设数据库中已经有一个名为"sensor_data"的表）
        try:
            print("Getting table 'sensor_data'...")
            table = db.get_table("sensor_data")
            print(f"Table found with {table.get_record_count()} records")
            
            # 插入记录
            print("Inserting a record...")
            record = {
                "id": 1,
                "value": 23.5,
                "timestamp": 1620000000
            }
            success = table.insert(record)
            print(f"Insert successful: {success}")
            
            # 获取记录（普通模式）
            print("Getting record with id=1...")
            result = table.get(1)
            print(f"Record found: {result}")
            
            # 获取记录（零拷贝模式）
            print("Getting record with id=1 (zero-copy mode)...")
            result_zero_copy = table.get(1, zero_copy=True)
            print(f"Zero-copy result type: {type(result_zero_copy)}")
            
            # 更新记录
            print("Updating record with id=1...")
            updated_record = {
                "id": 1,
                "value": 25.0,
                "timestamp": 1620000001
            }
            success = table.update(1, updated_record)
            print(f"Update successful: {success}")
            
            # 删除记录
            print("Deleting record with id=1...")
            success = table.delete(1)
            print(f"Delete successful: {success}")
            
        except remdb.NotFoundError as e:
            print(f"Table not found: {e}")
        
        # 执行SQL查询
        print("Executing SQL query...")
        try:
            result_set = db.execute_query("SELECT * FROM sensor_data LIMIT 10")
            print(f"Query returned {result_set.get_rows_count()} rows")
            
            # 遍历结果集
            print("Query results:")
            for row in result_set:
                print(row)
        except Exception as e:
            print(f"Query failed: {e}")
        
        # 事务处理示例
        print("\nTesting transaction...")
        try:
            with db.begin_transaction() as tx:
                print("Transaction started")
                # 在事务中执行操作
                # 这里可以执行多个操作，它们会作为一个原子操作执行
                print("Transaction committed successfully")
        except Exception as e:
            print(f"Transaction failed: {e}")
        
finally:
    # 清理临时文件
    if os.path.exists(db_path):
        os.unlink(db_path)
        print(f"\nCleaned up temporary file: {db_path}")

print("\nExample completed!")
