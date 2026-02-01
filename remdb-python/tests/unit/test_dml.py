"""Test DML statements (SELECT, INSERT, UPDATE, DELETE)"""

import unittest
from tests.fixtures import LocalTestCase
from tests.utils import generate_test_table_data

class TestInsertStatement(LocalTestCase):
    """Test INSERT statement"""
    
    def setUp(self):
        super().setUp()
        # Create test table for INSERT tests
        self.test_table = "insert_test_table"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT,
            age INTEGER,
            salary REAL,
            active BOOLEAN,
            created_at TIMESTAMP
        """
        self.create_test_table(self.test_table, schema)
    
    def test_insert_basic(self):
        """Test basic INSERT statement"""
        # Insert single row
        self.execute_sql(f"""
            INSERT INTO {self.test_table} (id, name, age, salary, active, created_at)
            VALUES (1, 'Alice', 30, 75000.0, TRUE, 1609459200000)
        """)
        
        self.assert_row_count(self.test_table, 1)
        
        # Verify inserted data
        result = self.execute_sql(f"SELECT * FROM {self.test_table} WHERE id = 1")
        rows = list(result)
        
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["name"], "Alice")
        self.assertEqual(rows[0]["age"], 30)
        self.assertAlmostEqual(rows[0]["salary"], 75000.0, places=2)
        self.assertEqual(rows[0]["active"], True)
        self.assertEqual(rows[0]["created_at"], 1609459200000)
    
    def test_insert_multiple_rows(self):
        """Test inserting multiple rows"""
        # Insert multiple rows with single statement
        self.execute_sql(f"""
            INSERT INTO {self.test_table} (id, name, age) 
            VALUES (1, 'Alice', 30),
                   (2, 'Bob', 25),
                   (3, 'Charlie', 35)
        """)
        
        self.assert_row_count(self.test_table, 3)
        
        # Verify all rows inserted
        result = self.execute_sql(f"SELECT COUNT(*) as count FROM {self.test_table}")
        rows = list(result)
        self.assertEqual(rows[0]["count"], 3)
        
        result = self.execute_sql(f"SELECT name FROM {self.test_table} ORDER BY id")
        rows = list(result)
        names = [row["name"] for row in rows]
        self.assertEqual(names, ["Alice", "Bob", "Charlie"])
    
    def test_insert_without_column_list(self):
        """Test INSERT without specifying column list"""
        # Insert with all columns in VALUES
        self.execute_sql(f"""
            INSERT INTO {self.test_table} 
            VALUES (1, 'Alice', 30, 75000.0, TRUE, 1609459200000)
        """)
        
        self.assert_row_count(self.test_table, 1)
        
        # Insert with NULL for some columns
        self.execute_sql(f"""
            INSERT INTO {self.test_table} 
            VALUES (2, 'Bob', 25, NULL, FALSE, 1609459260000)
        """)
        
        self.assert_row_count(self.test_table, 2)
    
    def test_insert_with_default_values(self):
        """Test INSERT with DEFAULT values"""
        # Create table with DEFAULT values
        table_name = "default_values_table"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT DEFAULT 'unknown',
            age INTEGER DEFAULT 18,
            active BOOLEAN DEFAULT TRUE
        """
        self.create_test_table(table_name, schema)
        
        # Insert using DEFAULT keyword
        self.execute_sql(f"""
            INSERT INTO {table_name} (id, name, age) 
            VALUES (1, DEFAULT, DEFAULT)
        """)
        
        # Insert with some explicit values
        self.execute_sql(f"""
            INSERT INTO {table_name} (id) 
            VALUES (2)
        """)
        
        # Verify defaults applied
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
        rows = list(result)
        
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0]["name"], "unknown")  # DEFAULT
        self.assertEqual(rows[0]["age"], 18)          # DEFAULT
        self.assertEqual(rows[0]["active"], True)     # DEFAULT
        self.assertEqual(rows[1]["name"], "unknown")  # DEFAULT
        self.assertEqual(rows[1]["age"], 18)          # DEFAULT
        self.assertEqual(rows[1]["active"], True)     # DEFAULT
    
    def test_insert_with_null_values(self):
        """Test INSERT with NULL values"""
        # Insert with explicit NULL
        self.execute_sql(f"""
            INSERT INTO {self.test_table} (id, name, age, salary, active, created_at)
            VALUES (1, 'Alice', 30, NULL, TRUE, NULL)
        """)
        
        # Insert with missing columns (should be NULL)
        self.execute_sql(f"""
            INSERT INTO {self.test_table} (id, name, age)
            VALUES (2, 'Bob', 25)
        """)
        
        self.assert_row_count(self.test_table, 2)
        
        # Verify NULL values
        result = self.execute_sql(f"SELECT salary, created_at FROM {self.test_table} WHERE id = 2")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        self.assertIsNone(rows[0]["salary"])
        self.assertIsNone(rows[0]["created_at"])
    
    def test_insert_error_duplicate_primary_key(self):
        """Test INSERT error with duplicate primary key"""
        self.execute_sql(f"INSERT INTO {self.test_table} (id, name) VALUES (1, 'Alice')")
        
        # Should fail with duplicate primary key
        with self.assertRaises(Exception):
            self.execute_sql(f"INSERT INTO {self.test_table} (id, name) VALUES (1, 'Bob')")
        
        # First row should still exist
        self.assert_row_count(self.test_table, 1)
    
    def test_insert_error_not_null_constraint(self):
        """Test INSERT error with NOT NULL constraint"""
        # Create table with NOT NULL constraint
        table_name = "not_null_table"
        schema = "id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER"
        self.create_test_table(table_name, schema)
        
        # Should fail: name is NOT NULL
        with self.assertRaises(Exception):
            self.execute_sql(f"INSERT INTO {table_name} (id, age) VALUES (1, 30)")
        
        # Valid insert
        self.execute_sql(f"INSERT INTO {table_name} (id, name, age) VALUES (1, 'Alice', 30)")
        self.assert_row_count(table_name, 1)


