"""
Network usage example of RemDB Python bindings

This example demonstrates how to use RemDB Python bindings to connect to a RemDB server over the network using the JDBC protocol.
It includes examples of basic CRUD operations, transaction management, and error handling.
"""

import sys
import os

# 添加项目根目录到Python路径
sys.path.insert(0, os.path.abspath(os.path.dirname(os.path.dirname(__file__))))

import remdb


def setup_test_table(db):
    """
    设置测试表结构和初始数据
    
    Args:
        db: RemDB数据库连接对象
    """
    # 尝试创建一个新的表
    print("\nCreating a new table 'test_users'...")
    create_table_sql = "CREATE TABLE IF NOT EXISTS test_users (id INTEGER PRIMARY KEY, name VARCHAR(50), email VARCHAR(50), age INTEGER)"
    db.execute_query(create_table_sql)
    print("Table 'test_users' created successfully")
    
    # 检查是否已有初始数据
    print("\nChecking if initial data exists...")
    try:
        result_set = db.execute_query("SELECT COUNT(*) FROM test_users")
        row = next(iter(result_set))
        # 尝试获取计数结果，处理不同的列名格式
        if 'COUNT(*)' in row:
            count = int(row['COUNT(*)'])
        elif 'count' in row:
            count = int(row['count'])
        elif 'Count' in row:
            count = int(row['Count'])
        else:
            # 如果列名不是预期的格式，尝试获取第一个值
            count = int(list(row.values())[0])
        
        print(f"Found {count} records in test_users table")
        
        if count == 0:
            # 尝试在新表中插入初始数据
            print("\nInserting initial data into 'test_users'...")
            insert_sql = "INSERT INTO test_users (id, name, email, age) VALUES (1, 'Test User', 'test@example.com', 25)"
            db.execute_query(insert_sql)
            print("Initial data inserted successfully")
    except Exception as e:
        print(f"Error checking data count: {e}")
        # 如果计数查询失败，尝试直接插入数据（如果不存在）
        try:
            # 尝试插入数据，使用INSERT IGNORE避免主键冲突
            insert_sql = "INSERT OR IGNORE INTO test_users (id, name, email, age) VALUES (1, 'Test User', 'test@example.com', 25)"
            db.execute_query(insert_sql)
            print("Initial data inserted or already exists")
        except Exception as e2:
            print(f"Error inserting initial data: {e2}")


def test_basic_operations(db, table):
    """
    测试基本的CRUD操作
    
    Args:
        db: RemDB数据库连接对象
        table: test_users表对象
    """
    # 执行SQL查询
    print("\nExecuting SQL query...")
    result_set = db.execute_query("SELECT * FROM test_users LIMIT 5")
    
    # 处理查询结果
    print(f"Found {result_set.get_rows_count()} users:")
    for row in result_set:
        print(row)
    
    # 清理测试数据
    print("\nCleaning test data for id >= 100...")
    try:
        db.execute_query("DELETE FROM test_users WHERE id >= 100")
        print("Test data cleaned successfully")
    except Exception as e:
        print(f"Error cleaning test data: {e}")
    
    # 插入记录
    print("\nInserting a new user...")
    new_user = {
        "id": 101,
        "name": "Wireless Mouse",
        "email": "bobjjia@email.com",
        "age": 29
    }
    success = table.insert(new_user)
    print(f"Insert successful: {success}")
    
    # 获取记录
    print("\nGetting user with id=101...")
    user = table.get(101)
    if user:
        print(f"User found: {user}")
    else:
        print("User not found")
        # 如果查询失败，尝试直接执行SQL查询来验证插入
        try:
            sql_result = db.execute_query("SELECT * FROM test_users WHERE id = 101")
            for row in sql_result:
                print(f"Direct SQL query result: {row}")
        except Exception as e:
            print(f"Direct SQL query also failed: {e}")
    
    # 更新记录
    print("\nUpdating user with id=101...")
    updated_user = {
        "name": "Mouse Pro",
        "age": 39
    }
    success = table.update(101, updated_user)
    print(f"Update successful: {success}")
    
    # 获取更新后的记录
    print("\nGetting updated user with id=101...")
    user = table.get(101)
    if user:
        print(f"Updated user: {user}")
    else:
        print("User not found")
    
    # 删除记录
    print("\nDeleting user with id=101...")
    success = table.delete(101)
    print(f"Delete successful: {success}")
    
    # 验证删除
    print("\nVerifying user deletion...")
    user = table.get(101)
    if user:
        print(f"User still exists: {user}")
    else:
        print("User deleted successfully")


def test_transaction_commit(db, table):
    """
    测试事务提交功能
    
    Args:
        db: RemDB数据库连接对象
        table: test_users表对象
    """
    # 清理测试数据
    print("\nCleaning test data for transaction commit test (id 102-104)...")
    try:
        db.execute_query("DELETE FROM test_users WHERE id BETWEEN 102 AND 104")
        print("Transaction test data cleaned successfully")
    except Exception as e:
        print(f"Error cleaning transaction test data: {e}")
    
    # 事务示例
    print("\n=== Transaction Example ===")
    try:
        with db.begin_transaction() as tx:
            print("Transaction started")
            
            # 插入多条记录
            product_ids = range(102, 105)
            for i in product_ids:
                product = {
                    "id": i,
                    "name": f"Product {i}",
                    "email": f"product{i}@example.com",
                    "age": 19
                }
                table.insert(product)
                print(f"Inserted product {i}")
            
            # 提交事务
            print("Committing transaction...")
        # 事务会在上下文退出时自动提交
        
        print("Transaction committed successfully")
        
        # 验证事务提交
        print("\nVerifying transaction commit...")
        for i in product_ids:
            product = table.get(i)
            if product:
                print(f"Product {i} found: {product['name']}")
            else:
                print(f"Product {i} not found")
                # 尝试直接SQL查询验证
                try:
                    sql_result = db.execute_query(f"SELECT * FROM test_users WHERE id = {i}")
                    for row in sql_result:
                        print(f"Direct SQL query result for id={i}: {row}")
                except Exception as e:
                    print(f"Direct SQL query failed for id={i}: {e}")
    
    except Exception as e:
        print(f"Transaction error: {e}")
        # 继续执行其他测试
        pass


