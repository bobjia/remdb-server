"""Test new features of RemDB Python bindings"""

import unittest
import remdb
import tempfile
import os

class TestGetByIdMethod(unittest.TestCase):
    """Test get_by_id method functionality"""

    def setUp(self):
        """Set up test environment"""
        # 创建临时文件作为数据库路径
        self.temp_file = tempfile.NamedTemporaryFile(delete=False)
        self.db_path = self.temp_file.name
        self.temp_file.close()

    def tearDown(self):
        """Clean up test environment"""
        # 删除临时文件
        if os.path.exists(self.db_path):
            os.unlink(self.db_path)

    def test_get_by_id_basic(self):
        """Test basic get_by_id functionality"""
        with remdb.connect(self.db_path) as db:
            # 创建表
            db.execute_query("CREATE TABLE test_get_by_id (id INTEGER PRIMARY KEY, name TEXT, value REAL)")
            
            # 获取表实例
            table = db.get_table("test_get_by_id")
            
            # 插入测试数据
            test_data = {"id": 1, "name": "Test Item", "value": 23.5}
            table.insert(test_data)
            
            # 使用get_by_id获取数据
            result = table.get_by_id(1)
            self.assertIsInstance(result, dict)
            self.assertEqual(result.get("id"), 1)
            
    def test_get_by_id_zero_copy(self):
        """Test get_by_id with zero-copy mode"""
        with remdb.connect(self.db_path) as db:
            # 创建表
            db.execute_query("CREATE TABLE test_get_by_id_zero_copy (id INTEGER PRIMARY KEY, name TEXT)")
            
            # 获取表实例
            table = db.get_table("test_get_by_id_zero_copy")
            
            # 插入测试数据
            table.insert({"id": 1, "name": "Test Item"})
            
            # 使用get_by_id的零拷贝模式
            result = table.get_by_id(1, zero_copy=True)
            # 零拷贝模式应该返回memoryview或字典
            self.assertIsInstance(result, (dict, memoryview))

    def test_get_by_id_not_found(self):
        """Test get_by_id with non-existent record"""
        with remdb.connect(self.db_path) as db:
            # 创建表
            db.execute_query("CREATE TABLE test_get_by_id_not_found (id INTEGER PRIMARY KEY, name TEXT)")
            
            # 获取表实例
            table = db.get_table("test_get_by_id_not_found")
            
            # 尝试获取不存在的记录
            result = table.get_by_id(999)
            self.assertIsNone(result)

class TestBatchOperations(unittest.TestCase):
    """Test batch operations functionality"""

    def setUp(self):
        """Set up test environment"""
        # 创建临时文件作为数据库路径
        self.temp_file = tempfile.NamedTemporaryFile(delete=False)
        self.db_path = self.temp_file.name
        self.temp_file.close()

    def tearDown(self):
        """Clean up test environment"""
        # 删除临时文件
        if os.path.exists(self.db_path):
            os.unlink(self.db_path)

    def test_batch_insert(self):
        """Test batch insert functionality"""
        with remdb.connect(self.db_path) as db:
            # 创建表
            db.execute_query("CREATE TABLE test_batch_insert (id INTEGER PRIMARY KEY, name TEXT, value REAL)")
            
            # 获取表实例
            table = db.get_table("test_batch_insert")
            
            # 准备批量插入数据
            test_records = [
                {"id": 1, "name": "Item 1", "value": 10.5},
                {"id": 2, "name": "Item 2", "value": 20.5},
                {"id": 3, "name": "Item 3", "value": 30.5}
            ]
            
            # 执行批量插入
            success = table.batch_insert(test_records)
            self.assertTrue(success)
            
            # 验证数据是否插入成功
            for i in range(1, 4):
                result = table.get(i)
                self.assertIsInstance(result, dict)
                self.assertEqual(result.get("id"), i)

    def test_batch_update(self):
        """Test batch update functionality"""
        with remdb.connect(self.db_path) as db:
            # 创建表
            db.execute_query("CREATE TABLE test_batch_update (id INTEGER PRIMARY KEY, name TEXT, value REAL)")
            
            # 获取表实例
            table = db.get_table("test_batch_update")
            
            # 插入初始数据
            initial_records = [
                {"id": 1, "name": "Item 1", "value": 10.5},
                {"id": 2, "name": "Item 2", "value": 20.5},
                {"id": 3, "name": "Item 3", "value": 30.5}
            ]
            for record in initial_records:
                table.insert(record)
            
            # 准备批量更新数据
            update_data = [
                (1, {"name": "Updated Item 1", "value": 15.5}),
                (2, {"name": "Updated Item 2", "value": 25.5}),
                (3, {"name": "Updated Item 3", "value": 35.5})
            ]
            
            # 执行批量更新
            success = table.batch_update(update_data)
            self.assertTrue(success)
            
            # 验证数据是否更新成功
            expected_values = [15.5, 25.5, 35.5]
            for i in range(1, 4):
                result = table.get(i)
                self.assertIsInstance(result, dict)
                self.assertEqual(result.get("id"), i)
                self.assertEqual(result.get("name"), f"Updated Item {i}")
                self.assertEqual(result.get("value"), expected_values[i-1])

    def test_batch_delete(self):
        """Test batch delete functionality"""
        with remdb.connect(self.db_path) as db:
            # 创建表
            db.execute_query("CREATE TABLE test_batch_delete (id INTEGER PRIMARY KEY, name TEXT)")
            
            # 获取表实例
            table = db.get_table("test_batch_delete")
            
            # 插入测试数据
            test_records = [
                {"id": 1, "name": "Item 1"},
                {"id": 2, "name": "Item 2"},
                {"id": 3, "name": "Item 3"},
                {"id": 4, "name": "Item 4"},
                {"id": 5, "name": "Item 5"}
            ]
            for record in test_records:
                table.insert(record)
            
            # 准备批量删除的ID列表
            delete_ids = [1, 3, 5]
            
            # 执行批量删除
            success = table.batch_delete(delete_ids)
            self.assertTrue(success)
            
            # 验证数据是否删除成功
            for delete_id in delete_ids:
                result = table.get(delete_id)
                self.assertIsNone(result)
            
            # 验证未删除的数据仍然存在
            for keep_id in [2, 4]:
                result = table.get(keep_id)
                self.assertIsInstance(result, dict)
                self.assertEqual(result.get("id"), keep_id)

if __name__ == '__main__':
    unittest.main()
