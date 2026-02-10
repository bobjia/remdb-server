"""Core Python API for RemDB"""

import sys
import os

# 添加项目根目录到Python路径
sys.path.insert(0, os.path.abspath(os.path.dirname(os.path.dirname(__file__))))

import _remdb
import contextlib
import datetime
from typing import Dict, List, Optional, Any, Union, Tuple

# 导入网络模块
from .network import JdbcClient, parse_jdbc_url, JdbcClientError

# 异常类
class RemDbError(Exception):
    """Base exception class for RemDB errors"""
    pass

class NotFoundError(RemDbError):
    """Record not found error"""
    pass

class TransactionError(RemDbError):
    """Transaction error"""
    pass

class ConfigError(RemDbError):
    """Configuration error"""
    pass

# JSON值包装类
class JsonValue:
    """Wrapper class for JSON values to provide type hints for the C API"""
    
    def __init__(self, value):
        """
        Initialize a JSON value wrapper
        
        Args:
            value: Python object to be serialized as JSON (dict, list, etc.)
        """
        self.value = value
    
    def __str__(self):
        """Convert to JSON string"""
        import json
        return json.dumps(self.value)
    
    def __repr__(self):
        return f"JsonValue({repr(self.value)})"

# 数据库连接类
class RemDbConnection:
    """Database connection class with context manager support"""

    def __init__(self, db_path: str = ""):
        """
        Initialize a database connection

        Args:
            db_path: Path to the database file (e.g., "path/to/database.rdb") or JDBC URL (e.g., "jdbc://host:port/database")
        """
        self.db_path = db_path
        self.connected = False
        self.is_network_connection = False
        
        # 判断连接类型
        if db_path.startswith("jdbc://"):
            # 网络连接
            self.is_network_connection = True
            self.jdbc_client = None
            parsed_url = parse_jdbc_url(db_path)
            self.host = parsed_url["host"]
            self.port = parsed_url["port"]
            self.database = parsed_url["database"]
        else:
            # 本地文件连接
            self.is_network_connection = False
            self.db = _remdb.RemDb()

    def __enter__(self):
        """Enter context manager"""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager"""
        self.close()

    def connect(self):
        """
        Connect to the database
        """
        if not self.connected:
            if self.is_network_connection:
                # 网络连接
                try:
                    self.jdbc_client = JdbcClient(self.host, self.port, database=self.database)
                    self.connected = self.jdbc_client.connect()
                    if not self.connected:
                        raise RemDbError(f"Failed to connect to database server: {self.host}:{self.port}")
                except Exception as e:
                    raise RemDbError(f"Failed to connect to database server: {e}")
            else:
                # 本地文件连接
                self.connected = self.db.connect(self.db_path)
                if not self.connected:
                    raise RemDbError(f"Failed to connect to database: {self.db_path}")
        return self.connected

    def close(self):
        """Close the database connection"""
        # 清理资源
        if self.is_network_connection and hasattr(self, 'jdbc_client') and self.jdbc_client:
            try:
                self.jdbc_client.disconnect()
            except Exception:
                pass
        elif not self.is_network_connection and hasattr(self, 'db') and self.db:
            try:
                if hasattr(self.db, 'close'):
                    self.db.close()
            except Exception:
                pass
        self.connected = False

    def get_table(self, table_name: str) -> "RemDbTable":
        """
        Get a table by name

        Args:
            table_name: Name of the table

        Returns:
            RemDbTable instance or NetworkTableAdapter instance

        Raises:
            NotFoundError: If the table does not exist
        """
        if not self.connected:
            raise RemDbError("Not connected to database")

        if self.is_network_connection:
            # 网络连接
            try:
                # 验证表是否存在
                sql = f"SELECT * FROM {table_name} LIMIT 1"
                self.jdbc_client.execute_query(sql)
                # 返回网络表适配器
                return RemDbTable(NetworkTableAdapter(table_name, self), table_name, self)
            except Exception as e:
                raise NotFoundError(f"Table not found: {table_name}")
        else:
            # 本地文件连接
            table = self.db.get_table(table_name)
            if not table:
                raise NotFoundError(f"Table not found: {table_name}")
            return RemDbTable(table, table_name, self)

    def begin_transaction(self) -> "RemDbTransaction":
        """
        Begin a transaction

        Returns:
            RemDbTransaction instance
        """
        if not self.connected:
            raise RemDbError("Not connected to database")

        if self.is_network_connection:
            # 网络连接
            try:
                transaction_id = self.jdbc_client.begin_transaction()
                # 创建网络事务适配器
                return NetworkTransactionAdapter(transaction_id, self.jdbc_client, self)
            except Exception as e:
                raise TransactionError(f"Failed to begin transaction: {e}")
        else:
            # 本地文件连接
            tx = self.db.begin_transaction()
            if not tx:
                raise TransactionError("Failed to begin transaction")
            return RemDbTransaction(tx, self)

    def execute_query(self, sql: str) -> "RemDbResultSet":
        """
        Execute SQL query

        Args:
            sql: SQL query string

        Returns:
            RemDbResultSet instance
        """
        if not self.connected:
            raise RemDbError("Not connected to database")

        # 处理 COUNT(*) 查询
        import re
        count_match = re.search(r'SELECT\s+COUNT\(\*\)\s+FROM\s+([a-zA-Z0-9_]+)', sql, re.IGNORECASE)
        if count_match and not self.is_network_connection:
            # 提取表名
            table_name = count_match.group(1)
            try:
                # 获取表实例
                table = self.get_table(table_name)
                # 获取实际记录数
                count = table.get_record_count()
                # 创建一个模拟的结果集
                class MockResultSet:
                    def get_columns(self):
                        return ["COUNT(*)"]
                    def get_rows_count(self):
                        return 1
                    def get_row(self, index):
                        if index == 0:
                            return {"COUNT(*)": count}
                        raise IndexError("Row index out of range")
                return RemDbResultSet(MockResultSet())
            except Exception:
                # 如果失败，回退到原始SQL执行
                pass

        if self.is_network_connection:
            # 网络连接
            try:
                result = self.jdbc_client.execute_query(sql)
                # 创建一个适配器，将网络查询结果转换为RemDbResultSet
                return NetworkResultSetAdapter(result)
            except Exception as e:
                raise RemDbError(f"Failed to execute query: {e}")
        else:
            # 本地文件连接
            result_set = self.db.execute_query(sql)
            if not result_set:
                raise RemDbError(f"Failed to execute query: {sql}")
            return RemDbResultSet(result_set)

    def save_snapshot(self, path: str) -> bool:
        """
        Save database snapshot

        Args:
            path: Path to save the snapshot

        Returns:
            True if successful, False otherwise
        """
        if not self.connected:
            raise RemDbError("Not connected to database")

        if self.is_network_connection:
            # 网络连接
            # 注意：网络连接的快照操作可能需要不同的实现
            # 这里简化处理，返回False表示不支持
            return False
        else:
            # 本地文件连接
            return self.db.save_snapshot(path)

    def restore_snapshot(self, path: str) -> bool:
        """
        Restore from snapshot

        Args:
            path: Path to the snapshot file

        Returns:
            True if successful, False otherwise
        """
        if not self.connected:
            raise RemDbError("Not connected to database")

        if self.is_network_connection:
            # 网络连接
            # 注意：网络连接的快照操作可能需要不同的实现
            # 这里简化处理，返回False表示不支持
            return False
        else:
            # 本地文件连接
            return self.db.restore_snapshot(path)

# 表操作类
class RemDbTable:
    """Table operation class"""

    def __init__(self, table, table_name: str, connection: RemDbConnection):
        """
        Initialize a table instance

        Args:
            table: Underlying table object
            table_name: Name of the table
            connection: Database connection
        """
        self.table = table
        self.table_name = table_name
        self.connection = connection

    def insert(self, record: Union[Dict[str, Any], List[Any]]) -> bool:
        """
        Insert a record into the table

        Args:
            record: Dictionary or list of values

        Returns:
            True if successful, False otherwise
        """
        # 确保记录格式正确
        if not record:
            return False
        
        # 检查是否是网络连接
        is_network_connection = hasattr(self.table, 'table_name')
        
        if isinstance(record, dict):
            if is_network_connection:
                # 对于网络连接，保留原始类型
                return self.table.insert(record)
            else:
                # 对于本地连接，将字典中的值转换为适当的类型
                str_record = {}
                for k, v in record.items():
                    if isinstance(v, (dict, list)):
                        # 对于JSON类型，转换为JSON字符串
                        import json
                        str_record[k] = json.dumps(v)
                    else:
                        # 其他类型转换为字符串
                        str_record[k] = str(v)
                return self.table.insert(str_record)
        elif isinstance(record, list):
            if is_network_connection:
                # 对于网络连接，保留原始类型
                return self.table.insert(record)
            else:
                # 对于本地连接，将列表中的值转换为字符串
                str_record = []
                for item in record:
                    if isinstance(item, (dict, list)):
                        # 对于JSON类型，转换为JSON字符串
                        import json
                        str_record.append(json.dumps(item))
                    else:
                        # 其他类型转换为字符串
                        str_record.append(str(item))
                return self.table.insert(str_record)
        
        return False

    def get(self, key: Any, zero_copy: bool = False) -> Optional[Union[Dict[str, Any], memoryview]]:
        """
        Get a record by key

        Args:
            key: Primary key value
            zero_copy: Whether to use zero-copy mode

        Returns:
            Dictionary of record values, memoryview (if zero_copy=True), or None if not found
        """
        if zero_copy:
            # 使用零拷贝模式
            try:
                zero_copy_data = self.table.get_zero_copy(str(key))
                # 转换为memoryview
                return memoryview(zero_copy_data.tobytes())
            except Exception:
                # 如果零拷贝失败，回退到普通模式
                pass
        
        # 普通模式
        record = []
        success = self.table.get(str(key), record)
        if not success:
            return None
        # 解析record为字典
        # 注意：这里假设record的第一个元素是id，后续元素是其他字段值
        # 实际应用中需要根据表结构进行解析
        result = {"id": record[0]} if record else {}
        
        # 尝试解析JSON值
        for k, v in result.items():
            if isinstance(v, str):
                # 尝试将字符串解析为JSON
                try:
                    import json
                    parsed_value = json.loads(v)
                    result[k] = parsed_value
                except (json.JSONDecodeError, ValueError):
                    # 如果不是有效的JSON，保持原始字符串
                    pass
        
        return result

    def get_by_id(self, key: Any, zero_copy: bool = False) -> Optional[Union[Dict[str, Any], memoryview]]:
        """
        Get a record by ID

        Args:
            key: Primary key value
            zero_copy: Whether to use zero-copy mode

        Returns:
            Dictionary of record values, memoryview (if zero_copy=True), or None if not found
        """
        return self.get(key, zero_copy)

    def update(self, key: Any, record: Union[Dict[str, Any], List[Any]]) -> bool:
        """
        Update a record by key

        Args:
            key: Primary key value
            record: Dictionary or list of values

        Returns:
            True if successful, False otherwise
        """
        # 确保记录格式正确
        if not record:
            return False
        
        # 检查是否是网络连接
        is_network_connection = hasattr(self.table, 'table_name')
        
        # 转换记录格式以匹配C++实现的期望
        if isinstance(record, dict):
            if is_network_connection:
                # 对于网络连接，保留原始类型
                return self.table.update(key, record)
            else:
                # 对于本地连接，将字典中的值转换为适当的类型
                str_record = {}
                for k, v in record.items():
                    if isinstance(v, (dict, list)):
                        # 对于JSON类型，转换为JSON字符串
                        import json
                        str_record[k] = json.dumps(v)
                    else:
                        # 其他类型转换为字符串
                        str_record[k] = str(v)
                return self.table.update(str(key), str_record)
        elif isinstance(record, list):
            if is_network_connection:
                # 对于网络连接，保留原始类型
                return self.table.update(key, record)
            else:
                # 对于本地连接，将列表中的值转换为字符串
                str_record = []
                for item in record:
                    if isinstance(item, (dict, list)):
                        # 对于JSON类型，转换为JSON字符串
                        import json
                        str_record.append(json.dumps(item))
                    else:
                        # 其他类型转换为字符串
                        str_record.append(str(item))
                return self.table.update(str(key), str_record)
        
        return False

    def delete(self, key: Any) -> bool:
        """
        Delete a record by key

        Args:
            key: Primary key value

        Returns:
            True if successful, False otherwise
        """
        return self.table.delete_record(str(key))

    def batch_insert(self, records: List[Union[Dict[str, Any], List[Any]]]) -> bool:
        """
        Batch insert multiple records

        Args:
            records: List of dictionaries or lists of values

        Returns:
            True if all records were inserted successfully, False otherwise
        """
        if not records:
            return True
        
        # 检查是否是网络连接
        is_network_connection = hasattr(self.table, 'table_name')
        
        all_success = True
        for record in records:
            if not self.insert(record):
                all_success = False
        return all_success

    def batch_update(self, updates: List[Tuple[Any, Union[Dict[str, Any], List[Any]]]]) -> bool:
        """
        Batch update multiple records

        Args:
            updates: List of tuples (key, record)

        Returns:
            True if all records were updated successfully, False otherwise
        """
        if not updates:
            return True
        
        all_success = True
        for key, record in updates:
            if not self.update(key, record):
                all_success = False
        return all_success

    def batch_delete(self, keys: List[Any]) -> bool:
        """
        Batch delete multiple records

        Args:
            keys: List of primary key values

        Returns:
            True if all records were deleted successfully, False otherwise
        """
        if not keys:
            return True
        
        all_success = True
        for key in keys:
            if not self.delete(key):
                all_success = False
        return all_success

    def get_record_count(self) -> int:
        """
        Get the number of records in the table

        Returns:
            Number of records
        """
        return self.table.get_record_count()

    def get_column_as_numpy(self, column_name: str, dtype: Any = None):
        """
        Get a column as NumPy array (zero-copy if possible)

        Args:
            column_name: Name of the column
            dtype: NumPy data type

        Returns:
            NumPy array
        """
        try:
            import numpy as np
        except ImportError:
            raise ImportError("NumPy is required for this function. Please install it with 'pip install numpy'")
        
        # 使用C扩展中的方法
        numpy_array = self.table.get_column_as_numpy(column_name)
        # 如果指定了数据类型，进行转换
        if dtype is not None:
            return numpy_array.astype(dtype)
        return numpy_array

    def insert_from_dataframe(self, dataframe, batch_size: int = 1000):
        """
        Insert data from pandas DataFrame

        Args:
            dataframe: pandas DataFrame
            batch_size: Batch size for insertion
        """
        from .extras.pandas import PandasIntegration
        PandasIntegration.insert_from_dataframe(self, dataframe, batch_size)

    def to_dataframe(self, columns: Optional[List[str]] = None):
        """
        Convert table to pandas DataFrame

        Args:
            columns: List of columns to include

        Returns:
            pandas DataFrame
        """
        from .extras.pandas import PandasIntegration
        return PandasIntegration.to_dataframe_from_table(self, columns)

    def vector_search(self, field_name: str, query_vector: List[float], k: int = 10) -> List[Dict[str, Any]]:
        """
        Perform vector similarity search

        Args:
            field_name: Name of the vector field
            query_vector: Query vector as a list of floats
            k: Number of nearest neighbors to return

        Returns:
            List of dictionaries with 'id' and 'distance' keys
        """
        # 确保参数有效
        if not field_name or not query_vector or k <= 0:
            return []
        
        # 调用底层实现
        results = self.table.vector_search(field_name, query_vector, k)
        # 转换结果为Python字典列表
        return [{'id': id, 'distance': distance} for id, distance in results]

    def hybrid_search(self, field_name: str, query_vector: List[float], filter_expr: str, k: int = 10) -> List[Dict[str, Any]]:
        """
        Perform hybrid search (vector + scalar filtering)

        Args:
            field_name: Name of the vector field
            query_vector: Query vector as a list of floats
            filter_expr: SQL WHERE clause for scalar filtering
            k: Number of nearest neighbors to return

        Returns:
            List of dictionaries with 'id' and 'distance' keys
        """
        # 确保参数有效
        if not field_name or not query_vector or not filter_expr or k <= 0:
            return []
        
        try:
            # 1. 首先执行向量搜索获取前k*2个结果（留出过滤空间）
            results = self.table.vector_search(field_name, query_vector, k * 2)
            
            # 2. 提取ID列表
            ids = [id for id, _ in results]
            if not ids:
                return []
            
            # 3. 构建SQL查询，应用过滤条件
            ids_str = ','.join(map(str, ids))
            sql = f"SELECT id FROM {self.table_name} WHERE id IN ({ids_str}) AND {filter_expr}"
            
            # 4. 执行查询获取符合过滤条件的ID
            result_set = self.connection.execute_query(sql)
            filtered_ids = {row['id'] for row in result_set}
            
            # 5. 过滤向量搜索结果
            filtered_results = [(id, distance) for id, distance in results if id in filtered_ids]
            
            # 6. 截取前k个结果
            filtered_results = filtered_results[:k]
            
            # 7. 转换结果为Python字典列表
            return [{'id': id, 'distance': distance} for id, distance in filtered_results]
        except Exception as e:
            # 处理异常
            return []

    def query(self) -> 'QueryBuilder':
        """
        Create a query builder for this table

        Returns:
            QueryBuilder instance
        """
        return QueryBuilder(self.table_name)

# 事务管理类
class RemDbTransaction:
    """Transaction management class"""

    def __init__(self, transaction, connection: RemDbConnection):
        """
        Initialize a transaction instance

        Args:
            transaction: Underlying transaction object
            connection: Database connection
        """
        self.transaction = transaction
        self.connection = connection
        self.active = True

    def __enter__(self):
        """Enter context manager"""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager"""
        if exc_type is None:
            self.commit()
        else:
            self.rollback()

    def commit(self) -> bool:
        """
        Commit the transaction

        Returns:
            True if successful, False otherwise
        """
        if self.active:
            success = self.transaction.commit()
            self.active = False
            return success
        return False

    def rollback(self) -> bool:
        """
        Rollback the transaction

        Returns:
            True if successful, False otherwise
        """
        if self.active:
            success = self.transaction.rollback()
            self.active = False
            return success
        return False

    def is_active(self) -> bool:
        """
        Check if the transaction is active

        Returns:
            True if active, False otherwise
        """
        return self.active

