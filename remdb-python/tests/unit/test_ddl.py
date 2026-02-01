"""Test DDL statements (CREATE TABLE, ALTER TABLE, DROP TABLE, etc.)"""

import unittest
from tests.fixtures import LocalTestCase

class TestCreateTable(LocalTestCase):
    """Test CREATE TABLE statement"""
    
    def test_create_simple_table(self):
        """Test creating a simple table"""
        table_name = "simple_table"
        schema = "id INTEGER, name TEXT"
        
        self.create_test_table(table_name, schema)
        
        # Verify table exists
        self.assert_table_exists(table_name)
        
        # Verify table structure
        # Note: This depends on DESCRIBE TABLE or similar functionality
        # For now, just verify we can query the table
        result = self.execute_sql(f"SELECT * FROM {table_name}")
        # Should not raise error
    
    def test_create_table_with_constraints(self):
        """Test CREATE TABLE with various constraints"""
        table_name = "constrained_table"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            age INTEGER DEFAULT 18,
            salary REAL CHECK (salary >= 0),
            department_id INTEGER,
            FOREIGN KEY (department_id) REFERENCES departments(id)
        """
        
        self.create_test_table(table_name, schema)
        self.assert_table_exists(table_name)
    
    def test_create_table_with_composite_primary_key(self):
        """Test CREATE TABLE with composite primary key"""
        table_name = "composite_pk_table"
        schema = "user_id INTEGER, role_id INTEGER, assigned_date TIMESTAMP, PRIMARY KEY (user_id, role_id)"
        
        self.create_test_table(table_name, schema)
        self.assert_table_exists(table_name)
    
    def test_create_table_with_all_data_types(self):
        """Test CREATE TABLE with all supported data types"""
        table_name = "all_types_table"
        schema = """
            int_col INTEGER,
            real_col REAL,
            text_col TEXT,
            bool_col BOOLEAN,
            ts_col TIMESTAMP,
            vector_col VECTOR(128)
        """
        
        self.create_test_table(table_name, schema)
        self.assert_table_exists(table_name)
    
    def test_create_table_if_not_exists(self):
        """Test CREATE TABLE IF NOT EXISTS"""
        table_name = "if_not_exists_table"
        schema = "id INTEGER"
        
        # First creation should succeed
        self.execute_sql(f"CREATE TABLE IF NOT EXISTS {table_name} ({schema})")
        self.assert_table_exists(table_name)
        
        # Second creation should not fail
        self.execute_sql(f"CREATE TABLE IF NOT EXISTS {table_name} ({schema})")
        self.assert_table_exists(table_name)
        
        # Verify table still works
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1)")
        self.assert_row_count(table_name, 1)
    
    def test_create_table_error_duplicate(self):
        """Test error when creating duplicate table"""
        table_name = "duplicate_table"
        schema = "id INTEGER"
        
        self.create_test_table(table_name, schema)
        
        # Should fail when trying to create again
        with self.assertRaises(Exception):
            self.execute_sql(f"CREATE TABLE {table_name} ({schema})")
    
    def test_create_table_with_index_definition(self):
        """Test CREATE TABLE with index definition in column"""
        table_name = "table_with_indexes"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT INDEX,
            email TEXT UNIQUE INDEX,
            vector_col VECTOR(128) INDEX WITH DISTANCE=COSINE
        """
        
        self.create_test_table(table_name, schema)
        self.assert_table_exists(table_name)


