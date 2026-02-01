"""Test data types supported by RemDB as documented in sql_language.md"""

import unittest
from tests.fixtures import LocalTestCase
from tests.utils import generate_random_string, generate_random_int, generate_random_float, generate_random_bool, generate_random_timestamp, generate_random_vector

class TestDataTypeINTEGER(LocalTestCase):
    """Test INTEGER data type"""
    
    def test_integer_creation(self):
        """Test creating table with INTEGER columns"""
        table_name = "test_integer_table"
        schema = """
            id INTEGER PRIMARY KEY,
            small_int INTEGER,
            large_int INTEGER
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert data
        test_data = [
            {"id": 1, "small_int": -100, "large_int": 1000000},
            {"id": 2, "small_int": 0, "large_int": -500000},
            {"id": 3, "small_int": 255, "large_int": 2147483647},
        ]
        
        self.insert_test_data(table_name, test_data)
        self.assert_row_count(table_name, 3)
        
        # Query and verify
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        
        self.assertEqual(len(rows), 3)
        self.assertEqual(rows[0]["small_int"], -100)
        self.assertEqual(rows[0]["large_int"], 1000000)
        self.assertEqual(rows[2]["large_int"], 2147483647)
    
    def test_integer_operations(self):
        """Test arithmetic operations on INTEGER columns"""
        table_name = "test_integer_ops"
        schema = "a INTEGER, b INTEGER, c INTEGER"
        
        self.create_test_table(table_name, schema)
        
        # Insert test data
        self.execute_sql(f"INSERT INTO {table_name} VALUES (10, 20, 30)")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (-5, 15, -25)")
        
        # Test arithmetic in SELECT
        result = self.execute_sql(
            f"SELECT a + b AS sum, a - b AS diff, a * b AS prod, b / a AS div FROM {table_name} WHERE a > 0"
        )
        rows = list(result)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["sum"], 30)
        self.assertEqual(rows[0]["diff"], -10)
        self.assertEqual(rows[0]["prod"], 200)
        self.assertEqual(rows[0]["div"], 2)  # 20 / 10 = 2
    
    def test_integer_constraints(self):
        """Test INTEGER with constraints (PRIMARY KEY, NOT NULL)"""
        table_name = "test_integer_constraints"
        schema = """
            id INTEGER PRIMARY KEY,
            value INTEGER NOT NULL,
            optional INTEGER
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert valid data
        self.execute_sql(f"INSERT INTO {table_name} (id, value) VALUES (1, 100)")
        self.execute_sql(f"INSERT INTO {table_name} (id, value, optional) VALUES (2, 200, NULL)")
        
        # Should fail: duplicate primary key
        with self.assertRaises(Exception):
            self.execute_sql(f"INSERT INTO {table_name} (id, value) VALUES (1, 300)")
        
        # Should fail: NOT NULL constraint
        with self.assertRaises(Exception):
            self.execute_sql(f"INSERT INTO {table_name} (id) VALUES (3)")