# 网络事务适配器类
class NetworkTransactionAdapter:
    """
    Adapter for network transactions to match RemDbTransaction interface
    """

    def __init__(self, transaction_id: int, jdbc_client, connection):
        """
        Initialize a network transaction adapter

        Args:
            transaction_id: Transaction ID
            jdbc_client: JDBC client instance
            connection: Database connection
        """
        self.transaction_id = transaction_id
        self.jdbc_client = jdbc_client
        self.connection = connection
        self.active = True

    def __enter__(self):
        """
        Enter context manager
        """
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """
        Exit context manager
        """
        if exc_type is None:
            self.commit()
        else:
            self.rollback()

    def commit(self) -> bool:
        """
        Commit the transaction

        Returns:
            True if successful, False otherwise
        """
        if self.active:
            try:
                success = self.jdbc_client.commit_transaction()
                self.active = False
                return success
            except Exception:
                return False
        return False

    def rollback(self) -> bool:
        """
        Rollback the transaction

        Returns:
            True if successful, False otherwise
        """
        if self.active:
            try:
                success = self.jdbc_client.rollback_transaction()
                self.active = False
                return success
            except Exception:
                return False
        return False

    def is_active(self) -> bool:
        """
        Check if the transaction is active

        Returns:
            True if active, False otherwise
        """
        return self.active

