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
    result_set = db.execute_query("SELECT COUNT(*) FROM test_users")
    row = next(iter(result_set))
    count = int(row['COUNT(*)'])
    
    if count == 0:
        # 尝试在新表中插入初始数据
        print("\nInserting initial data into 'test_users'...")
        insert_sql = "INSERT INTO test_users (id, name, email, age) VALUES (1, 'Test User', 'test@example.com', 25)"
        db.execute_query(insert_sql)
        print("Initial data inserted successfully")


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
    # 事务示例
    print("\n=== Transaction Example ===")
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
    
    # 验证事务提交
    print("\nVerifying transaction commit...")
    for i in product_ids:
        product = table.get(i)
        if product:
            print(f"Product {i} found: {product['name']}")
        else:
            print(f"Product {i} not found")


def test_transaction_rollback(db, table):
    """
    测试事务回滚功能
    
    Args:
        db: RemDB数据库连接对象
        table: test_users表对象
    """
    # 回滚事务示例
    print("\n=== Rollback Transaction Example ===")
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
    
    # 验证回滚
    print("\nVerifying transaction rollback...")
    temp_product = table.get(temp_id)
    if temp_product:
        print(f"Temporary product still exists: {temp_product['name']}")
    else:
        print("Temporary product rolled back successfully")


def main():
    """
    主函数，执行所有测试操作
    """
    # 网络连接示例
    print("=== Network Connection Example ===")
    
    # JDBC URL格式: jdbc://host:port
    # 注意：请确保RemDB服务器已经在指定的主机和端口上运行
    jdbc_url = "jdbc://localhost:6666"
    
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
            test_basic_operations(db, table)
            
            # 测试事务提交
            test_transaction_commit(db, table)
            
            # 测试事务回滚
            test_transaction_rollback(db, table)
            
            print("\n=== All tests completed successfully! ===")
            
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