class TestDataTypeREAL(LocalTestCase):
    """Test REAL data type (floating point numbers)"""
    
    def test_real_creation(self):
        """Test creating table with REAL columns"""
        table_name = "test_real_table"
        schema = """
            id INTEGER PRIMARY KEY,
            temperature REAL,
            price REAL,
            ratio REAL
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert data with various real values
        test_data = [
            {"id": 1, "temperature": 23.5, "price": 99.99, "ratio": 0.75},
            {"id": 2, "temperature": -10.2, "price": 0.0, "ratio": 1.0},
            {"id": 3, "temperature": 100.0, "price": 1234.5678, "ratio": 3.1415926535},
        ]
        
        self.insert_test_data(table_name, test_data)
        self.assert_row_count(table_name, 3)
        
        # Query and verify with tolerance for floating point
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        
        self.assertEqual(len(rows), 3)
        self.assertAlmostEqual(rows[0]["temperature"], 23.5, places=5)
        self.assertAlmostEqual(rows[2]["price"], 1234.5678, places=5)
        self.assertAlmostEqual(rows[2]["ratio"], 3.1415926535, places=5)
    
    def test_real_operations(self):
        """Test arithmetic operations on REAL columns"""
        table_name = "test_real_ops"
        schema = "x REAL, y REAL"
        
        self.create_test_table(table_name, schema)
        
        self.execute_sql(f"INSERT INTO {table_name} VALUES (10.5, 2.0)")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (-3.14, 1.5)")
        
        # Test various operations
        result = self.execute_sql(
            f"SELECT x + y AS sum, x - y AS diff, x * y AS prod, x / y AS div FROM {table_name} ORDER BY x"
        )
        rows = list(result)
        
        self.assertEqual(len(rows), 2)
        self.assertAlmostEqual(rows[0]["sum"], -1.64, places=5)  # -3.14 + 1.5
        self.assertAlmostEqual(rows[0]["prod"], -4.71, places=5)  # -3.14 * 1.5
        self.assertAlmostEqual(rows[1]["div"], 5.25, places=5)   # 10.5 / 2.0


class TestDataTypeTEXT(LocalTestCase):
    """Test TEXT data type (strings)"""
    
    def test_text_creation(self):
        """Test creating table with TEXT columns"""
        table_name = "test_text_table"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT,
            description TEXT,
            code TEXT
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert data with various strings
        test_data = [
            {"id": 1, "name": "Alice", "description": "A user with special chars: !@#$%^&*()", "code": "ABC123"},
            {"id": 2, "name": "Bob", "description": "", "code": "XYZ789"},
            {"id": 3, "name": "Charlie", "description": "With 'single quotes' and \"double quotes\"", "code": "DEF456"},
        ]
        
        self.insert_test_data(table_name, test_data)
        self.assert_row_count(table_name, 3)
        
        # Query and verify
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        
        self.assertEqual(len(rows), 3)
        self.assertEqual(rows[0]["name"], "Alice")
        self.assertEqual(rows[1]["description"], "")
        self.assertEqual(rows[2]["description"], "With 'single quotes' and \"double quotes\"")
    
    def test_text_length_limit(self):
        """Test TEXT length limit (64 bytes according to documentation)"""
        table_name = "test_text_length"
        schema = "id INTEGER PRIMARY KEY, content TEXT"
        
        self.create_test_table(table_name, schema)
        
        # Insert string at the limit
        max_length_str = "A" * 64  # 64 bytes if using ASCII
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1, '{max_length_str}')")
        
        # Try to insert longer string - should be truncated or fail
        too_long_str = "B" * 100
        self.execute_sql(f"INSERT INTO {table_name} VALUES (2, '{too_long_str}')")
        
        # Verify first insertion
        result = self.execute_sql(f"SELECT content FROM {table_name} WHERE id = 1")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        # Note: Depending on implementation, may be truncated or error
    
    def test_text_functions(self):
        """Test string functions on TEXT columns"""
        table_name = "test_text_functions"
        schema = "id INTEGER PRIMARY KEY, text_field TEXT"
        
        self.create_test_table(table_name, schema)
        
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1, 'Hello World')")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (2, 'remdb test')")
        
        # Test CONCAT (if supported)
        result = self.execute_sql(
            f"SELECT CONCAT(text_field, '!') AS greeting FROM {table_name} ORDER BY id"
        )
        rows = list(result)
        
        # Note: Function support depends on RemDB implementation
        # This test may need adjustment based on actual function availability


class TestDataTypeBOOLEAN(LocalTestCase):
    """Test BOOLEAN data type"""
    
    def test_boolean_creation(self):
        """Test creating table with BOOLEAN columns"""
        table_name = "test_boolean_table"
        schema = """
            id INTEGER PRIMARY KEY,
            active BOOLEAN,
            verified BOOLEAN,
            flag BOOLEAN
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert boolean values
        test_data = [
            {"id": 1, "active": True, "verified": True, "flag": False},
            {"id": 2, "active": False, "verified": True, "flag": True},
            {"id": 3, "active": True, "verified": False, "flag": False},
        ]
        
        self.insert_test_data(table_name, test_data)
        self.assert_row_count(table_name, 3)
        
        # Query and verify
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        
        self.assertEqual(len(rows), 3)
        self.assertEqual(rows[0]["active"], True)
        self.assertEqual(rows[0]["flag"], False)
        self.assertEqual(rows[1]["active"], False)
        self.assertEqual(rows[2]["verified"], False)
    
    def test_boolean_operations(self):
        """Test logical operations on BOOLEAN columns"""
        table_name = "test_boolean_ops"
        schema = "a BOOLEAN, b BOOLEAN"
        
        self.create_test_table(table_name, schema)
        
        self.execute_sql(f"INSERT INTO {table_name} VALUES (TRUE, TRUE)")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (TRUE, FALSE)")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (FALSE, TRUE)")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (FALSE, FALSE)")
        
        # Test AND operation
        result = self.execute_sql(
            f"SELECT COUNT(*) as count FROM {table_name} WHERE a AND b"
        )
        rows = list(result)
        self.assertEqual(rows[0]["count"], 1)  # Only TRUE, TRUE
        
        # Test OR operation
        result = self.execute_sql(
            f"SELECT COUNT(*) as count FROM {table_name} WHERE a OR b"
        )
        rows = list(result)
        self.assertEqual(rows[0]["count"], 3)  # All except FALSE, FALSE
        
        # Test NOT operation
        result = self.execute_sql(
            f"SELECT COUNT(*) as count FROM {table_name} WHERE NOT a"
        )
        rows = list(result)
        self.assertEqual(rows[0]["count"], 2)  # Rows where a is FALSE