# 网络表适配器类
class NetworkTableAdapter:
    """
    Adapter for network tables to match RemDbTable interface
    """

    def __init__(self, table_name: str, connection):
        """
        Initialize a network table adapter

        Args:
            table_name: Table name
            connection: Database connection
        """
        self.table_name = table_name
        self.connection = connection

    def insert(self, record: Union[Dict[str, Any], List[Any]]) -> bool:
        """
        Insert a record

        Args:
            record: Dictionary or list of values

        Returns:
            True if successful, False otherwise
        """
        try:
            if isinstance(record, dict):
                # 构建INSERT语句
                columns = list(record.keys())
                values = list(record.values())
                
                # 构建SQL语句，直接插入值
                cols_str = ", ".join(columns)
                # 为不同类型的值添加适当的引号
                def format_value(v):
                    if isinstance(v, str):
                        # 检查是否是JSON字符串（以{或[开头）
                        if v.startswith('{') or v.startswith('['):
                            # JSON字符串不需要引号
                            return v
                        else:
                            # 普通字符串需要引号
                            return f"'{v}'"
                    elif isinstance(v, bool):
                        return str(v).lower()
                    elif v is None:
                        return "NULL"
                    # 对于数字类型，转换为字符串但不添加引号
                    elif isinstance(v, (int, float)):
                        return str(v)
                    else:
                        return str(v)
                
                values_str = ", ".join([format_value(v) for v in values])
                sql = f"INSERT INTO {self.table_name} ({cols_str}) VALUES ({values_str})"
                
                # 执行查询，检查结果
                result = self.connection.jdbc_client.execute_query(sql)
                

                
                # 检查执行结果
                if "affected_rows" in result:
                    # 这是UPDATE/DELETE/INSERT操作的结果
                    affected = result.get("affected_rows", 0)
                    return affected > 0
                elif "last_insert_id" in result:
                    # 有last_insert_id，说明插入成功
                    return True
                elif "rows" in result:
                    # 检查是否是空的result_set（可能表示INSERT成功）
                    rows = result.get("rows", [])
                    # 如果是空列表，可能表示INSERT成功但没有返回数据
                    if rows == []:
                        # 对于INSERT，空的result_set可能表示成功
                        return True
                    # 否则是SELECT查询的结果
                    # 对于INSERT，不应该返回非空行
                    return False
                else:
                    # 没有返回结果，检查是否有其他指示成功的字段
                    # 有时服务器可能返回空字典表示成功
                    if result == {}:
                        return True
                    # 或者检查是否有错误信息
                    if "error_message" in result:
                        return False
                    # 默认假设成功
                    return True
            else:
                # 对于列表类型，需要知道列名，这里简化处理
                # 将列表作为整体插入，不拆分为单独的值
                # 构建INSERT语句，直接插入列表值
                columns = list(record.keys())
                values_str = str(record)  # 直接使用字符串表示
                sql = f"INSERT INTO {self.table_name} ({', '.join(columns)}) VALUES ({values_str})"
                
                # 执行查询，检查结果
                result = self.connection.jdbc_client.execute_query(sql)
                
                # 检查执行结果
                if "affected_rows" in result:
                    # 这是UPDATE/DELETE/INSERT操作的结果
                    affected = result.get("affected_rows", 0)
                    return affected > 0
                elif "last_insert_id" in result:
                    # 有last_insert_id，说明插入成功
                    return True
                elif "rows" in result:
                    # 检查是否是空的result_set（可能表示INSERT成功）
                    rows = result.get("rows", [])
                    # 如果是空列表，可能表示INSERT成功但没有返回数据
                    if rows == []:
                        # 对于INSERT，空的result_set可能表示成功
                        return True
                    # 否则是SELECT查询的结果
                    # 对于INSERT，不应该返回非空行
                    return False
                else:
                    # 没有返回结果，检查是否有其他指示成功的字段
                    # 有时服务器可能返回空字典表示成功
                    if result == {}:
                        return True
                    # 或者检查是否有错误信息
                    if "error_message" in result:
                        return False
                    # 默认假设成功
                    return True
        except Exception:
            return False

    def get(self, key: Any, zero_copy: bool = False) -> Optional[Dict[str, Any]]:
        """
        Get a record by key

        Args:
            key: Primary key value
            zero_copy: Whether to use zero-copy mode

        Returns:
            Dictionary of record values or None if not found
        """
        try:
            # 构建SELECT语句，直接插入值而不是使用参数占位符
            # 为key添加适当的引号
            def format_key(k):
                if isinstance(k, str):
                    return f"'{k}'"
                elif isinstance(k, bool):
                    return str(k).lower()
                elif k is None:
                    return "NULL"
                # 对于数字类型，直接返回字符串表示，不加引号
                elif isinstance(k, (int, float)):
                    return str(k)
                else:
                    return str(k)
            
            # 尝试不同的主键列名
            primary_key_columns = ["id", "ID", "Id"]
            
            for pk_col in primary_key_columns:
                try:
                    sql = f"SELECT * FROM {self.table_name} WHERE {pk_col} = {format_key(key)}"
                    result = self.connection.jdbc_client.execute_query(sql)
                    
                    # 检查结果
                    if result.get("rows") and len(result.get("rows")) > 0:
                        row = result.get("rows")[0]
                        columns = result.get("columns")
                        
                        # 验证列名和行数据长度匹配
                        if columns and len(columns) == len(row):
                            return dict(zip(columns, row))
                        elif columns and len(columns) != len(row):
                            # 列数和行数据不匹配，创建默认映射
                            return {f"col_{i}": row[i] for i in range(len(row))}
                        else:
                            # 没有列名信息，使用默认列名
                            return {f"column_{i}": row[i] for i in range(len(row))}
                    
                    # 如果找到结果，返回它；否则继续尝试下一个列名
                    if result.get("rows") and len(result.get("rows")) > 0:
                        break
                        
                except Exception:
                    # 当前列名查询失败，尝试下一个
                    continue
            
            # 如果所有列名都失败，返回None
            return None
        except Exception:
            return None

    def get_by_id(self, key: Any, zero_copy: bool = False) -> Optional[Dict[str, Any]]:
        """
        Get a record by ID

        Args:
            key: Primary key value
            zero_copy: Whether to use zero-copy mode

        Returns:
            Dictionary of record values or None if not found
        """
        return self.get(key, zero_copy)

    def update(self, key: Any, record: Union[Dict[str, Any], List[Any]]) -> bool:
        """
        Update a record by key

        Args:
            key: Primary key value
            record: Dictionary or list of values

        Returns:
            True if successful, False otherwise
        """
        try:
            if isinstance(record, dict):
                # 构建UPDATE语句，直接插入值
                # 为不同类型的值添加适当的引号
                def format_value(v):
                    if isinstance(v, str):
                        return f"'{v}'"
                    elif isinstance(v, bool):
                        return str(v).lower()
                    elif v is None:
                        return "NULL"
                    # 对于数字类型，转换为字符串但不添加引号
                    elif isinstance(v, (int, float)):
                        return str(v)
                    else:
                        return str(v)
                
                # 为key添加适当的引号
                def format_key(k):
                    if isinstance(k, str):
                        return f"'{k}'"
                    elif isinstance(k, bool):
                        return str(k).lower()
                    elif k is None:
                        return "NULL"
                    else:
                        return str(k)
                
                # 尝试不同的主键列名
                primary_key_columns = ["id", "ID", "Id"]
                success = False
                
                for pk_col in primary_key_columns:
                    try:
                        set_clause = ", ".join([f"{col} = {format_value(v)}" for col, v in record.items()])
                        sql = f"UPDATE {self.table_name} SET {set_clause} WHERE {pk_col} = {format_key(key)}"
                        
                        # 执行查询，检查结果
                        result = self.connection.jdbc_client.execute_query(sql)
                        
                        # 检查执行结果
                        if "affected_rows" in result:
                            affected = result.get("affected_rows", 0)
                            if affected > 0:
                                success = True
                                break
                        elif "rows" in result:
                            # 检查是否是空的result_set（可能表示UPDATE成功）
                            rows = result.get("rows", [])
                            if rows == []:
                                # 对于UPDATE，空的result_set可能表示成功
                                success = True
                                break
                            # 否则是SELECT查询的结果，不是UPDATE
                            continue
                        else:
                            # 没有返回结果，检查是否有其他指示成功的字段
                            if result == {}:
                                success = True
                                break
                            # 或者检查是否有错误信息
                            if "error_message" in result:
                                continue
                            # 默认假设成功
                            success = True
                            break
                    except Exception:
                        # 当前列名查询失败，尝试下一个
                        continue
                
                return success
            else:
                # 对于列表类型，需要知道列名，这里简化处理
                return False
        except Exception:
            return False

    def delete_record(self, key: Any) -> bool:
        """
        Delete a record by key

        Args:
            key: Primary key value

        Returns:
            True if successful, False otherwise
        """
        try:
            # 构建DELETE语句，直接插入值
            def format_key(k):
                if isinstance(k, str):
                    return f"'{k}'"
                elif isinstance(k, bool):
                    return str(k).lower()
                elif k is None:
                    return "NULL"
                else:
                    return str(k)
            
            # 尝试不同的主键列名
            primary_key_columns = ["id", "ID", "Id"]
            success = False
            
            for pk_col in primary_key_columns:
                try:
                    sql = f"DELETE FROM {self.table_name} WHERE {pk_col} = {format_key(key)}"
                    
                    # 执行查询，检查结果
                    result = self.connection.jdbc_client.execute_query(sql)
                    
                    # 检查执行结果
                    if "affected_rows" in result:
                        affected = result.get("affected_rows", 0)
                        if affected > 0:
                            success = True
                            break
                    elif "rows" in result:
                        # 检查是否是空的result_set（可能表示DELETE成功）
                        rows = result.get("rows", [])
                        if rows == []:
                            # 对于DELETE，空的result_set可能表示成功
                            success = True
                            break
                        # 否则是SELECT查询的结果，不是DELETE
                        continue
                    else:
                        # 没有返回结果，检查是否有其他指示成功的字段
                        if result == {}:
                            success = True
                            break
                        # 或者检查是否有错误信息
                        if "error_message" in result:
                            continue
                        # 默认假设成功
                        success = True
                        break
                except Exception:
                    # 当前列名查询失败，尝试下一个
                    continue
            
            return success
        except Exception:
            return False
            
    def delete(self, key: Any) -> bool:
        """
        Delete a record by key

        Args:
            key: Primary key value

        Returns:
            True if successful, False otherwise
        """
        return self.delete_record(key)

    def batch_insert(self, records: List[Union[Dict[str, Any], List[Any]]]) -> bool:
        """
        Batch insert multiple records

        Args:
            records: List of dictionaries or lists of values

        Returns:
            True if all records were inserted successfully, False otherwise
        """
        if not records:
            return True
        
        all_success = True
        for record in records:
            if not self.insert(record):
                all_success = False
        return all_success

    def batch_update(self, updates: List[Tuple[Any, Union[Dict[str, Any], List[Any]]]]) -> bool:
        """
        Batch update multiple records

        Args:
            updates: List of tuples (key, record)

        Returns:
            True if all records were updated successfully, False otherwise
        """
        if not updates:
            return True
        
        all_success = True
        for key, record in updates:
            if not self.update(key, record):
                all_success = False
        return all_success

    def batch_delete(self, keys: List[Any]) -> bool:
        """
        Batch delete multiple records

        Args:
            keys: List of primary key values

        Returns:
            True if all records were deleted successfully, False otherwise
        """
        if not keys:
            return True
        
        all_success = True
        for key in keys:
            if not self.delete(key):
                all_success = False
        return all_success

    def get_record_count(self) -> int:
        """
        Get number of records

        Returns:
            Number of records
        """
        try:
            # 构建COUNT语句
            sql = f"SELECT COUNT(*) FROM {self.table_name}"
            result = self.connection.jdbc_client.execute_query(sql)
            
            # 解析结果
            if result.get("rows") and len(result.get("rows")) > 0:
                return result.get("rows")[0][0]
            return 0
        except Exception:
            return 0

    def get_column_as_numpy(self, column_name: str, dtype: Any = None):
        """
        Get column as NumPy array

        Args:
            column_name: Name of the column
            dtype: NumPy data type

        Returns:
            NumPy array
        """
        try:
            import numpy as np
            
            # 构建SELECT语句
            sql = f"SELECT {column_name} FROM {self.table_name}"
            result = self.connection.jdbc_client.execute_query(sql)
            
            # 解析结果
            if result.get("rows"):
                values = [row[0] for row in result.get("rows")]
                array = np.array(values, dtype=dtype)
                return array
            return np.array([], dtype=dtype)
        except Exception:
            return None

    def insert_from_dataframe(self, dataframe, batch_size: int = 1000):
        """
        Insert data from pandas DataFrame

        Args:
            dataframe: pandas DataFrame
            batch_size: Batch size for insertion
        """
        try:
            # 构建INSERT语句
            columns = list(dataframe.columns)
            cols_str = ", ".join(columns)
            placeholders = ", ".join(["?"] * len(columns))
            sql = f"INSERT INTO {self.table_name} ({cols_str}) VALUES ({placeholders})"
            
            # 批量插入
            for i in range(0, len(dataframe), batch_size):
                batch = dataframe.iloc[i:i+batch_size]
                for _, row in batch.iterrows():
                    values = list(row)
                    self.connection.jdbc_client.execute_query(sql, values)
        except Exception:
            pass

    def to_dataframe(self, columns: Optional[List[str]] = None):
        """
        Convert table to pandas DataFrame

        Args:
            columns: List of columns to include

        Returns:
            pandas DataFrame
        """
        try:
            import pandas as pd
            
            # 构建SELECT语句
            if columns:
                cols_str = ", ".join(columns)
                sql = f"SELECT {cols_str} FROM {self.table_name}"
            else:
                sql = f"SELECT * FROM {self.table_name}"
            
            # 执行查询
            result = self.connection.jdbc_client.execute_query(sql)
            
            # 转换为DataFrame
            if result.get("rows") and result.get("columns"):
                df = pd.DataFrame(result.get("rows"), columns=result.get("columns"))
                return df
            return pd.DataFrame()
        except Exception:
            return None

    def vector_search(self, field_name: str, query_vector: List[float], k: int = 10) -> List[Dict[str, Any]]:
        """
        Perform vector similarity search

        Args:
            field_name: Name of the vector field
            query_vector: Query vector as a list of floats
            k: Number of nearest neighbors to return

        Returns:
            List of dictionaries with 'id' and 'distance' keys
        """
        try:
            # 构建向量搜索SQL语句
            # 注意：这里假设RemDB支持向量搜索的SQL语法
            sql = f"SELECT id, VECTOR_DISTANCE({field_name}, ?) as distance FROM {self.table_name} ORDER BY distance LIMIT {k}"
            result = self.connection.jdbc_client.execute_query(sql, [query_vector])
            
            # 解析结果
            results = []
            if result.get("rows"):
                for row in result.get("rows"):
                    results.append({"id": row[0], "distance": row[1]})
            return results
        except Exception:
            return []

    def hybrid_search(self, field_name: str, query_vector: List[float], filter_expr: str, k: int = 10) -> List[Dict[str, Any]]:
        """
        Perform hybrid search (vector + scalar filtering)

        Args:
            field_name: Name of the vector field
            query_vector: Query vector as a list of floats
            filter_expr: SQL WHERE clause for scalar filtering
            k: Number of nearest neighbors to return

        Returns:
            List of dictionaries with 'id' and 'distance' keys
        """
        try:
            # 构建混合搜索SQL语句
            sql = f"SELECT id, VECTOR_DISTANCE({field_name}, ?) as distance FROM {self.table_name} WHERE {filter_expr} ORDER BY distance LIMIT {k}"
            result = self.connection.jdbc_client.execute_query(sql, [query_vector])
            
            # 解析结果
            results = []
            if result.get("rows"):
                for row in result.get("rows"):
                    results.append({"id": row[0], "distance": row[1]})
            return results
        except Exception:
            return []

    def query(self) -> "QueryBuilder":
        """
        Create a query builder for this table

        Returns:
            QueryBuilder instance
        """
        return QueryBuilder(self.table_name)