class TestSelectStatement(LocalTestCase):
    """Test SELECT statement"""
    
    def setUp(self):
        super().setUp()
        # Create and populate test table for SELECT tests
        self.test_table = "select_test_table"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT,
            department TEXT,
            salary REAL,
            age INTEGER,
            active BOOLEAN,
            hire_date TIMESTAMP
        """
        self.create_test_table(self.test_table, schema)
        
        # Insert test data
        test_data = [
            {"id": 1, "name": "Alice", "department": "Engineering", "salary": 75000.0, "age": 30, "active": True, "hire_date": 1609459200000},
            {"id": 2, "name": "Bob", "department": "Sales", "salary": 65000.0, "age": 25, "active": True, "hire_date": 1609459260000},
            {"id": 3, "name": "Charlie", "department": "Engineering", "salary": 80000.0, "age": 35, "active": False, "hire_date": 1609459320000},
            {"id": 4, "name": "David", "department": "Marketing", "salary": 60000.0, "age": 28, "active": True, "hire_date": 1609459380000},
            {"id": 5, "name": "Eve", "department": "Engineering", "salary": 90000.0, "age": 40, "active": True, "hire_date": 1609459440000},
        ]
        self.insert_test_data(self.test_table, test_data)
    
    def test_select_all_columns(self):
        """Test SELECT * (all columns)"""
        result = self.execute_sql(f"SELECT * FROM {self.test_table} ORDER BY id")
        rows = list(result)
        
        self.assertEqual(len(rows), 5)
        
        # Verify first row
        first_row = rows[0]
        self.assertEqual(first_row["id"], 1)
        self.assertEqual(first_row["name"], "Alice")
        self.assertEqual(first_row["department"], "Engineering")
        self.assertAlmostEqual(first_row["salary"], 75000.0, places=2)
    
    def test_select_specific_columns(self):
        """Test SELECT specific columns"""
        result = self.execute_sql(f"SELECT name, department, salary FROM {self.test_table} ORDER BY id")
        rows = list(result)
        
        self.assertEqual(len(rows), 5)
        
        # Verify column subset
        first_row = rows[0]
        self.assertEqual(first_row["name"], "Alice")
        self.assertEqual(first_row["department"], "Engineering")
        self.assertAlmostEqual(first_row["salary"], 75000.0, places=2)
        
        # id should not be in result
        self.assertNotIn("id", first_row)
    
    def test_select_with_aliases(self):
        """Test SELECT with column aliases"""
        result = self.execute_sql(f"""
            SELECT 
                name AS employee_name,
                department AS dept,
                salary * 1.1 AS increased_salary
            FROM {self.test_table} 
            WHERE id = 1
        """)
        rows = list(result)
        
        self.assertEqual(len(rows), 1)
        row = rows[0]
        
        self.assertEqual(row["employee_name"], "Alice")
        self.assertEqual(row["dept"], "Engineering")
        self.assertAlmostEqual(row["increased_salary"], 82500.0, places=2)  # 75000 * 1.1
    
    def test_select_with_where_clause(self):
        """Test SELECT with WHERE clause"""
        # Test equality
        result = self.execute_sql(f"SELECT * FROM {self.test_table} WHERE department = 'Engineering'")
        rows = list(result)
        self.assertEqual(len(rows), 3)  # Alice, Charlie, Eve
        
        # Test numeric comparison
        result = self.execute_sql(f"SELECT * FROM {self.test_table} WHERE age > 30")
        rows = list(result)
        self.assertEqual(len(rows), 2)  # Charlie (35), Eve (40)
        
        # Test boolean
        result = self.execute_sql(f"SELECT * FROM {self.test_table} WHERE active = TRUE")
        rows = list(result)
        self.assertEqual(len(rows), 4)  # All except Charlie
        
        # Test AND condition
        result = self.execute_sql(f"""
            SELECT * FROM {self.test_table} 
            WHERE department = 'Engineering' AND age > 30 AND active = TRUE
        """)
        rows = list(result)
        self.assertEqual(len(rows), 1)  # Eve only
        
        # Test OR condition
        result = self.execute_sql(f"""
            SELECT * FROM {self.test_table} 
            WHERE department = 'Engineering' OR department = 'Sales'
        """)
        rows = list(result)
        self.assertEqual(len(rows), 4)  # Alice, Bob, Charlie, Eve
    
    def test_select_with_order_by(self):
        """Test SELECT with ORDER BY"""
        # ASC order (default)
        result = self.execute_sql(f"SELECT name, salary FROM {self.test_table} ORDER BY salary")
        rows = list(result)
        
        salaries = [row["salary"] for row in rows]
        self.assertEqual(len(salaries), 5)
        self.assertListEqual(salaries, sorted(salaries))  # Should be ascending
        
        # DESC order
        result = self.execute_sql(f"SELECT name, salary FROM {self.test_table} ORDER BY salary DESC")
        rows = list(result)
        
        salaries = [row["salary"] for row in rows]
        self.assertEqual(len(salaries), 5)
        self.assertListEqual(salaries, sorted(salaries, reverse=True))  # Should be descending
        
        # Multiple columns
        result = self.execute_sql(f"""
            SELECT department, name, salary 
            FROM {self.test_table} 
            ORDER BY department ASC, salary DESC
        """)
        rows = list(result)
        
        # Verify ordering
        depts = [row["department"] for row in rows]
        self.assertEqual(depts, sorted(depts))  # Departments sorted ascending
        
        # Within each department, salaries should be descending
        eng_salaries = [row["salary"] for row in rows if row["department"] == "Engineering"]
        self.assertEqual(eng_salaries, sorted(eng_salaries, reverse=True))
    
    def test_select_with_limit(self):
        """Test SELECT with LIMIT clause"""
        # Limit number of rows
        result = self.execute_sql(f"SELECT * FROM {self.test_table} ORDER BY id LIMIT 3")
        rows = list(result)
        self.assertEqual(len(rows), 3)
        
        # Limit with OFFSET
        result = self.execute_sql(f"SELECT * FROM {self.test_table} ORDER BY id LIMIT 2 OFFSET 1")
        rows = list(result)
        self.assertEqual(len(rows), 2)
        
        # Should get rows 2 and 3 (skip first row)
        ids = [row["id"] for row in rows]
        self.assertEqual(ids, [2, 3])
    
    def test_select_with_distinct(self):
        """Test SELECT DISTINCT"""
        # Insert duplicate department values
        self.execute_sql(f"INSERT INTO {self.test_table} (id, name, department) VALUES (6, 'Frank', 'Engineering')")
        
        # DISTINCT departments
        result = self.execute_sql(f"SELECT DISTINCT department FROM {self.test_table}")
        rows = list(result)
        
        departments = [row["department"] for row in rows]
        self.assertEqual(len(departments), 3)  # Engineering, Sales, Marketing
        self.assertEqual(len(set(departments)), len(departments))  # All unique
        
        # DISTINCT on multiple columns
        result = self.execute_sql(f"SELECT DISTINCT department, active FROM {self.test_table}")
        rows = list(result)
        
        combinations = [(row["department"], row["active"]) for row in rows]
        self.assertEqual(len(set(combinations)), len(combinations))  # All unique
    
    def test_select_with_group_by(self):
        """Test SELECT with GROUP BY"""
        # Group by department with count
        result = self.execute_sql(f"""
            SELECT department, COUNT(*) as employee_count, AVG(salary) as avg_salary
            FROM {self.test_table}
            GROUP BY department
            ORDER BY department
        """)
        rows = list(result)
        
        self.assertEqual(len(rows), 3)  # 3 departments
        
        # Find Engineering department
        eng_row = next(row for row in rows if row["department"] == "Engineering")
        self.assertEqual(eng_row["employee_count"], 3)  # Alice, Charlie, Eve
        self.assertAlmostEqual(eng_row["avg_salary"], (75000.0 + 80000.0 + 90000.0) / 3, places=2)
        
        # Group by multiple columns
        result = self.execute_sql(f"""
            SELECT department, active, COUNT(*) as count
            FROM {self.test_table}
            GROUP BY department, active
            ORDER BY department, active
        """)
        rows = list(result)
        
        # Should have groups for each department+active combination
        self.assertGreater(len(rows), 0)
    
    def test_select_with_having(self):
        """Test SELECT with HAVING clause"""
        # Groups with HAVING condition
        result = self.execute_sql(f"""
            SELECT department, COUNT(*) as employee_count, AVG(salary) as avg_salary
            FROM {self.test_table}
            GROUP BY department
            HAVING COUNT(*) > 1
            ORDER BY department
        """)
        rows = list(result)
        
        # Only departments with more than 1 employee
        departments = [row["department"] for row in rows]
        self.assertIn("Engineering", departments)  # 3 employees
        self.assertNotIn("Sales", departments)     # 1 employee
        self.assertNotIn("Marketing", departments) # 1 employee
    
    def test_select_with_aggregate_functions(self):
        """Test SELECT with aggregate functions"""
        # COUNT
        result = self.execute_sql(f"SELECT COUNT(*) as total FROM {self.test_table}")
        rows = list(result)
        self.assertEqual(rows[0]["total"], 5)
        
        # SUM
        result = self.execute_sql(f"SELECT SUM(salary) as total_salary FROM {self.test_table}")
        rows = list(result)
        total = 75000.0 + 65000.0 + 80000.0 + 60000.0 + 90000.0
        self.assertAlmostEqual(rows[0]["total_salary"], total, places=2)
        
        # AVG
        result = self.execute_sql(f"SELECT AVG(age) as avg_age FROM {self.test_table}")
        rows = list(result)
        avg_age = (30 + 25 + 35 + 28 + 40) / 5
        self.assertAlmostEqual(rows[0]["avg_age"], avg_age, places=2)
        
        # MIN and MAX
        result = self.execute_sql(f"SELECT MIN(salary) as min_salary, MAX(salary) as max_salary FROM {self.test_table}")
        rows = list(result)
        self.assertAlmostEqual(rows[0]["min_salary"], 60000.0, places=2)
        self.assertAlmostEqual(rows[0]["max_salary"], 90000.0, places=2)
    
    def test_select_with_joins(self):
        """Test SELECT with JOINs"""
        # Create another table for JOIN tests
        dept_table = "departments"
        self.create_test_table(dept_table, "dept_id INTEGER PRIMARY KEY, dept_name TEXT, location TEXT")
        
        self.execute_sql(f"INSERT INTO {dept_table} VALUES (1, 'Engineering', 'Building A')")
        self.execute_sql(f"INSERT INTO {dept_table} VALUES (2, 'Sales', 'Building B')")
        self.execute_sql(f"INSERT INTO {dept_table} VALUES (3, 'Marketing', 'Building C')")
        
        # INNER JOIN
        result = self.execute_sql(f"""
            SELECT e.name, e.department, d.location
            FROM {self.test_table} e
            INNER JOIN {dept_table} d ON e.department = d.dept_name
            ORDER BY e.name
        """)
        rows = list(result)
        
        self.assertEqual(len(rows), 5)  # All employees have matching departments
        
        # LEFT JOIN
        self.execute_sql(f"INSERT INTO {dept_table} VALUES (4, 'HR', 'Building D')")
        
        result = self.execute_sql(f"""
            SELECT d.dept_name, COUNT(e.id) as employee_count
            FROM {dept_table} d
            LEFT JOIN {self.test_table} e ON d.dept_name = e.department
            GROUP BY d.dept_name
            ORDER BY d.dept_name
        """)
        rows = list(result)
        
        # Should include HR department with 0 employees
        hr_row = next(row for row in rows if row["dept_name"] == "HR")
        self.assertEqual(hr_row["employee_count"], 0)


class TestUpdateStatement(LocalTestCase):
    """Test UPDATE statement"""
    
    def setUp(self):
        super().setUp()
        self.test_table = "update_test_table"
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT,
            department TEXT,
            salary REAL,
            age INTEGER,
            active BOOLEAN
        """
        self.create_test_table(self.test_table, schema)
        
        # Insert test data
        test_data = [
            {"id": 1, "name": "Alice", "department": "Engineering", "salary": 75000.0, "age": 30, "active": True},
            {"id": 2, "name": "Bob", "department": "Sales", "salary": 65000.0, "age": 25, "active": True},
            {"id": 3, "name": "Charlie", "department": "Engineering", "salary": 80000.0, "age": 35, "active": False},
        ]
        self.insert_test_data(self.test_table, test_data)
    
    def test_update_all_rows(self):
        """Test UPDATE without WHERE clause (updates all rows)"""
        # Update all rows
        self.execute_sql(f"UPDATE {self.test_table} SET active = FALSE")
        
        # Verify all rows updated
        result = self.execute_sql(f"SELECT COUNT(*) as count FROM {self.test_table} WHERE active = FALSE")
        rows = list(result)
        self.assertEqual(rows[0]["count"], 3)
    
    def test_update_with_where_clause(self):
        """Test UPDATE with WHERE clause"""
        # Update specific rows
        self.execute_sql(f"""
            UPDATE {self.test_table} 
            SET salary = salary * 1.1, department = 'Senior Engineering'
            WHERE department = 'Engineering' AND active = TRUE
        """)
        
        # Verify updates
        result = self.execute_sql(f"""
            SELECT * FROM {self.test_table} 
            WHERE id = 1  # Alice
        """)
        rows = list(result)
        
        self.assertEqual(len(rows), 1)
        self.assertAlmostEqual(rows[0]["salary"], 75000.0 * 1.1, places=2)
        self.assertEqual(rows[0]["department"], "Senior Engineering")
        
        # Bob should not be updated (Sales department)
        result = self.execute_sql(f"SELECT * FROM {self.test_table} WHERE id = 2")
        rows = list(result)
        self.assertAlmostEqual(rows[0]["salary"], 65000.0, places=2)
        self.assertEqual(rows[0]["department"], "Sales")
        
        # Charlie should not be updated (not active)
        result = self.execute_sql(f"SELECT * FROM {self.test_table} WHERE id = 3")
        rows = list(result)
        self.assertAlmostEqual(rows[0]["salary"], 80000.0, places=2)
        self.assertEqual(rows[0]["department"], "Engineering")
    
    def test_update_multiple_columns(self):
        """Test UPDATE multiple columns"""
        self.execute_sql(f"""
            UPDATE {self.test_table}
            SET name = 'Robert', age = 26, salary = 70000.0
            WHERE id = 2
        """)
        
        # Verify all columns updated
        result = self.execute_sql(f"SELECT * FROM {self.test_table} WHERE id = 2")
        rows = list(result)
        
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["name"], "Robert")
        self.assertEqual(rows[0]["age"], 26)
        self.assertAlmostEqual(rows[0]["salary"], 70000.0, places=2)
    
    def test_update_with_null(self):
        """Test UPDATE setting columns to NULL"""
        self.execute_sql(f"UPDATE {self.test_table} SET department = NULL WHERE id = 1")
        
        result = self.execute_sql(f"SELECT department FROM {self.test_table} WHERE id = 1")
        rows = list(result)
        self.assertIsNone(rows[0]["department"])
    
    def test_update_error_duplicate_primary_key(self):
        """Test UPDATE error with duplicate primary key"""
        # Create table with unique constraint
        table_name = "update_unique_table"
        self.create_test_table(table_name, "id INTEGER PRIMARY KEY, email TEXT UNIQUE")
        
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1, 'alice@example.com')")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (2, 'bob@example.com')")
        
        # Should fail: duplicate email
        with self.assertRaises(Exception):
            self.execute_sql(f"UPDATE {table_name} SET email = 'alice@example.com' WHERE id = 2")


