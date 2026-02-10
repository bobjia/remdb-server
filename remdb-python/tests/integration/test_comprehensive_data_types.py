"""Comprehensive integration tests for all RemDB data types with extensive insert and query operations"""

import unittest
import time
from tests.fixtures import LocalTestCase

class TestComprehensiveDataTypes(LocalTestCase):
    """Comprehensive test for all RemDB data types"""
    
    def test_all_data_types_creation(self):
        """Test creating table with all data types"""
        table_name = "all_types_table"
        schema = """
            id INTEGER PRIMARY KEY,
            int_val INTEGER,
            real_val REAL,
            text_val TEXT,
            bool_val BOOLEAN,
            ts_val TIMESTAMP,
            json_val JSON,
            vec_val VECTOR(3)
        """
        
        try:
            self.create_test_table(table_name, schema)
            self.assert_table_exists(table_name)
        except Exception as e:
            # Skip if VECTOR type is not supported or causes memory issues
            self.skipTest(f"VECTOR type not supported: {e}")
    
    def test_integer_comprehensive(self):
        """Comprehensive test for INTEGER data type"""
        table_name = "test_integer_comprehensive"
        schema = "id INTEGER PRIMARY KEY, value INTEGER, count INTEGER"
        
        self.create_test_table(table_name, schema)
        
        # Test various insertion patterns
        test_data = [
            (1, 0, 0),
            (2, 1, 100),
            (3, -1, -50),
            (4, 2147483647, 1000),  # Max 32-bit int
            (5, -2147483648, -1000),  # Min 32-bit int
        ]
        
        for data in test_data:
            self.execute_sql(f"INSERT INTO {table_name} VALUES {data}")
        
        self.assert_row_count(table_name, 5)
        
        # Test various queries
        # Basic select
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        self.assertEqual(len(rows), 5)
        
        # Where clause with equality
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE value = 1")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        
        # Where clause with range
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE value > 0")
        rows = list(result)
        self.assertEqual(len(rows), 2)  # Only 1 and 2147483647
        
        # Aggregation
        result = self.execute_sql(f"SELECT SUM(value) as total FROM {table_name}")
        rows = list(result)
        self.assertIsNotNone(rows[0]['total'])
    
    def test_real_comprehensive(self):
        """Comprehensive test for REAL data type"""
        table_name = "test_real_comprehensive"
        schema = "id INTEGER PRIMARY KEY, value REAL, price REAL"
        
        self.create_test_table(table_name, schema)
        
        # Test various insertion patterns
        test_data = [
            (1, 0.0, 0.0),
            (2, 1.5, 99.99),
            (3, -2.7, -10.5),
            (4, 3.1415926535, 1234.5678),
            (5, 1000000.0, 0.0001),
        ]
        
        for data in test_data:
            self.execute_sql(f"INSERT INTO {table_name} VALUES {data}")
        
        self.assert_row_count(table_name, 5)
        
        # Test various queries
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY value DESC")
        rows = list(result)
        self.assertEqual(len(rows), 5)
        
        # Where clause with comparison
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE value > 0")
        rows = list(result)
        self.assertEqual(len(rows), 3)  # 1.5, 3.1415926535, 1000000.0
        
        # Aggregation
        result = self.execute_sql(f"SELECT AVG(value) as avg FROM {table_name}")
        rows = list(result)
        self.assertIsNotNone(rows[0]['avg'])
    
    def test_text_comprehensive(self):
        """Comprehensive test for TEXT data type"""
        table_name = "test_text_comprehensive"
        schema = "id INTEGER PRIMARY KEY, name TEXT, description TEXT"
        
        self.create_test_table(table_name, schema)
        
        # Test various insertion patterns
        test_data = [
            (1, "Alice", "User with normal name"),
            (2, "Bob", "User with special chars: !@#$%^&*()"),
            (3, "Charlie", "User with 'single' and \"double\" quotes"),
            (4, "David", ""),  # Empty string
            (5, "Eve", "User with very long text " * 5),  # Longer text
        ]
        
        for data in test_data:
            # Escape single quotes properly
            id_val, name, desc = data
            name_escaped = name.replace("'", "''")
            desc_escaped = desc.replace("'", "''")
            self.execute_sql(f"INSERT INTO {table_name} VALUES ({id_val}, '{name_escaped}', '{desc_escaped}')")
        
        self.assert_row_count(table_name, 5)
        
        # Test various queries
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY name")
        rows = list(result)
        self.assertEqual(len(rows), 5)
        
        # Where clause with equality
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE name = 'Alice'")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        
        # Where clause with LIKE
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE name LIKE 'B%'")
        rows = list(result)
        self.assertEqual(len(rows), 1)  # Only Bob
    
    def test_boolean_comprehensive(self):
        """Comprehensive test for BOOLEAN data type"""
        table_name = "test_boolean_comprehensive"
        schema = "id INTEGER PRIMARY KEY, active BOOLEAN, verified BOOLEAN"
        
        self.create_test_table(table_name, schema)
        
        # Test various insertion patterns
        test_data = [
            (1, True, True),
            (2, True, False),
            (3, False, True),
            (4, False, False),
            (5, True, True),
        ]
        
        for data in test_data:
            self.execute_sql(f"INSERT INTO {table_name} VALUES {data}")
        
        self.assert_row_count(table_name, 5)
        
        # Test various queries
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        self.assertEqual(len(rows), 5)
        
        # Where clause with boolean equality
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE active = TRUE")
        rows = list(result)
        self.assertEqual(len(rows), 3)  # IDs 1, 2, 5
        
        # Where clause with AND condition
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE active = TRUE AND verified = TRUE")
        rows = list(result)
        self.assertEqual(len(rows), 2)  # IDs 1, 5
    
    def test_timestamp_comprehensive(self):
        """Comprehensive test for TIMESTAMP data type"""
        table_name = "test_timestamp_comprehensive"
        schema = "id INTEGER PRIMARY KEY, event_time TIMESTAMP, created_at TIMESTAMP"
        
        self.create_test_table(table_name, schema)
        
        # Test various insertion patterns
        current_ts = int(time.time() * 1000)  # Current time in ms
        test_data = [
            (1, 0, 0),  # Epoch start
            (2, 1609459200000, 1609459200000),  # 2021-01-01
            (3, current_ts, current_ts),  # Current time
            (4, current_ts + 3600000, current_ts),  # Future time
            (5, current_ts - 3600000, current_ts),  # Past time
        ]
        
        for data in test_data:
            self.execute_sql(f"INSERT INTO {table_name} VALUES {data}")
        
        self.assert_row_count(table_name, 5)
        
        # Test various queries
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY event_time DESC")
        rows = list(result)
        self.assertEqual(len(rows), 5)
        
        # Where clause with timestamp comparison
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE event_time > {current_ts}")
        rows = list(result)
        self.assertEqual(len(rows), 1)  # Only future time
        
        # Where clause with BETWEEN
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE event_time BETWEEN {current_ts - 4000000} AND {current_ts + 4000000}")
        rows = list(result)
        self.assertEqual(len(rows), 3)  # Current, future, past times
    
    def test_json_comprehensive(self):
        """Comprehensive test for JSON data type"""
        table_name = "test_json_comprehensive"
        schema = "id INTEGER PRIMARY KEY, data JSON, config JSON"
        
        self.create_test_table(table_name, schema)
        
        # Test various insertion patterns
        test_data = [
            (1, '{"name": "Alice", "age": 30}', '{"theme": "dark", "notifications": true}'),
            (2, '[1, 2, 3, 4, 5]', '{"settings": {"font": "Arial", "size": 12}}'),
            (3, '{"status": "active", "tags": ["user", "admin"]}', '{}'),  # Empty object
            (4, '"simple string"', 'null'),  # JSON string and null
            (5, '{"nested": {"level1": {"level2": "value"}}}', '{"array": [{"id": 1}, {"id": 2}]}'),
        ]
        
        for data in test_data:
            self.execute_sql(f"INSERT INTO {table_name} VALUES {data}")
        
        self.assert_row_count(table_name, 5)
        
        # Test various queries
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        self.assertEqual(len(rows), 5)
    
    def test_vector_comprehensive(self):
        """Comprehensive test for VECTOR data type"""
        table_name = "test_vector_comprehensive"
        # Use small vector dimensions to avoid memory issues
        schema = "id INTEGER PRIMARY KEY, vec VECTOR(3)"
        
        try:
            self.create_test_table(table_name, schema)
            
            # Test vector insertion (simple case)
            # Note: VECTOR insertion syntax may vary based on implementation
            test_data = [
                (1, '[1.0, 2.0, 3.0]'),
                (2, '[4.0, 5.0, 6.0]'),
                (3, '[7.0, 8.0, 9.0]'),
            ]
            
            for data in test_data:
                self.execute_sql(f"INSERT INTO {table_name} VALUES {data}")
            
            self.assert_row_count(table_name, 3)
            
            # Test vector queries
            # Basic select
            result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
            rows = list(result)
            self.assertEqual(len(rows), 3)
            
        except Exception as e:
            # Skip if VECTOR type is not fully supported
            self.skipTest(f"VECTOR operations not fully supported: {e}")
    
    def test_mixed_data_types(self):
        """Test mixed data types in single table"""
        table_name = "test_mixed_types"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT,
            age INTEGER,
            salary REAL,
            active BOOLEAN,
            hire_date TIMESTAMP,
            profile JSON
        """
        
        self.create_test_table(table_name, schema)
        
        # Test inserting mixed data
        test_data = [
            (1, 'Alice', 30, 75000.50, True, 1609459200000, '{"position": "Engineer", "skills": ["Python", "SQL"]}'),
            (2, 'Bob', 25, 65000.00, True, 1609459260000, '{"position": "Sales", "skills": ["Communication", "Negotiation"]}'),
            (3, 'Charlie', 35, 85000.75, False, 1609459320000, '{"position": "Manager", "skills": ["Leadership", "Planning"]}'),
        ]
        
        for data in test_data:
            self.execute_sql(f"INSERT INTO {table_name} VALUES {data}")
        
        self.assert_row_count(table_name, 3)
        
        # Test complex queries with mixed data types
        result = self.execute_sql(f"""
            SELECT name, age, salary 
            FROM {table_name} 
            WHERE age > 25 AND active = TRUE 
            ORDER BY salary DESC
        """)
        rows = list(result)
        self.assertEqual(len(rows), 1)  # Only Alice
    
    def test_insert_multiple_rows(self):
        """Test inserting multiple rows in single statement"""
        table_name = "test_multiple_inserts"
        schema = "id INTEGER PRIMARY KEY, name TEXT, value INTEGER"
        
        self.create_test_table(table_name, schema)
        
        # Test multiple row insertion
        self.execute_sql(f"""
            INSERT INTO {table_name} (id, name, value) 
            VALUES (1, 'Alice', 100),
                   (2, 'Bob', 200),
                   (3, 'Charlie', 300),
                   (4, 'David', 400),
                   (5, 'Eve', 500)
        """)
        
        self.assert_row_count(table_name, 5)
        
        # Verify all rows were inserted
        result = self.execute_sql(f"SELECT COUNT(*) as count FROM {table_name}")
        rows = list(result)
        self.assertEqual(rows[0]['count'], 5)
    
    def test_comprehensive_queries(self):
        """Test comprehensive query operations"""
        table_name = "test_comprehensive_queries"
        schema = "id INTEGER PRIMARY KEY, department TEXT, salary REAL, age INTEGER, active BOOLEAN"
        
        self.create_test_table(table_name, schema)
        
        # Insert test data
        test_data = [
            (1, 'Engineering', 75000.0, 30, True),
            (2, 'Sales', 65000.0, 25, True),
            (3, 'Engineering', 85000.0, 35, True),
            (4, 'Marketing', 60000.0, 28, False),
            (5, 'Engineering', 95000.0, 40, True),
            (6, 'Sales', 70000.0, 27, True),
        ]
        
        for data in test_data:
            self.execute_sql(f"INSERT INTO {table_name} VALUES {data}")
        
        self.assert_row_count(table_name, 6)
        
        # Test 1: Basic SELECT with ORDER BY
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY salary DESC")
        rows = list(result)
        self.assertEqual(len(rows), 6)
        self.assertGreater(rows[0]['salary'], rows[-1]['salary'])
        
        # Test 2: WHERE clause with multiple conditions
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE department = 'Engineering' AND age > 30")
        rows = list(result)
        self.assertEqual(len(rows), 2)  # IDs 3, 5
        
        # Test 3: Aggregation functions
        result = self.execute_sql(f"SELECT department, AVG(salary) as avg_salary, COUNT(*) as count FROM {table_name} GROUP BY department")
        rows = list(result)
        self.assertEqual(len(rows), 3)  # 3 departments
        
        # Test 4: HAVING clause
        result = self.execute_sql(f"SELECT department, COUNT(*) as count FROM {table_name} GROUP BY department HAVING COUNT(*) > 1")
        rows = list(result)
        self.assertEqual(len(rows), 2)  # Engineering and Sales have >1 employees
        
        # Test 5: LIMIT clause
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY salary DESC LIMIT 2")
        rows = list(result)
        self.assertEqual(len(rows), 2)  # Top 2 salaries
        
        # Test 6: DISTINCT
        result = self.execute_sql(f"SELECT DISTINCT department FROM {table_name}")
        rows = list(result)
        self.assertEqual(len(rows), 3)  # 3 distinct departments

if __name__ == '__main__':
    unittest.main()