# 网络结果集适配器类
class NetworkResultSetAdapter:
    """
    Adapter for network result sets to match RemDbResultSet interface
    """

    def __init__(self, result):
        """
        Initialize a network result set adapter

        Args:
            result: Network query result dictionary
        """
        self.result = result
        self.columns = result.get("columns", [])
        self.rows = result.get("rows", [])
        self.rows_count = len(self.rows)
        self.current_row = 0

    def __iter__(self):
        """
        Iterate over rows
        """
        self.current_row = 0
        return self

    def __next__(self):
        """
        Get next row
        """
        if self.current_row >= self.rows_count:
            raise StopIteration
        row = self.get_row(self.current_row)
        self.current_row += 1
        return row

    def get_columns(self) -> List[str]:
        """
        Get column names

        Returns:
            List of column names
        """
        return self.columns

    def get_rows_count(self) -> int:
        """
        Get number of rows

        Returns:
            Number of rows
        """
        return self.rows_count

    def get_row(self, row_index: int) -> Dict[str, Any]:
        """
        Get a row by index

        Args:
            row_index: Row index

        Returns:
            Dictionary of row values
        """
        if row_index < 0 or row_index >= self.rows_count:
            raise IndexError(f"Row index out of range: {row_index}")
        
        row = self.rows[row_index]
        return dict(zip(self.columns, row))

    def to_dataframe(self):
        """
        Convert result set to pandas DataFrame

        Returns:
            pandas DataFrame
        """
        from .extras.pandas import PandasIntegration
        return PandasIntegration.to_dataframe(self)

