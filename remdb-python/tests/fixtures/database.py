"""Database test fixtures and base test case for RemDB Python bindings"""

import unittest
import tempfile
import os
import sys
from typing import Optional, Union

# Try to import remdb, but allow tests to be skipped if not available
try:
    import remdb
    REMDB_AVAILABLE = True
except ImportError:
    remdb = None
    REMDB_AVAILABLE = False

class BaseTestCase(unittest.TestCase):
    """Base test case for RemDB tests with support for both local and network connections"""
    
    # Test configuration
    USE_NETWORK = False  # Set to True to test network connections
    NETWORK_HOST = "localhost"
    NETWORK_PORT = 6666
    NETWORK_URL = f"jdbc://{NETWORK_HOST}:{NETWORK_PORT}"
    
    def setUp(self):
        """Set up test environment"""
        super().setUp()
        
        # Skip test if remdb is not available
        if not REMDB_AVAILABLE:
            self.skipTest("remdb module not available")
        
        # Create temporary file for local database if not using network
        if not self.USE_NETWORK:
            self.temp_file = tempfile.NamedTemporaryFile(delete=False, suffix='.rdb')
            self.db_path = self.temp_file.name
            self.temp_file.close()
            self.connection_url = self.db_path
        else:
            self.db_path = None
            self.connection_url = self.NETWORK_URL
            
        # Connect to database
        self.conn = remdb.connect(self.connection_url)
        self.conn.connect()
        
    def tearDown(self):
        """Clean up test environment"""
        if self.conn:
            self.conn.close()
            
        # Clean up temporary file for local database
        if not self.USE_NETWORK and self.db_path and os.path.exists(self.db_path):
            os.unlink(self.db_path)
            
        super().tearDown()
    
    def execute_sql(self, sql: str, params: Optional[list] = None):
        """Execute SQL statement and return result set"""
        return self.conn.execute_query(sql)
    
    def assert_table_exists(self, table_name: str):
        """Assert that a table exists in the database"""
        try:
            table = self.conn.get_table(table_name)
            self.assertIsNotNone(table)
        except remdb.NotFoundError:
            self.fail(f"Table '{table_name}' does not exist")
    
    def assert_table_not_exists(self, table_name: str):
        """Assert that a table does not exist in the database"""
        try:
            table = self.conn.get_table(table_name)
            if table is not None:
                self.fail(f"Table '{table_name}' exists but should not")
        except remdb.NotFoundError:
            # Expected - table should not exist
            pass
    
    def assert_row_count(self, table_name: str, expected_count: int):
        """Assert that a table has the expected number of rows"""
        try:
            table = self.conn.get_table(table_name)
            count = table.get_record_count()
            self.assertEqual(count, expected_count, 
                           f"Table '{table_name}' has {count} rows, expected {expected_count}")
        except remdb.NotFoundError:
            self.fail(f"Table '{table_name}' does not exist")
    
    def assert_sql_result(self, sql: str, expected_rows: list):
        """Execute SQL and assert the result matches expected rows"""
        result_set = self.execute_sql(sql)
        rows = []
        for row in result_set:
            rows.append(row)
        
        self.assertEqual(len(rows), len(expected_rows),
                        f"Expected {len(expected_rows)} rows, got {len(rows)}")
        
        for i, (actual_row, expected_row) in enumerate(zip(rows, expected_rows)):
            self.assertEqual(actual_row, expected_row,
                           f"Row {i} mismatch: expected {expected_row}, got {actual_row}")
    
    def create_test_table(self, table_name: str, schema: str):
        """Create a test table with the given schema"""
        sql = f"CREATE TABLE {table_name} ({schema})"
        self.execute_sql(sql)
        self.assert_table_exists(table_name)
    
    def drop_test_table(self, table_name: str):
        """Drop a test table if it exists"""
        try:
            sql = f"DROP TABLE IF EXISTS {table_name}"
            self.execute_sql(sql)
        except Exception as e:
            print(f"Warning: Failed to drop table {table_name}: {e}")
    
    def insert_test_data(self, table_name: str, data: list):
        """Insert test data into a table"""
        for row in data:
            if isinstance(row, dict):
                columns = list(row.keys())
                values = list(row.values())
                columns_str = ", ".join(columns)
                # Format values properly for SQL
                values_str = ", ".join([self._format_sql_value(v) for v in values])
                sql = f"INSERT INTO {table_name} ({columns_str}) VALUES ({values_str})"
            else:
                # Assume row is a tuple/list
                values_str = ", ".join([self._format_sql_value(v) for v in row])
                sql = f"INSERT INTO {table_name} VALUES ({values_str})"
            
            self.execute_sql(sql)
    
    def _format_sql_value(self, value):
        """Format a Python value for SQL insertion"""
        if value is None:
            return "NULL"
        elif isinstance(value, str):
            # Escape single quotes
            escaped = value.replace("'", "''")
            return f"'{escaped}'"
        elif isinstance(value, bool):
            return "TRUE" if value else "FALSE"
        elif isinstance(value, (int, float)):
            return str(value)
        else:
            # Convert to string and escape
            escaped = str(value).replace("'", "''")
            return f"'{escaped}'"


class LocalTestCase(BaseTestCase):
    """Test case for local database connections"""
    USE_NETWORK = False


class NetworkTestCase(BaseTestCase):
    """Test case for network database connections"""
    USE_NETWORK = True


def skip_if_network_unavailable(test_func):
    """Decorator to skip tests if network connection is unavailable"""
    def wrapper(self, *args, **kwargs):
        if not hasattr(self, 'USE_NETWORK') or not self.USE_NETWORK:
            return test_func(self, *args, **kwargs)
        
        # Check if network server is available
        import socket
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(1)
        try:
            s.connect((BaseTestCase.NETWORK_HOST, BaseTestCase.NETWORK_PORT))
            s.close()
            return test_func(self, *args, **kwargs)
        except (socket.error, ConnectionRefusedError):
            s.close()
            self.skipTest(f"Network server not available at {BaseTestCase.NETWORK_HOST}:{BaseTestCase.NETWORK_PORT}")
    
    return wrapper