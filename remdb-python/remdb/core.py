"""Core Python API for RemDB"""

import _remdb
import contextlib
import datetime
from typing import Dict, List, Optional, Any, Union

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
            db_path: Path to the database file or empty string for in-memory database
        """
        self.db_path = db_path
        self.db = _remdb.RemDb()
        self.connected = False

    def __enter__(self):
        """Enter context manager"""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager"""
        self.close()

    def connect(self):
        """Connect to the database"""
        if not self.connected:
            self.connected = self.db.connect(self.db_path)
            if not self.connected:
                raise RemDbError(f"Failed to connect to database: {self.db_path}")
        return self.connected

    def close(self):
        """Close the database connection"""
        # 清理资源
        self.connected = False

    def get_table(self, table_name: str) -> "RemDbTable":
        """
        Get a table by name

        Args:
            table_name: Name of the table

        Returns:
            RemDbTable instance

        Raises:
            NotFoundError: If the table does not exist
        """
        if not self.connected:
            raise RemDbError("Not connected to database")

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

        return self.db.save_snapshot(path)

    def restore_snapshot(self, path: str) -> bool:
        """
        Restore database from snapshot

        Args:
            path: Path to the snapshot file

        Returns:
            True if successful, False otherwise
        """
        if not self.connected:
            raise RemDbError("Not connected to database")

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
        # TODO: 实现记录转换逻辑
        # 这里简化处理，假设record已经是正确的字节序列
        return self.table.insert(record)

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
                zero_copy_data = self.table.get_zero_copy(key)
                # 转换为memoryview
                return memoryview(zero_copy_data.tobytes())
            except Exception:
                # 如果零拷贝失败，回退到普通模式
                pass
        
        # 普通模式
        record = []
        success = self.table.get(key, record)
        if not success:
            return None
        # TODO: 解析record为字典
        return {}

    def update(self, key: Any, record: Union[Dict[str, Any], List[Any]]) -> bool:
        """
        Update a record by key

        Args:
            key: Primary key value
            record: Dictionary or list of values

        Returns:
            True if successful, False otherwise
        """
        # TODO: 实现记录转换逻辑
        return self.table.update(key, record)

    def delete(self, key: Any) -> bool:
        """
        Delete a record by key

        Args:
            key: Primary key value

        Returns:
            True if successful, False otherwise
        """
        return self.table.delete_record(key)

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

    def query(self) -> QueryBuilder:
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

# 结果集类
class RemDbResultSet:
    """Query result set class"""

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
        self.limit = None

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
        self.limit = limit
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
        if self.limit is not None:
            limit_clause = f"LIMIT {self.limit}"

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
        # 注意：这里需要实现参数化查询，暂时简化处理
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