# 结果集类
class RemDbResultSet:
    """
    Query result set class
    """

    def __init__(self, result_set):
        """
        Initialize a result set instance

        Args:
            result_set: Underlying result set object
        """
        self.result_set = result_set
        self.columns = self.result_set.get_columns()
        self.rows_count = self.result_set.get_rows_count()
        self.current_row = 0
        
        print(f"DEBUG Python RemDbResultSet.__init__: columns={self.columns}, rows_count={self.rows_count}")

    def __iter__(self):
        """Iterate over rows"""
        self.current_row = 0
        return self

    def __next__(self):
        """Get next row"""
        if self.current_row >= self.rows_count:
            raise StopIteration
        row = self.get_row(self.current_row)
        self.current_row += 1
        return row

    def get_columns(self) -> List[str]:
        """
        Get column names

        Returns:
            List of column names
        """
        return self.columns

    def get_rows_count(self) -> int:
        """
        Get number of rows

        Returns:
            Number of rows
        """
        return self.rows_count

    def get_row(self, row_index: int) -> Dict[str, Any]:
        """
        Get a row by index

        Args:
            row_index: Row index

        Returns:
            Dictionary of row values
        """
        values = self.result_set.get_row(row_index)
        return values

    def to_dataframe(self):
        """
        Convert result set to pandas DataFrame

        Returns:
            pandas DataFrame
        """
        from .extras.pandas import PandasIntegration
        return PandasIntegration.to_dataframe(self)

