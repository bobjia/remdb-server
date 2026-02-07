#!/usr/bin/env python3
# 初始化数据库：创建表并插入测试数据

import sys
import os

# 添加remdb-python目录到Python路径
sys.path.append(os.path.join(os.path.dirname(__file__), 'remdb-python'))

from remdb import RemDbConnection

def init_db():
    """初始化数据库：创建表并插入测试数据"""
    try:
        # 连接数据库
        print("连接到数据库...")
        db = RemDbConnection("jdbc://localhost:6666/default")
        db.connect()
        print("连接成功！")
        
        # 删除已存在的表
        print("\n删除已存在的表...")
        try:
            db.execute_query("DROP TABLE IF EXISTS datatype_test")
            db.execute_query("DROP TABLE IF EXISTS orders")
            db.execute_query("DROP TABLE IF EXISTS products")
            db.execute_query("DROP TABLE IF EXISTS users")
            print("已删除旧表！")
        except:
            pass
        
        # 创建users表
        print("\n创建users表...")
        db.execute_query("CREATE TABLE users (id INTEGER PRIMARY KEY AUTO_INCREMENT, name varchar(16) UNIQUE NOT NULL, email TEXT NOT NULL, age INT default 20, created_at timestamptz(6))")
        print("users表创建成功！")
        
        # 创建products表
        print("\n创建products表...")
        db.execute_query("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT NOT NULL, price DOUBLE NOT NULL, description TEXT, stock INT NOT NULL DEFAULT 0)")
        print("products表创建成功！")
        
        # 创建orders表
        print("\n创建orders表...")
        db.execute_query("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INT NOT NULL, product_id INT NOT NULL, quantity INT NOT NULL, total_price DOUBLE NOT NULL, order_date BIGINT NOT NULL)")
        print("orders表创建成功！")
        
        # 创建datatype_test表测试所有remdb支持的数据类型
        print("\n创建datatype_test表...")
        db.execute_query("CREATE TABLE datatype_test (id INTEGER PRIMARY KEY AUTO_INCREMENT, bool_col BOOLEAN NOT NULL DEFAULT TRUE, char_col CHAR(10), varchar_col VARCHAR(50), text_col TEXT, int_col INTEGER, real_col REAL, double_col DOUBLE, timestamp_col TIMESTAMP, timestamptz_col TIMESTAMPTZ(6), json_col JSON)")
        print("datatype_test表创建成功！")
        
        # 插入测试数据
        print("\n插入测试数据...")
        test_data = [
            ("bob1", "a9", 1, 111111),
            ("bob2", "a8", 2, 111112),
            ("bob3", "a0", 3, 111113),
            ("bob4", "a", 1, 111211),
            ("bob5", "ba", 4, 111114),
            ("bob6", "ac", 1, 111111),
            ("bob7", "ab", 2, 111112),
            ("bob8", "ar", 3, 111113),
            ("bob9", "at", 1, 111211),
            ("bob10", "a1", 4, 111114),
            ("bob11", "a2", 4, 111114),
            ("bob12", "a3", 4, 111114),
            ("bob13", "a4", 4, 111114),
            ("bob14", "a5", 4, 111114),
            ("bob15", "a6", 4, 111114),
        ]
        
        for name, email, age, created_at in test_data:
            db.execute_query(f'INSERT INTO users (name, email, age, created_at) VALUES ("{name}", "{email}", {age}, {created_at})')
        
        print(f"成功插入{len(test_data)}条测试数据！")
        
        # 插入datatype_test表测试数据
        print("\n插入datatype_test表测试数据...")
        datatype_test_data = [
            (1, "FIXED", "variable string", "This is a long text field", 42, 3.14, 2.71828, 1704067200000, 1704067200000, '{"name": "test", "value": 123, "active": true}'),
            (0, "CHAR2", "varchar2", "Another text field", 100, -3.14, -2.71828, 1704153600000, 1704153600000, '{"name": "test2", "value": 456, "active": false}'),
            (1, "CHAR3", "varchar3", "Third text field", -50, 0.0, 0.0, 1704240000000, 1704240000000, '{"nested": {"key": "value"}, "array": [1, 2, 3]}'),
        ]
        
        for bool_val, char_val, varchar_val, text_val, int_val, real_val, double_val, timestamp_val, timestamptz_val, json_val in datatype_test_data:
            bool_str = "TRUE" if bool_val == 1 else "FALSE"
            db.execute_query(f'INSERT INTO datatype_test (bool_col, char_col, varchar_col, text_col, int_col, real_col, double_col, timestamp_col, timestamptz_col, json_col) VALUES ({bool_str}, "{char_val}", "{varchar_val}", "{text_val}", {int_val}, {real_val}, {double_val}, {timestamp_val}, {timestamptz_val}, {json_val})')
        
        print(f"成功插入{len(datatype_test_data)}条datatype_test测试数据！")
        
        print("\n数据库初始化完成！")
        
    except Exception as e:
        print(f"初始化失败: {e}")
        import traceback
        traceback.print_exc()
    finally:
        if 'db' in locals():
            db.close()

if __name__ == "__main__":
    init_db()