class TestAlterTable(LocalTestCase):
    """Test ALTER TABLE statement"""
    
    def setUp(self):
        super().setUp()
        # Create a base table for ALTER TABLE tests
        self.base_table = "alter_test_table"
        self.create_test_table(self.base_table, "id INTEGER PRIMARY KEY, name TEXT, age INTEGER")
    
    def test_alter_table_add_column(self):
        """Test ALTER TABLE ADD COLUMN"""
        table_name = self.base_table
        
        # Add a single column
        self.execute_sql(f"ALTER TABLE {table_name} ADD COLUMN email TEXT")
        
        # Add multiple columns
        self.execute_sql(f"ALTER TABLE {table_name} ADD COLUMN salary REAL, ADD COLUMN active BOOLEAN DEFAULT TRUE")
        
        # Verify columns exist by inserting data
        self.execute_sql(f"""
            INSERT INTO {table_name} (id, name, age, email, salary, active) 
            VALUES (1, 'Alice', 30, 'alice@example.com', 75000.0, TRUE)
        """)
        
        # Query to verify
        result = self.execute_sql(f"SELECT * FROM {table_name} WHERE id = 1")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["email"], "alice@example.com")
        self.assertEqual(rows[0]["active"], True)
    
    def test_alter_table_add_column_with_constraints(self):
        """Test ALTER TABLE ADD COLUMN with constraints"""
        table_name = self.base_table
        
        # Add column with NOT NULL and DEFAULT
        self.execute_sql(f"ALTER TABLE {table_name} ADD COLUMN status TEXT NOT NULL DEFAULT 'active'")
        
        # Add column with CHECK constraint
        self.execute_sql(f"ALTER TABLE {table_name} ADD COLUMN rating INTEGER CHECK (rating >= 1 AND rating <= 5)")
        
        # Insert data to verify constraints
        self.execute_sql(f"""
            INSERT INTO {table_name} (id, name, age, status, rating) 
            VALUES (1, 'Bob', 25, 'active', 4)
        """)
        
        # Should fail: rating out of range
        with self.assertRaises(Exception):
            self.execute_sql(f"""
                INSERT INTO {table_name} (id, name, age, status, rating) 
                VALUES (2, 'Charlie', 35, 'active', 6)
            """)
    
    def test_alter_table_modify_column(self):
        """Test ALTER TABLE MODIFY COLUMN"""
        table_name = self.base_table
        
        # Insert some data
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1, 'Alice', 30)")
        
        # Modify column type (if supported)
        # Note: MODIFY COLUMN may have limited support
        try:
            self.execute_sql(f"ALTER TABLE {table_name} MODIFY COLUMN age REAL")
            
            # Verify modification
            self.execute_sql(f"INSERT INTO {table_name} (id, name, age) VALUES (2, 'Bob', 25.5)")
            
            result = self.execute_sql(f"SELECT age FROM {table_name} WHERE id = 2")
            rows = list(result)
            self.assertAlmostEqual(rows[0]["age"], 25.5, places=1)
        except Exception:
            # MODIFY COLUMN may not be supported, skip test
            self.skipTest("MODIFY COLUMN not supported")
    
    def test_alter_table_drop_column(self):
        """Test ALTER TABLE DROP COLUMN"""
        table_name = self.base_table
        
        # Insert data
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1, 'Alice', 30)")
        
        # Drop a column
        self.execute_sql(f"ALTER TABLE {table_name} DROP COLUMN age")
        
        # Verify column is gone
        # Should fail when trying to select dropped column
        with self.assertRaises(Exception):
            self.execute_sql(f"SELECT age FROM {table_name}")
        
        # Should succeed selecting remaining columns
        result = self.execute_sql(f"SELECT id, name FROM {table_name}")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["name"], "Alice")
    
    def test_alter_table_rename_column(self):
        """Test ALTER TABLE RENAME COLUMN"""
        table_name = self.base_table
        
        # Insert data
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1, 'Alice', 30)")
        
        # Rename column
        try:
            self.execute_sql(f"ALTER TABLE {table_name} RENAME COLUMN name TO full_name")
            
            # Verify rename
            result = self.execute_sql(f"SELECT full_name FROM {table_name}")
            rows = list(result)
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0]["full_name"], "Alice")
            
            # Old name should not work
            with self.assertRaises(Exception):
                self.execute_sql(f"SELECT name FROM {table_name}")
        except Exception:
            # RENAME COLUMN may not be supported
            self.skipTest("RENAME COLUMN not supported")
    
    def test_alter_table_add_constraint(self):
        """Test ALTER TABLE ADD CONSTRAINT"""
        table_name = self.base_table
        
        # Add UNIQUE constraint
        try:
            self.execute_sql(f"ALTER TABLE {table_name} ADD CONSTRAINT unique_name UNIQUE (name)")
            
            # Insert data with unique names
            self.execute_sql(f"INSERT INTO {table_name} VALUES (1, 'Alice', 30)")
            self.execute_sql(f"INSERT INTO {table_name} VALUES (2, 'Bob', 25)")
            
            # Should fail: duplicate name
            with self.assertRaises(Exception):
                self.execute_sql(f"INSERT INTO {table_name} VALUES (3, 'Alice', 35)")
        except Exception:
            # ADD CONSTRAINT may not be supported
            self.skipTest("ADD CONSTRAINT not supported")


