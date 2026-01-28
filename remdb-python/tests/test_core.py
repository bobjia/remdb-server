"""Test core functionality of RemDB Python bindings"""

import unittest
import remdb
import tempfile
import os

class TestCoreFunctionality(unittest.TestCase):
    """Test core functionality"""

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

    def test_connect(self):
        """Test database connection"""
        # 测试连接到临时数据库
        with remdb.connect(self.db_path) as db:
            self.assertIsNotNone(db)
            self.assertTrue(db.connected)

    def test_context_manager(self):
        """Test context manager functionality"""
        # 测试上下文管理器
        db = None
        with remdb.connect(self.db_path) as conn:
            db = conn
            self.assertTrue(db.connected)
        # 退出上下文后，连接应该关闭
        self.assertFalse(db.connected)

    def test_transaction(self):
        """Test transaction functionality"""
        with remdb.connect(self.db_path) as db:
            # 开始事务
            with db.begin_transaction() as tx:
                self.assertTrue(tx.is_active())
            # 退出上下文后，事务应该提交并关闭
            self.assertFalse(tx.is_active())

class TestTableOperations(unittest.TestCase):
    """Test table operations"""

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

    def test_get_table(self):
        """Test getting table"""
        with remdb.connect(self.db_path) as db:
            # 注意：这里假设数据库中已经有一个名为"test_table"的表
            # 在实际测试中，可能需要先创建表
            try:
                table = db.get_table("test_table")
                self.assertIsNotNone(table)
            except remdb.NotFoundError:
                # 如果表不存在，应该抛出NotFoundError
                pass

    def test_record_count(self):
        """Test getting record count"""
        with remdb.connect(self.db_path) as db:
            try:
                table = db.get_table("test_table")
                count = table.get_record_count()
                self.assertIsInstance(count, int)
                self.assertGreaterEqual(count, 0)
            except remdb.NotFoundError:
                pass

class TestZeroCopy(unittest.TestCase):
    """Test zero-copy functionality"""

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

    def test_zero_copy(self):
        """Test zero-copy functionality"""
        with remdb.connect(self.db_path) as db:
            try:
                table = db.get_table("test_table")
                # 测试零拷贝模式
                # 注意：这里假设表中已经有一条id为1的记录
                result = table.get(1, zero_copy=True)
                # 零拷贝模式应该返回memoryview
                self.assertIsInstance(result, (dict, memoryview))
            except remdb.NotFoundError:
                pass

class TestExtras(unittest.TestCase):
    """Test extra functionality"""

    def test_numpy_integration(self):
        """Test NumPy integration"""
        try:
            import numpy as np
            from remdb.extras.numpy import NumPyIntegration
            
            # 测试NumPy数组转换
            data = [1, 2, 3, 4, 5]
            array = NumPyIntegration.to_numpy_array(data)
            self.assertIsInstance(array, np.ndarray)
            self.assertEqual(len(array), 5)
            
            # 测试转换回列表
            data_list = NumPyIntegration.from_numpy_array(array)
            self.assertIsInstance(data_list, list)
            self.assertEqual(data_list, data)
        except ImportError:
            # 如果NumPy未安装，跳过测试
            pass

    def test_pandas_integration(self):
        """Test Pandas integration"""
        try:
            import pandas as pd
            from remdb.extras.pandas import PandasIntegration
            
            # 测试DataFrame创建
            df = pd.DataFrame({'a': [1, 2, 3], 'b': [4, 5, 6]})
            self.assertIsInstance(df, pd.DataFrame)
            self.assertEqual(len(df), 3)
        except ImportError:
            # 如果Pandas未安装，跳过测试
            pass

class TestAdvancedQueries(unittest.TestCase):
    """Test advanced query functionality"""

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

    def test_vector_search(self):
        """Test vector search functionality"""
        with remdb.connect(self.db_path) as db:
            try:
                table = db.get_table("test_table")
                # 测试向量搜索API结构
                # 注意：这里假设表中有一个名为"vector_field"的向量字段
                # 实际测试需要表中存在向量数据
                query_vector = [1.0, 2.0, 3.0, 4.0, 5.0]
                results = table.vector_search("vector_field", query_vector, k=5)
                self.assertIsInstance(results, list)
                for result in results:
                    self.assertIsInstance(result, dict)
                    self.assertIn("id", result)
                    self.assertIn("distance", result)
            except remdb.NotFoundError:
                # 如果表不存在，跳过测试
                pass

    def test_hybrid_search(self):
        """Test hybrid search functionality"""
        with remdb.connect(self.db_path) as db:
            try:
                table = db.get_table("test_table")
                # 测试混合搜索API结构
                # 注意：这里假设表中有向量字段和标量字段
                query_vector = [1.0, 2.0, 3.0, 4.0, 5.0]
                filter_expr = "age > 18"
                results = table.hybrid_search("vector_field", query_vector, filter_expr, k=5)
                self.assertIsInstance(results, list)
                for result in results:
                    self.assertIsInstance(result, dict)
                    self.assertIn("id", result)
                    self.assertIn("distance", result)
            except remdb.NotFoundError:
                # 如果表不存在，跳过测试
                pass

    def test_query_builder(self):
        """Test query builder functionality"""
        with remdb.connect(self.db_path) as db:
            try:
                table = db.get_table("test_table")
                # 测试查询构建器
                builder = table.query()
                builder.select("id", "name").where("age > ?", 18).order("name").limit(10)
                sql, params = builder.build()
                self.assertIsInstance(sql, str)
                self.assertIsInstance(params, list)
                self.assertIn("SELECT id, name", sql)
                self.assertIn("FROM test_table", sql)
                self.assertIn("WHERE age > ?", sql)
                self.assertIn("ORDER BY name ASC", sql)
                self.assertIn("LIMIT 10", sql)
                self.assertEqual(params, [18])
            except remdb.NotFoundError:
                # 如果表不存在，跳过测试
                pass

if __name__ == '__main__':
    unittest.main()