# 查询构建器类
class QueryBuilder:
    """
    SQL query builder to prevent SQL injection
    """

    def __init__(self, table_name: str):
        """
        Initialize a query builder

        Args:
            table_name: Name of the table
        """
        self.table_name = table_name
        self.columns = []
        self.conditions = []
        self.params = []
        self.order_by = []
        self._limit = None

    def select(self, *columns: str) -> "QueryBuilder":
        """
        Specify columns to select

        Args:
            *columns: Column names

        Returns:
            Self for method chaining
        """
        if columns:
            self.columns.extend(columns)
        else:
            self.columns.append("*")
        return self

    def where(self, condition: str, *params: Any) -> "QueryBuilder":
        """
        Add a WHERE condition

        Args:
            condition: Condition string with placeholders
            *params: Parameters for the placeholders

        Returns:
            Self for method chaining
        """
        self.conditions.append(condition)
        self.params.extend(params)
        return self

    def order(self, column: str, ascending: bool = True) -> "QueryBuilder":
        """
        Add an ORDER BY clause

        Args:
            column: Column name to order by
            ascending: Whether to sort in ascending order

        Returns:
            Self for method chaining
        """
        direction = "ASC" if ascending else "DESC"
        self.order_by.append(f"{column} {direction}")
        return self

    def limit(self, limit: int) -> "QueryBuilder":
        """
        Set a LIMIT clause

        Args:
            limit: Maximum number of rows to return

        Returns:
            Self for method chaining
        """
        self._limit = limit
        return self

    def build(self) -> tuple[str, List[Any]]:
        """
        Build the SQL query

        Returns:
            Tuple of (SQL query string, parameters list)
        """
        # Build SELECT clause
        if not self.columns:
            select_clause = "SELECT *"
        else:
            select_clause = f"SELECT {', '.join(self.columns)}"

        # Build FROM clause
        from_clause = f"FROM {self.table_name}"

        # Build WHERE clause
        where_clause = ""
        if self.conditions:
            where_clause = f"WHERE {' AND '.join(self.conditions)}"

        # Build ORDER BY clause
        order_by_clause = ""
        if self.order_by:
            order_by_clause = f"ORDER BY {', '.join(self.order_by)}"

        # Build LIMIT clause
        limit_clause = ""
        if self._limit is not None:
            limit_clause = f"LIMIT {self._limit}"

        # Combine all clauses
        sql = f"{select_clause} {from_clause} {where_clause} {order_by_clause} {limit_clause}"

        return sql.strip(), self.params

    def execute(self, connection: RemDbConnection) -> RemDbResultSet:
        """
        Execute the query

        Args:
            connection: Database connection

        Returns:
            RemDbResultSet instance
        """
        sql, params = self.build()
        # 执行查询
        return connection.execute_query(sql)

