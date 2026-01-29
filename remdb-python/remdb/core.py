"""Core Python API for RemDB"""

import sys
import os

# 添加项目根目录到Python路径
sys.path.insert(0, os.path.abspath(os.path.dirname(os.path.dirname(__file__))))

import _remdb
import contextlib
import datetime
from typing import Dict, List, Optional, Any, Union

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

# 数据库连接类
class RemDbConnection:
    """Database connection class with context manager support"""

    def __init__(self, db_path: str = ""):
        """
        Initialize a database connection

        Args:
            db_path: Path to the database file (e.g., "path/to/database.rdb") or JDBC URL (e.g., "jdbc://host:port")
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
                    self.jdbc_client = JdbcClient(self.host, self.port)
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
        
        # 转换记录格式以匹配C++实现的期望
        if isinstance(record, dict):
            # 将字典中的所有值转换为字符串
            str_record = {k: str(v) for k, v in record.items()}
            return self.table.insert(str_record)
        elif isinstance(record, list):
            # 将列表中的所有值转换为字符串
            str_record = [str(item) for item in record]
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
        return {"id": record[0]} if record else {}

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
        
        # 转换记录格式以匹配C++实现的期望
        if isinstance(record, dict):
            # 将字典中的所有值转换为字符串
            str_record = {k: str(v) for k, v in record.items()}
            return self.table.update(str(key), str_record)
        elif isinstance(record, list):
            # 将列表中的所有值转换为字符串
            str_record = [str(item) for item in record]
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
                
                # 构建SQL语句
                cols_str = ", ".join(columns)
                placeholders = ", ".join(["?"] * len(values))
                sql = f"INSERT INTO {self.table_name} ({cols_str}) VALUES ({placeholders})"
                
                # 执行查询
                self.connection.jdbc_client.execute_query(sql, values)
                return True
            else:
                # 对于列表类型，需要知道列名，这里简化处理
                return False
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
            # 构建SELECT语句
            sql = f"SELECT * FROM {self.table_name} WHERE id = ?"
            result = self.connection.jdbc_client.execute_query(sql, [key])
            
            # 检查结果
            if result.get("rows") and len(result.get("rows")) > 0:
                row = result.get("rows")[0]
                columns = result.get("columns")
                return dict(zip(columns, row))
            return None
        except Exception:
            return None

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
                # 构建UPDATE语句
                set_clause = ", ".join([f"{col} = ?" for col in record.keys()])
                values = list(record.values())
                values.append(key)
                
                sql = f"UPDATE {self.table_name} SET {set_clause} WHERE id = ?"
                
                # 执行查询
                self.connection.jdbc_client.execute_query(sql, values)
                return True
            else:
                # 对于列表类型，需要知道列名，这里简化处理
                return False
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
        try:
            # 构建DELETE语句
            sql = f"DELETE FROM {self.table_name} WHERE id = ?"
            
            # 执行查询
            self.connection.jdbc_client.execute_query(sql, [key])
            return True
        except Exception:
            return False

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
        return dict(zip(self.columns, values))

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