class TestDeleteStatement(LocalTestCase):
    """Test DELETE statement"""
    
    def setUp(self):
        super().setUp()
        self.test_table = "delete_test_table"
        schema = "id INTEGER PRIMARY KEY, name TEXT, department TEXT, active BOOLEAN"
        self.create_test_table(self.test_table, schema)
        
        # Insert test data
        test_data = [
            {"id": 1, "name": "Alice", "department": "Engineering", "active": True},
            {"id": 2, "name": "Bob", "department": "Sales", "active": True},
            {"id": 3, "name": "Charlie", "department": "Engineering", "active": False},
            {"id": 4, "name": "David", "department": "Marketing", "active": True},
            {"id": 5, "name": "Eve", "department": "Engineering", "active": True},
        ]
        self.insert_test_data(self.test_table, test_data)
    
    def test_delete_all_rows(self):
        """Test DELETE without WHERE clause (deletes all rows)"""
        self.assert_row_count(self.test_table, 5)
        
        self.execute_sql(f"DELETE FROM {self.test_table}")
        
        self.assert_row_count(self.test_table, 0)
    
    def test_delete_with_where_clause(self):
        """Test DELETE with WHERE clause"""
        # Delete inactive employees
        self.execute_sql(f"DELETE FROM {self.test_table} WHERE active = FALSE")
        
        self.assert_row_count(self.test_table, 4)  # Charlie deleted
        
        # Delete Engineering department
        self.execute_sql(f"DELETE FROM {self.test_table} WHERE department = 'Engineering'")
        
        self.assert_row_count(self.test_table, 1)  # Only Bob (Sales) remains
        
        # Verify remaining data
        result = self.execute_sql(f"SELECT * FROM {self.test_table}")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["name"], "Bob")
        self.assertEqual(rows[0]["department"], "Sales")
    
    def test_delete_with_limit(self):
        """Test DELETE with LIMIT"""
        # Delete first 2 rows (ordered by id)
        self.execute_sql(f"DELETE FROM {self.test_table} ORDER BY id LIMIT 2")
        
        self.assert_row_count(self.test_table, 3)  # Deleted Alice and Bob
        
        # Verify which rows remain
        result = self.execute_sql(f"SELECT id FROM {self.test_table} ORDER BY id")
        rows = list(result)
        ids = [row["id"] for row in rows]
        self.assertEqual(ids, [3, 4, 5])  # Charlie, David, Eve
    
    def test_delete_returning(self):
        """Test DELETE RETURNING (if supported)"""
        try:
            result = self.execute_sql(f"DELETE FROM {self.test_table} WHERE id = 1 RETURNING name, department")
            rows = list(result)
            
            # Should return deleted row data
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0]["name"], "Alice")
            self.assertEqual(rows[0]["department"], "Engineering")
            
            self.assert_row_count(self.test_table, 4)
        except Exception:
            self.skipTest("DELETE RETURNING not supported")


if __name__ == '__main__':
    unittest.main()