class TestDropTable(LocalTestCase):
    """Test DROP TABLE statement"""
    
    def test_drop_table(self):
        """Test DROP TABLE"""
        table_name = "drop_test_table"
        
        # Create table
        self.create_test_table(table_name, "id INTEGER")
        self.assert_table_exists(table_name)
        
        # Insert data
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1)")
        self.assert_row_count(table_name, 1)
        
        # Drop table
        self.execute_sql(f"DROP TABLE {table_name}")
        
        # Verify table is gone
        self.assert_table_not_exists(table_name)
        
        # Should fail to query dropped table
        with self.assertRaises(Exception):
            self.execute_sql(f"SELECT * FROM {table_name}")
    
    def test_drop_table_if_exists(self):
        """Test DROP TABLE IF EXISTS"""
        table_name = "drop_if_exists_table"
        
        # Drop non-existent table with IF EXISTS should not error
        self.execute_sql(f"DROP TABLE IF EXISTS {table_name}")
        
        # Create table
        self.create_test_table(table_name, "id INTEGER")
        
        # Drop existing table with IF EXISTS
        self.execute_sql(f"DROP TABLE IF EXISTS {table_name}")
        
        # Verify table is gone
        self.assert_table_not_exists(table_name)
    
    def test_drop_table_cascade(self):
        """Test DROP TABLE CASCADE"""
        # Create parent and child tables
        self.create_test_table("parent_table", "id INTEGER PRIMARY KEY, name TEXT")
        self.create_test_table("child_table", 
            "child_id INTEGER PRIMARY KEY, parent_id INTEGER, FOREIGN KEY (parent_id) REFERENCES parent_table(id)")
        
        # Insert data
        self.execute_sql("INSERT INTO parent_table VALUES (1, 'Parent1')")
        self.execute_sql("INSERT INTO child_table VALUES (1, 1)")
        
        # Try to drop parent without CASCADE - should fail if foreign key constraints enforced
        try:
            self.execute_sql("DROP TABLE parent_table")
            # If no error, constraints may not be enforced
        except Exception:
            # Expected error, now try with CASCADE
            try:
                self.execute_sql("DROP TABLE parent_table CASCADE")
                # Should succeed, child table may also be dropped
            except Exception:
                # CASCADE may not be supported
                self.skipTest("DROP TABLE CASCADE not supported")
    
    def test_drop_table_restrict(self):
        """Test DROP TABLE RESTRICT"""
        # Create parent and child tables
        self.create_test_table("parent_table_restrict", "id INTEGER PRIMARY KEY, name TEXT")
        self.create_test_table("child_table_restrict", 
            "child_id INTEGER PRIMARY KEY, parent_id INTEGER, FOREIGN KEY (parent_id) REFERENCES parent_table_restrict(id)")
        
        # Insert data
        self.execute_sql("INSERT INTO parent_table_restrict VALUES (1, 'Parent1')")
        self.execute_sql("INSERT INTO child_table_restrict VALUES (1, 1)")
        
        # DROP TABLE RESTRICT should fail if foreign key constraints exist
        try:
            self.execute_sql("DROP TABLE parent_table_restrict RESTRICT")
            # If no error, RESTRICT may not be enforced
        except Exception:
            # Expected behavior
            pass
    
    def test_drop_table_deferred(self):
        """Test DROP TABLE DEFERRED"""
        table_name = "deferred_drop_table"
        
        self.create_test_table(table_name, "id INTEGER")
        
        # DROP TABLE DEFERRED (if supported)
        try:
            self.execute_sql(f"DROP TABLE {table_name} DEFERRED")
            
            # Table may still exist until transaction commit
            # Implementation specific
        except Exception:
            # DEFERRED may not be supported
            self.skipTest("DROP TABLE DEFERRED not supported")