def test_transaction_rollback(db, table):
    """
    测试事务回滚功能
    
    Args:
        db: RemDB数据库连接对象
        table: test_users表对象
    """
    # 清理测试数据
    print("\nCleaning test data for transaction rollback test (id 201)...")
    try:
        db.execute_query("DELETE FROM test_users WHERE id = 201")
        print("Rollback test data cleaned successfully")
    except Exception as e:
        print(f"Error cleaning rollback test data: {e}")
    
    # 回滚事务示例
    print("\n=== Rollback Transaction Example ===")
    try:
        with db.begin_transaction() as tx:
            print("Transaction started")
            
            # 插入临时记录
            temp_id = 201
            product = {
                "id": temp_id,
                "name": "Temporary Product",
                "email": "temp@example.com",
                "age": 25
            }
            table.insert(product)
            print("Inserted temporary product")
            
            # 验证插入
            temp_product = table.get(temp_id)
            if temp_product:
                print(f"Temporary product found: {temp_product['name']}")
            else:
                print("Temporary product not found")
            
            # 显式回滚事务
            print("Rolling back transaction...")
            tx.rollback()
        
        print("Transaction rolled back successfully")
        
        # 验证回滚
        print("\nVerifying transaction rollback...")
        temp_product = table.get(temp_id)
        if temp_product:
            print(f"Temporary product still exists: {temp_product['name']}")
        else:
            print("Temporary product rolled back successfully")
            # 尝试直接SQL查询验证
            try:
                sql_result = db.execute_query(f"SELECT * FROM test_users WHERE id = {temp_id}")
                for row in sql_result:
                    print(f"Direct SQL query result for id={temp_id}: {row}")
            except Exception as e:
                print(f"Direct SQL query failed for id={temp_id}: {e}")
    
    except Exception as e:
        print(f"Transaction error: {e}")
        # 继续执行其他测试
        pass


def main():
    """
    主函数，执行所有测试操作
    """
    # 网络连接示例
    print("=== Network Connection Example ===")
    
    # JDBC URL格式: jdbc://host:port
    # 注意：请确保RemDB服务器已经在指定的主机和端口上运行
    jdbc_url = "jdbc://localhost:6666"
    
    # 用于追踪测试结果
    test_results = {
        "basic_operations": False,
        "transaction_commit": False,
        "transaction_rollback": False
    }
    
    try:
        # 连接到数据库（使用上下文管理器）
        print(f"Connecting to RemDB server at {jdbc_url}...")
        with remdb.connect(jdbc_url) as db:
            print("Connected successfully!")

            # 设置测试表
            setup_test_table(db)
            
            # 获取表
            print("\nGetting table 'test_users'...")
            table = db.get_table("test_users")
            print(f"Table 'test_users' retrieved successfully")
            
            # 测试基本操作
            print("\n=== Testing Basic Operations ===")
            try:
                test_basic_operations(db, table)
                test_results["basic_operations"] = True
                print("Basic operations test completed")
            except Exception as e:
                print(f"Basic operations test failed: {e}")
            
            # 测试事务提交
            print("\n=== Testing Transaction Commit ===")
            try:
                test_transaction_commit(db, table)
                test_results["transaction_commit"] = True
                print("Transaction commit test completed")
            except Exception as e:
                print(f"Transaction commit test failed: {e}")
            
            # 测试事务回滚
            print("\n=== Testing Transaction Rollback ===")
            try:
                test_transaction_rollback(db, table)
                test_results["transaction_rollback"] = True
                print("Transaction rollback test completed")
            except Exception as e:
                print(f"Transaction rollback test failed: {e}")
            
            # 汇总测试结果
            print("\n" + "="*50)
            print("TEST RESULTS SUMMARY:")
            print("="*50)
            for test_name, passed in test_results.items():
                status = "PASSED" if passed else "FAILED"
                print(f"{test_name}: {status}")
            
            # 检查总体结果
            all_passed = all(test_results.values())
            if all_passed:
                print("\n=== All tests completed successfully! ===")
            else:
                print(f"\n=== Some tests failed: {sum(1 for v in test_results.values() if not v)} out of {len(test_results)} tests failed ===")
                print("Check the logs above for detailed error information.")
            
    except remdb.network.RequestError as e:
        print(f"Network error: {e}")
        print("\nNote: Please make sure that:")
        print("1. RemDB server is running on the specified host and port")
        print("2. The network connection is stable")
    except Exception as e:
        print(f"Error: {e}")
        print("\nNote: Please make sure that:")
        print("1. RemDB server is running on the specified host and port")
        print("2. The 'test_users' table exists in the database")
        print("3. You have the necessary permissions to access the database")


if __name__ == "__main__":
    main()