# 订阅发布相关类
class RemDbPubSub:
    """
    RemDB Publish/Subscribe system
    """

    def __init__(self, config=None):
        """
        Initialize PubSub system

        Args:
            config: PubSub configuration dict
        """
        if config is None:
            config = {}
        
        # 默认配置
        self.config = {
            "udp_mode": config.get("udp_mode", "unicast"),
            "multicast_addr": config.get("multicast_addr", "239.0.0.1"),
            "port": config.get("port", 5555),
            "buffer_size": config.get("buffer_size", 8192),
            "max_topics": config.get("max_topics", 100),
            "max_subscribers_per_topic": config.get("max_subscribers_per_topic", 10),
            "enable_nack": config.get("enable_nack", True),
            "retransmit_timeout": config.get("retransmit_timeout", 1000),
            "max_retransmits": config.get("max_retransmits", 3)
        }
        
        # 初始化订阅者管理器
        self.subscribers = {}
        self.next_subscription_id = 1
        
        # 主题ID映射
        self.topic_map = {}
        self.next_topic_id = 1

    def subscribe(self, topic, callback):
        """
        Subscribe to a topic

        Args:
            topic: Topic name or wildcard "*"
            callback: Callback function to receive messages

        Returns:
            Subscription ID
        """
        # 获取或创建主题ID
        if topic == "*":
            topic_id = 0xFFFF  # 通配符主题ID
        else:
            if topic not in self.topic_map:
                self.topic_map[topic] = self.next_topic_id
                self.next_topic_id += 1
            topic_id = self.topic_map[topic]
        
        # 生成订阅ID
        subscription_id = self.next_subscription_id
        self.next_subscription_id += 1
        
        # 存储订阅信息
        self.subscribers[subscription_id] = {
            "topic": topic,
            "topic_id": topic_id,
            "callback": callback
        }
        
        return subscription_id

    def unsubscribe(self, subscription_id):
        """
        Unsubscribe from a topic

        Args:
            subscription_id: Subscription ID

        Returns:
            True if successful, False otherwise
        """
        if subscription_id in self.subscribers:
            del self.subscribers[subscription_id]
            return True
        return False

    def publish(self, topic, message):
        """
        Publish a message to a topic

        Args:
            topic: Topic name
            message: Message to publish

        Returns:
            True if successful, False otherwise
        """
        # 获取主题ID
        if topic not in self.topic_map:
            return False
        
        topic_id = self.topic_map[topic]
        
        # 分发消息给订阅者
        self._dispatch_message(topic_id, message)
        
        return True

    def _dispatch_message(self, topic_id, message):
        """
        Dispatch message to subscribers

        Args:
            topic_id: Topic ID
            message: Message to dispatch
        """
        # 分发消息给具体主题的订阅者
        for subscription_id, subscriber in self.subscribers.items():
            if subscriber["topic_id"] == topic_id:
                try:
                    subscriber["callback"](topic_id, message)
                except Exception:
                    pass
        
        # 分发消息给通配符订阅者
        for subscription_id, subscriber in self.subscribers.items():
            if subscriber["topic_id"] == 0xFFFF:
                try:
                    subscriber["callback"](topic_id, message)
                except Exception:
                    pass

    def start(self):
        """
        Start the PubSub system

        Returns:
            True if successful, False otherwise
        """
        # 这里可以启动接收线程
        # 由于Rust实现中使用轮询方式，Python端暂时只提供基本功能
        return True

    def stop(self):
        """
        Stop the PubSub system

        Returns:
            True if successful, False otherwise
        """
        # 清理资源
        self.subscribers.clear()
        return True

# 辅助函数
def connect(db_path: str = "") -> RemDbConnection:
    """
    Connect to a database

    Args:
        db_path: Path to the database file or empty string for in-memory database

    Returns:
        RemDbConnection instance
    """
    conn = RemDbConnection(db_path)
    conn.connect()
    return conn

def create_pubsub(config=None):
    """
    Create a PubSub instance

    Args:
        config: PubSub configuration dict

    Returns:
        RemDbPubSub instance
    """
    return RemDbPubSub(config)