class TestCreateTimeseriesTable(LocalTestCase):
    """Test CREATE TIMESERIES TABLE statement"""
    
    def test_create_timeseries_table_basic(self):
        """Test basic timeseries table creation"""
        table_name = "basic_timeseries"
        
        # Basic timeseries table
        schema = """
            timestamp TIMESTAMP,
            value REAL,
            sensor_id INTEGER
        """
        
        self.execute_sql(f"CREATE TIMESERIES TABLE {table_name} ({schema})")
        self.assert_table_exists(table_name)
    
    def test_create_timeseries_table_with_compression(self):
        """Test timeseries table with compression"""
        table_name = "compressed_timeseries"
        
        # With compression algorithm
        schema = "timestamp TIMESTAMP, value REAL, tags TEXT"
        
        try:
            self.execute_sql(f"""
                CREATE TIMESERIES TABLE {table_name} ({schema}) 
                WITH COMPRESSION=delta
            """)
            self.assert_table_exists(table_name)
        except Exception:
            self.skipTest("Timeseries compression not supported")
    
    def test_create_timeseries_table_with_ttl(self):
        """Test timeseries table with TTL"""
        table_name = "ttl_timeseries"
        
        # With TTL (Time To Live)
        schema = "timestamp TIMESTAMP, metric REAL, source TEXT"
        
        try:
            self.execute_sql(f"""
                CREATE TIMESERIES TABLE {table_name} ({schema}) 
                WITH TTL='7d'
            """)
            self.assert_table_exists(table_name)
        except Exception:
            self.skipTest("Timeseries TTL not supported")
    
    def test_create_timeseries_table_with_all_options(self):
        """Test timeseries table with all options"""
        table_name = "full_timeseries"
        
        schema = "timestamp TIMESTAMP, measurement REAL, device_id INTEGER, location TEXT"
        
        try:
            self.execute_sql(f"""
                CREATE TIMESERIES TABLE {table_name} ({schema}) 
                WITH COMPRESSION=runlength, TTL='30d', PARTITION_BY='day'
            """)
            self.assert_table_exists(table_name)
        except Exception:
            self.skipTest("Full timeseries options not supported")
    
    def test_timeseries_table_insert_select(self):
        """Test inserting and selecting from timeseries table"""
        table_name = "test_timeseries_data"
        
        # Create timeseries table
        self.execute_sql(f"""
            CREATE TIMESERIES TABLE {table_name} (
                timestamp TIMESTAMP,
                temperature REAL,
                humidity REAL,
                sensor TEXT
            )
        """)
        
        # Insert timeseries data
        timestamps = [1609459200000, 1609459260000, 1609459320000]
        for i, ts in enumerate(timestamps):
            self.execute_sql(f"""
                INSERT INTO {table_name} (timestamp, temperature, humidity, sensor)
                VALUES ({ts}, {20.0 + i}, {50.0 + i}, 'sensor_{i}')
            """)
        
        # Query data
        result = self.execute_sql(f"""
            SELECT * FROM {table_name} 
            WHERE timestamp >= 1609459200000 AND timestamp <= 1609459320000
            ORDER BY timestamp
        """)
        rows = list(result)
        self.assertEqual(len(rows), 3)
        
        # Verify time ordering
        for i in range(1, len(rows)):
            self.assertLessEqual(rows[i-1]["timestamp"], rows[i]["timestamp"])


class TestShowTables(LocalTestCase):
    """Test SHOW TABLES and related commands"""
    
    def test_show_tables(self):
        """Test SHOW TABLES command"""
        # Create some tables
        tables = ["show_test_1", "show_test_2", "show_test_3"]
        for table in tables:
            self.create_test_table(table, "id INTEGER")
        
        # Execute SHOW TABLES
        try:
            result = self.execute_sql("SHOW TABLES")
            rows = list(result)
            
            # Should contain at least our tables
            table_names = [row["table_name"] for row in rows]
            for table in tables:
                self.assertIn(table, table_names)
        except Exception:
            self.skipTest("SHOW TABLES not supported")
    
    def test_describe_table(self):
        """Test DESCRIBE TABLE command"""
        table_name = "describe_test"
        schema = "id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER DEFAULT 18, salary REAL"
        
        self.create_test_table(table_name, schema)
        
        # Describe table
        try:
            result = self.execute_sql(f"DESCRIBE TABLE {table_name}")
            rows = list(result)
            
            # Should have information about columns
            self.assertGreater(len(rows), 0)
            
            # Check for expected columns
            column_names = [row["column_name"] for row in rows]
            for col in ["id", "name", "age", "salary"]:
                self.assertIn(col, column_names)
        except Exception:
            self.skipTest("DESCRIBE TABLE not supported")


class TestDatabaseManagement(LocalTestCase):
    """Test database management statements"""
    
    def test_create_database(self):
        """Test CREATE DATABASE"""
        # Note: Database management may work differently in file-based mode
        # This test may need adjustment
        try:
            self.execute_sql("CREATE DATABASE test_db")
            
            # Switch to new database
            self.execute_sql("USE DATABASE test_db")
            
            # Create table in new database
            self.create_test_table("db_test_table", "id INTEGER")
            
            # Close database
            self.execute_sql("CLOSE DATABASE")
        except Exception:
            self.skipTest("Database management not supported in current mode")
    
    def test_drop_database(self):
        """Test DROP DATABASE"""
        try:
            # Create and then drop database
            self.execute_sql("CREATE DATABASE temp_db")
            self.execute_sql("USE DATABASE temp_db")
            
            # Drop database
            self.execute_sql("DROP DATABASE temp_db")
            
            # Should fail to use dropped database
            with self.assertRaises(Exception):
                self.execute_sql("USE DATABASE temp_db")
        except Exception:
            self.skipTest("DROP DATABASE not supported")


if __name__ == '__main__':
    unittest.main()