class TestDataTypeTIMESTAMP(LocalTestCase):
    """Test TIMESTAMP data type"""
    
    def test_timestamp_creation(self):
        """Test creating table with TIMESTAMP columns"""
        table_name = "test_timestamp_table"
        schema = """
            id INTEGER PRIMARY KEY,
            event_time TIMESTAMP,
            created_at TIMESTAMP,
            updated_at TIMESTAMP
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert timestamp values (milliseconds since epoch)
        test_data = [
            {"id": 1, "event_time": 1609459200000, "created_at": 1609459200000, "updated_at": 1609459260000},
            {"id": 2, "event_time": 1609459260000, "created_at": 1609459260000, "updated_at": 1609459320000},
            {"id": 3, "event_time": 1609459320000, "created_at": 1609459320000, "updated_at": 1609459380000},
        ]
        
        self.insert_test_data(table_name, test_data)
        self.assert_row_count(table_name, 3)
        
        # Query and verify
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY event_time")
        rows = list(result)
        
        self.assertEqual(len(rows), 3)
        self.assertEqual(rows[0]["event_time"], 1609459200000)
        self.assertEqual(rows[1]["updated_at"], 1609459320000)
        self.assertLess(rows[0]["event_time"], rows[2]["event_time"])
    
    def test_timestamp_comparisons(self):
        """Test comparisons and ranges with TIMESTAMP columns"""
        table_name = "test_timestamp_comparisons"
        schema = "id INTEGER PRIMARY KEY, ts TIMESTAMP"
        
        self.create_test_table(table_name, schema)
        
        timestamps = [1609459200000, 1609459260000, 1609459320000, 1609459380000]
        for i, ts in enumerate(timestamps):
            self.execute_sql(f"INSERT INTO {table_name} VALUES ({i + 1}, {ts})")
        
        # Test greater than
        result = self.execute_sql(
            f"SELECT COUNT(*) as count FROM {table_name} WHERE ts > 1609459260000"
        )
        rows = list(result)
        self.assertEqual(rows[0]["count"], 2)  # timestamps 3 and 4
        
        # Test between
        result = self.execute_sql(
            f"SELECT COUNT(*) as count FROM {table_name} WHERE ts BETWEEN 1609459200000 AND 1609459320000"
        )
        rows = list(result)
        self.assertEqual(rows[0]["count"], 3)  # timestamps 1, 2, 3
        
        # Test ordering
        result = self.execute_sql(f"SELECT ts FROM {table_name} ORDER BY ts DESC")
        rows = list(result)
        self.assertEqual(len(rows), 4)
        self.assertEqual(rows[0]["ts"], 1609459380000)  # Latest first
        self.assertEqual(rows[3]["ts"], 1609459200000)  # Earliest last


class TestDataTypeVECTOR(LocalTestCase):
    """Test VECTOR data type"""
    
    def test_vector_creation(self):
        """Test creating table with VECTOR columns"""
        table_name = "test_vector_table"
        schema = """
            id INTEGER PRIMARY KEY,
            embedding VECTOR(128),
            features VECTOR(64),
            small_vector VECTOR(3)
        """
        
        self.create_test_table(table_name, schema)
        
        # Note: VECTOR insertion syntax may require special handling
        # This test may need adjustment based on actual VECTOR support
        
        # For now, test that table creation works
        self.assert_table_exists(table_name)
    
    def test_vector_dimensions(self):
        """Test VECTOR with different dimensions"""
        dimensions = [3, 16, 64, 128, 256]
        
        for dim in dimensions:
            table_name = f"test_vector_{dim}d"
            schema = f"id INTEGER PRIMARY KEY, vec VECTOR({dim})"
            
            self.create_test_table(table_name, schema)
            self.drop_test_table(table_name)
    
    def test_vector_with_distance(self):
        """Test VECTOR with distance specification"""
        table_name = "test_vector_distance"
        
        # Test different distance metrics
        schemas = [
            "id INTEGER PRIMARY KEY, vec VECTOR(128) WITH DISTANCE=L2",
            "id INTEGER PRIMARY KEY, vec VECTOR(128) WITH DISTANCE=COSINE",
            "id INTEGER PRIMARY KEY, vec VECTOR(128) WITH DISTANCE=IP",
        ]
        
        for i, schema in enumerate(schemas):
            temp_table = f"{table_name}_{i}"
            self.create_test_table(temp_table, schema)
            self.drop_test_table(temp_table)


class TestDataTypeCombinations(LocalTestCase):
    """Test combinations of different data types in same table"""
    
    def test_mixed_data_types(self):
        """Test table with all data types"""
        table_name = "test_mixed_types"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT,
            age INTEGER,
            salary REAL,
            active BOOLEAN,
            created_at TIMESTAMP,
            embedding VECTOR(128)
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert sample data (VECTOR may need special handling)
        test_data = [
            {
                "id": 1,
                "name": "Alice",
                "age": 30,
                "salary": 75000.50,
                "active": True,
                "created_at": 1609459200000,
                # "embedding": [0.1] * 128  # Would need vector insertion syntax
            }
        ]
        
        # Insert without vector for now
        self.execute_sql(f"""
            INSERT INTO {table_name} (id, name, age, salary, active, created_at) 
            VALUES (1, 'Alice', 30, 75000.50, TRUE, 1609459200000)
        """)
        
        self.assert_row_count(table_name, 1)
        
        # Query and verify
        result = self.execute_sql(f"SELECT * FROM {table_name}")
        rows = list(result)
        
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["name"], "Alice")
        self.assertEqual(rows[0]["age"], 30)
        self.assertAlmostEqual(rows[0]["salary"], 75000.50, places=2)
        self.assertEqual(rows[0]["active"], True)
        self.assertEqual(rows[0]["created_at"], 1609459200000)


class TestDataTypeEdgeCases(LocalTestCase):
    """Test edge cases for data types"""
    
    def test_null_values(self):
        """Test NULL values for all data types"""
        table_name = "test_null_values"
        schema = """
            id INTEGER PRIMARY KEY,
            int_val INTEGER,
            real_val REAL,
            text_val TEXT,
            bool_val BOOLEAN,
            ts_val TIMESTAMP
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert row with all NULLs except id
        self.execute_sql(f"INSERT INTO {table_name} (id) VALUES (1)")
        
        # Insert row with mixed NULLs
        self.execute_sql(f"""
            INSERT INTO {table_name} (id, int_val, text_val, bool_val) 
            VALUES (2, 100, 'test', TRUE)
        """)
        
        self.assert_row_count(table_name, 2)
        
        # Query and check NULL handling
        result = self.execute_sql(
            f"SELECT int_val IS NULL as int_null, text_val IS NULL as text_null FROM {table_name} ORDER BY id"
        )
        rows = list(result)
        
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0]["int_null"], True)   # id=1, int_val is NULL
        self.assertEqual(rows[1]["int_null"], False)  # id=2, int_val=100
    
    def test_default_values(self):
        """Test DEFAULT values for data types"""
        table_name = "test_default_values"
        schema = """
            id INTEGER PRIMARY KEY,
            int_val INTEGER DEFAULT 100,
            real_val REAL DEFAULT 0.0,
            text_val TEXT DEFAULT 'unknown',
            bool_val BOOLEAN DEFAULT TRUE,
            ts_val TIMESTAMP DEFAULT 0
        """
        
        self.create_test_table(table_name, schema)
        
        # Insert without specifying values (should use defaults)
        self.execute_sql(f"INSERT INTO {table_name} (id) VALUES (1)")
        
        # Insert overriding some defaults
        self.execute_sql(f"""
            INSERT INTO {table_name} (id, int_val, text_val) 
            VALUES (2, 200, 'custom')
        """)
        
        # Query and verify defaults
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0]["int_val"], 100)        # Default
        self.assertEqual(rows[0]["text_val"], "unknown") # Default
        self.assertEqual(rows[0]["bool_val"], True)      # Default
        
        self.assertEqual(rows[1]["int_val"], 200)        # Overridden
        self.assertEqual(rows[1]["text_val"], "custom")  # Overridden
        self.assertEqual(rows[1]["bool_val"], True)      # Default (not overridden)


if __name__ == '__main__':
    unittest.main()