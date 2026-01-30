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
        
        # 尝试获取表，如果不存在则创建
        try:
            print("Getting table 'sensor_data'...")
            table = db.get_table("sensor_data")
            print(f"Table found with {table.get_record_count()} records")
        except remdb.NotFoundError:
            print("Table 'sensor_data' not found, creating it...")
            # 尝试创建表
            try:
                # 使用更简单的SQL语法
                create_table_sql = "CREATE TABLE sensor_data (id INTEGER PRIMARY KEY, value REAL, timestamp INTEGER)"
                result = db.execute_query(create_table_sql)
                print(f"Table creation result: {result}")
                # 重新获取表
                table = db.get_table("sensor_data")
                print("Table created successfully")
            except Exception as e:
                print(f"Error creating table: {e}")
                raise
            
            # 插入记录
            print("Inserting a record...")
            # 使用SQL INSERT语句直接插入数据
            insert_sql = "INSERT INTO sensor_data (id, value, timestamp) VALUES (1, 23.5, 1620000000)"
            try:
                result = db.execute_query(insert_sql)
                print(f"Insert result: {result}")
                print("Insert successful")
            except Exception as e:
                print(f"Error inserting record: {e}")
            
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
            # 使用SQL UPDATE语句直接更新数据
            update_sql = "UPDATE sensor_data SET value = 25.0, timestamp = 1620000001 WHERE id = 1"
            try:
                result = db.execute_query(update_sql)
                print(f"Update result: {result}")
                print("Update successful")
            except Exception as e:
                print(f"Error updating record: {e}")
            
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
            # 尝试开始事务
            print("Attempting to begin transaction...")
            tx = db.begin_transaction()
            print("Transaction started")
            
            # 在事务中执行实际的数据库操作
            # 插入一条新记录
            insert_sql = "INSERT INTO sensor_data (id, value, timestamp) VALUES (2, 26.5, 1620000002)"
            db.execute_query(insert_sql)
            print("Inserted record in transaction")
            
            # 提交事务
            tx.commit()
            print("Transaction committed successfully")
        except remdb.TransactionError as e:
            print(f"Transaction failed: {e}")
            print("Note: Transaction support may not be available in local file mode")
            print("This is expected behavior in some RemDB configurations")
        except Exception as e:
            print(f"Unexpected error during transaction: {e}")
            import traceback
            traceback.print_exc()
        
finally:
    # 清理临时文件
    if os.path.exists(db_path):
        os.unlink(db_path)
        print(f"\nCleaned up temporary file: {db_path}")

print("\nExample completed!")
