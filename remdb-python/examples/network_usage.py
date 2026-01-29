"""
Network usage example of RemDB Python bindings

This example demonstrates how to use RemDB Python bindings to connect to a RemDB server over the network using the JDBC protocol.
"""

import remdb


def main():
    """
    Main function
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
            
            # 执行SQL查询
            print("\nExecuting SQL query...")
            result_set = db.execute_query("SELECT * FROM products LIMIT 5")
            
            # 处理查询结果
            print(f"Found {result_set.get_rows_count()} products:")
            for row in result_set:
                print(row)
            
            # 获取表
            print("\nGetting table 'products'...")
            table = db.get_table("products")
            print(f"Table 'products' retrieved successfully")
            
            # 插入记录
            print("\nInserting a new product...")
            new_product = {
                "id": 101,
                "name": "Wireless Mouse",
                "price": 29.99,
                "category": "electronics"
            }
            success = table.insert(new_product)
            print(f"Insert successful: {success}")
            
            # 获取记录
            print("\nGetting product with id=101...")
            product = table.get(101)
            if product:
                print(f"Product found: {product}")
            else:
                print("Product not found")
            
            # 更新记录
            print("\nUpdating product with id=101...")
            updated_product = {
                "name": "Wireless Mouse Pro",
                "price": 39.99
            }
            success = table.update(101, updated_product)
            print(f"Update successful: {success}")
            
            # 获取更新后的记录
            print("\nGetting updated product with id=101...")
            product = table.get(101)
            if product:
                print(f"Updated product: {product}")
            else:
                print("Product not found")
            
            # 删除记录
            print("\nDeleting product with id=101...")
            success = table.delete(101)
            print(f"Delete successful: {success}")
            
            # 验证删除
            print("\nVerifying product deletion...")
            product = table.get(101)
            if product:
                print(f"Product still exists: {product}")
            else:
                print("Product deleted successfully")
            
            # 事务示例
            print("\n=== Transaction Example ===")
            with db.begin_transaction() as tx:
                print("Transaction started")
                
                # 插入多条记录
                for i in range(102, 105):
                    product = {
                        "id": i,
                        "name": f"Product {i}",
                        "price": 19.99 + i,
                        "category": "electronics"
                    }
                    table.insert(product)
                    print(f"Inserted product {i}")
                
                # 提交事务
                print("Committing transaction...")
            # 事务会在上下文退出时自动提交
            
            # 验证事务提交
            print("\nVerifying transaction commit...")
            for i in range(102, 105):
                product = table.get(i)
                if product:
                    print(f"Product {i} found: {product['name']}")
                else:
                    print(f"Product {i} not found")
            
            # 回滚事务示例
            print("\n=== Rollback Transaction Example ===")
            with db.begin_transaction() as tx:
                print("Transaction started")
                
                # 插入记录
                product = {
                    "id": 201,
                    "name": "Temporary Product",
                    "price": 99.99,
                    "category": "test"
                }
                table.insert(product)
                print("Inserted temporary product")
                
                # 验证插入
                temp_product = table.get(201)
                if temp_product:
                    print(f"Temporary product found: {temp_product['name']}")
                else:
                    print("Temporary product not found")
                
                # 显式回滚事务
                print("Rolling back transaction...")
                tx.rollback()
            
            # 验证回滚
            print("\nVerifying transaction rollback...")
            temp_product = table.get(201)
            if temp_product:
                print(f"Temporary product still exists: {temp_product['name']}")
            else:
                print("Temporary product rolled back successfully")
            
    except Exception as e:
        print(f"Error: {e}")
        print("\nNote: Please make sure that:")
        print("1. RemDB server is running on the specified host and port")
        print("2. The 'products' table exists in the database")
        print("3. You have the necessary permissions to access the database")


if __name__ == "__main__":
    main()
