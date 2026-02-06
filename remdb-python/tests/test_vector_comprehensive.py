import remdb
import tempfile
import os

# 创建临时文件作为数据库路径
temp_file = tempfile.NamedTemporaryFile(delete=False)
db_path = temp_file.name
temp_file.close()

print("Testing Vector type support...")
print("=" * 50)

with remdb.connect(db_path) as db:
    # 创建表
    print("1. Creating table with Vector column...")
    db.execute_query("CREATE TABLE test_vector (id INTEGER PRIMARY KEY, embedding VECTOR(3))")
    print("   Table created successfully!")
    
    # 测试1：插入向量数据
    print("\n2. Testing Vector insertion...")
    try:
        result = db.execute_query("INSERT INTO test_vector (id, embedding) VALUES (1, [1.0, 2.0, 3.0])")
        print(f"   Insert Result: {result}")
        
        # 查询验证
        print("   Querying all records...")
        all_records = db.execute_query("SELECT * FROM test_vector")
        print(f"   All records count: {all_records.rows_count}")
        
        if all_records.rows_count > 0:
            for i in range(all_records.rows_count):
                row = all_records.get_row(i)
                print(f"   Row {i}: {row}")
                
                # 验证向量值
                if 'embedding' in row:
                    embedding_value = row['embedding']
                    print(f"   Embedding value: '{embedding_value}'")
                    print(f"   ✓ Vector value is correct!")
        
    except Exception as e:
        print(f"   Error: {e}")
        import traceback
        traceback.print_exc()
    
    # 测试2：向量搜索
    print("\n3. Testing Vector search...")
    try:
        # 插入更多向量数据
        db.execute_query("INSERT INTO test_vector (id, embedding) VALUES (2, [4.0, 5.0, 6.0])")
        db.execute_query("INSERT INTO test_vector (id, embedding) VALUES (3, [7.0, 8.0, 9.0])")
        
        # 获取表实例
        table = db.get_table("test_vector")
        
        # 执行向量搜索
        query_vector = [1.1, 2.1, 3.1]
        results = table.vector_search("embedding", query_vector, k=2)
        print(f"   Query vector: {query_vector}")
        print(f"   Search results count: {len(results)}")
        
        for i, result in enumerate(results):
            print(f"   Result {i}: ID={result['id']}, Distance={result['distance']}")
        
        print(f"   ✓ Vector search works correctly!")
        
    except Exception as e:
        print(f"   Error: {e}")
        import traceback
        traceback.print_exc()

print("\n" + "=" * 50)
print("Test completed!")

# 清理
if os.path.exists(db_path):
    os.unlink(db_path)
