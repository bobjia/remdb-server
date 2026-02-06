import remdb
import tempfile
import os
import json

# 创建临时文件作为数据库路径
temp_file = tempfile.NamedTemporaryFile(delete=False)
db_path = temp_file.name
temp_file.close()

print("Testing JSON type support...")
print("=" * 50)

with remdb.connect(db_path) as db:
    # 创建表
    print("1. Creating table with JSON column...")
    db.execute_query("CREATE TABLE test (id INTEGER PRIMARY KEY, data JSON)")
    print("   Table created successfully!")
    
    # 测试1：不带引号的JSON（字面量）
    print("\n2. Testing JSON literal without quotes...")
    try:
        result = db.execute_query("INSERT INTO test (id, data) VALUES (1, [1,2,3])")
        print(f"   Insert Result: {result}")
        
        # 查询验证
        print("   Querying all records...")
        all_records = db.execute_query("SELECT * FROM test")
        print(f"   All records count: {all_records.rows_count}")
        
        if all_records.rows_count > 0:
            for i in range(all_records.rows_count):
                row = all_records.get_row(i)
                print(f"   Row {i}: {row}")
                
                # 验证JSON值
                if 'data' in row:
                    data_value = row['data']
                    print(f"   Data value: '{data_value}'")
                    try:
                        parsed_json = json.loads(data_value)
                        print(f"   Parsed JSON: {parsed_json}")
                        print(f"   ✓ JSON value is correct!")
                    except json.JSONDecodeError as e:
                        print(f"   ✗ Failed to parse JSON: {e}")
        
    except Exception as e:
        print(f"   Error: {e}")
        import traceback
        traceback.print_exc()
    
    # 测试2：带引号的JSON字符串
    print("\n3. Testing JSON string with quotes...")
    try:
        result = db.execute_query("INSERT INTO test (id, data) VALUES (2, '{\"name\":\"test\",\"value\":42}')")
        print(f"   Insert Result: {result}")
        
        # 查询验证
        print("   Querying all records...")
        all_records = db.execute_query("SELECT * FROM test")
        print(f"   All records count: {all_records.rows_count}")
        
        if all_records.rows_count > 0:
            for i in range(all_records.rows_count):
                row = all_records.get_row(i)
                print(f"   Row {i}: {row}")
                
                # 验证JSON值
                if 'data' in row:
                    data_value = row['data']
                    print(f"   Data value: '{data_value}'")
                    try:
                        parsed_json = json.loads(data_value)
                        print(f"   Parsed JSON: {parsed_json}")
                        print(f"   ✓ JSON value is correct!")
                    except json.JSONDecodeError as e:
                        print(f"   ✗ Failed to parse JSON: {e}")
        
    except Exception as e:
        print(f"   Error: {e}")
        import traceback
        traceback.print_exc()

print("\n" + "=" * 50)
print("Test completed!")

# 清理
if os.path.exists(db_path):
    os.unlink(db_path)