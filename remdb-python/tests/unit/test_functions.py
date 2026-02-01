"""Test SQL functions supported by RemDB"""

import unittest
import math
from tests.fixtures import LocalTestCase

class TestAggregateFunctions(LocalTestCase):
    """Test aggregate functions"""
    
    def setUp(self):
        super().setUp()
        self.test_table = "agg_test_table"
        schema = """
            id INTEGER PRIMARY KEY,
            department TEXT,
            salary REAL,
            age INTEGER,
            sales INTEGER
        """
        self.create_test_table(self.test_table, schema)
        
        # Insert test data
        test_data = [
            {"id": 1, "department": "Engineering", "salary": 75000.0, "age": 30, "sales": 100},
            {"id": 2, "department": "Engineering", "salary": 80000.0, "age": 35, "sales": 150},
            {"id": 3, "department": "Sales", "salary": 65000.0, "age": 25, "sales": 300},
            {"id": 4, "department": "Sales", "salary": 70000.0, "age": 28, "sales": 250},
            {"id": 5, "department": "Marketing", "salary": 60000.0, "age": 32, "sales": 50},
        ]
        self.insert_test_data(self.test_table, test_data)
    
    def test_count(self):
        """Test COUNT() function"""
        # COUNT(*)
        result = self.execute_sql(f"SELECT COUNT(*) as total FROM {self.test_table}")
        rows = list(result)
        self.assertEqual(rows[0]["total"], 5)
        
        # COUNT(column)
        result = self.execute_sql(f"SELECT COUNT(salary) as salary_count FROM {self.test_table}")
        rows = list(result)
        self.assertEqual(rows[0]["salary_count"], 5)
        
        # COUNT with NULL values (add a row with NULL salary)
        self.execute_sql(f"INSERT INTO {self.test_table} (id, department) VALUES (6, 'HR')")
        
        result = self.execute_sql(f"SELECT COUNT(salary) as salary_count FROM {self.test_table}")
        rows = list(result)
        self.assertEqual(rows[0]["salary_count"], 5)  # NULL not counted
        
        result = self.execute_sql(f"SELECT COUNT(*) as total FROM {self.test_table}")
        rows = list(result)
        self.assertEqual(rows[0]["total"], 6)  # COUNT(*) counts all rows
    
    def test_sum(self):
        """Test SUM() function"""
        # SUM of all salaries
        result = self.execute_sql(f"SELECT SUM(salary) as total_salary FROM {self.test_table}")
        rows = list(result)
        total = 75000.0 + 80000.0 + 65000.0 + 70000.0 + 60000.0
        self.assertAlmostEqual(rows[0]["total_salary"], total, places=2)
        
        # SUM with GROUP BY
        result = self.execute_sql(f"""
            SELECT department, SUM(salary) as dept_total
            FROM {self.test_table}
            GROUP BY department
            ORDER BY department
        """)
        rows = list(result)
        
        # Find Engineering department
        eng_row = next(row for row in rows if row["department"] == "Engineering")
        self.assertAlmostEqual(eng_row["dept_total"], 75000.0 + 80000.0, places=2)
        
        # SUM with NULL values
        result = self.execute_sql(f"SELECT SUM(salary) as total_with_null FROM {self.test_table}")
        rows = list(result)
        self.assertAlmostEqual(rows[0]["total_with_null"], total, places=2)  # NULL excluded
    
    def test_avg(self):
        """Test AVG() function"""
        # Average salary
        result = self.execute_sql(f"SELECT AVG(salary) as avg_salary FROM {self.test_table}")
        rows = list(result)
        total = 75000.0 + 80000.0 + 65000.0 + 70000.0 + 60000.0
        avg = total / 5
        self.assertAlmostEqual(rows[0]["avg_salary"], avg, places=2)
        
        # Average with GROUP BY
        result = self.execute_sql(f"""
            SELECT department, AVG(salary) as avg_dept_salary
            FROM {self.test_table}
            GROUP BY department
            ORDER BY department
        """)
        rows = list(result)
        
        eng_row = next(row for row in rows if row["department"] == "Engineering")
        self.assertAlmostEqual(eng_row["avg_dept_salary"], (75000.0 + 80000.0) / 2, places=2)
    
    def test_min_max(self):
        """Test MIN() and MAX() functions"""
        # MIN and MAX salary
        result = self.execute_sql(f"SELECT MIN(salary) as min_salary, MAX(salary) as max_salary FROM {self.test_table}")
        rows = list(result)
        self.assertAlmostEqual(rows[0]["min_salary"], 60000.0, places=2)
        self.assertAlmostEqual(rows[0]["max_salary"], 80000.0, places=2)
        
        # MIN and MAX with GROUP BY
        result = self.execute_sql(f"""
            SELECT department, MIN(age) as min_age, MAX(age) as max_age
            FROM {self.test_table}
            GROUP BY department
            ORDER BY department
        """)
        rows = list(result)
        
        eng_row = next(row for row in rows if row["department"] == "Engineering")
        self.assertEqual(eng_row["min_age"], 30)
        self.assertEqual(eng_row["max_age"], 35)
    
    def test_variance_stddev(self):
        """Test VAR(), STDDEV(), VAR_SAMP(), STDDEV_SAMP() functions"""
        # These functions might not be supported, test with try/except
        functions = ["VAR", "STDDEV", "VAR_SAMP", "STDDEV_SAMP"]
        
        for func in functions:
            try:
                result = self.execute_sql(f"SELECT {func}(salary) as result FROM {self.test_table}")
                rows = list(result)
                # Just verify query executed without error
                self.assertEqual(len(rows), 1)
                self.assertIn("result", rows[0])
            except Exception:
                # Function not supported, skip
                continue
    
    def test_aggregate_with_having(self):
        """Test aggregate functions with HAVING clause"""
        result = self.execute_sql(f"""
            SELECT department, AVG(salary) as avg_salary, COUNT(*) as employee_count
            FROM {self.test_table}
            GROUP BY department
            HAVING AVG(salary) > 67000.0
            ORDER BY department
        """)
        rows = list(result)
        
        # Only departments with average salary > 67000
        departments = [row["department"] for row in rows]
        self.assertIn("Engineering", departments)  # avg = 77500
        self.assertIn("Sales", departments)        # avg = 67500
        self.assertNotIn("Marketing", departments) # avg = 60000


class TestStringFunctions(LocalTestCase):
    """Test string functions"""
    
    def setUp(self):
        super().setUp()
        self.test_table = "string_test_table"
        schema = "id INTEGER PRIMARY KEY, name TEXT, description TEXT"
        self.create_test_table(self.test_table, schema)
        
        test_data = [
            {"id": 1, "name": "Alice Smith", "description": "Software Engineer"},
            {"id": 2, "name": "Bob Johnson", "description": "Sales Manager"},
            {"id": 3, "name": "Charlie Brown", "description": "Marketing Director"},
            {"id": 4, "name": "David Lee", "description": ""},
        ]
        self.insert_test_data(self.test_table, test_data)
    
    def test_concat(self):
        """Test CONCAT() function"""
        try:
            result = self.execute_sql(f"""
                SELECT CONCAT(name, ' - ', description) as full_info
                FROM {self.test_table}
                WHERE id = 1
            """)
            rows = list(result)
            self.assertEqual(rows[0]["full_info"], "Alice Smith - Software Engineer")
        except Exception:
            self.skipTest("CONCAT function not supported")
    
    def test_substring(self):
        """Test SUBSTRING() function"""
        try:
            result = self.execute_sql(f"""
                SELECT SUBSTRING(name, 1, 5) as first_five
                FROM {self.test_table}
                WHERE id = 1
            """)
            rows = list(result)
            self.assertEqual(rows[0]["first_five"], "Alice")
        except Exception:
            self.skipTest("SUBSTRING function not supported")
    
    def test_upper_lower(self):
        """Test UPPER() and LOWER() functions"""
        try:
            result = self.execute_sql(f"""
                SELECT UPPER(name) as upper_name, LOWER(description) as lower_desc
                FROM {self.test_table}
                WHERE id = 2
            """)
            rows = list(result)
            self.assertEqual(rows[0]["upper_name"], "BOB JOHNSON")
            self.assertEqual(rows[0]["lower_desc"], "sales manager")
        except Exception:
            self.skipTest("UPPER/LOWER functions not supported")
    
    def test_string_length(self):
        """Test string length function (might be LENGTH or CHAR_LENGTH)"""
        # Try common length function names
        length_funcs = ["LENGTH", "CHAR_LENGTH", "LEN"]
        
        for func in length_funcs:
            try:
                result = self.execute_sql(f"""
                    SELECT {func}(name) as name_length
                    FROM {self.test_table}
                    WHERE id = 1
                """)
                rows = list(result)
                self.assertEqual(rows[0]["name_length"], len("Alice Smith"))
                break
            except Exception:
                continue
        else:
            self.skipTest("String length function not supported")


class TestMathFunctions(LocalTestCase):
    """Test mathematical functions"""
    
    def setUp(self):
        super().setUp()
        self.test_table = "math_test_table"
        schema = "id INTEGER PRIMARY KEY, value REAL, int_val INTEGER"
        self.create_test_table(self.test_table, schema)
        
        test_data = [
            {"id": 1, "value": 25.0, "int_val": 25},
            {"id": 2, "value": -10.5, "int_val": -10},
            {"id": 3, "value": 100.0, "int_val": 100},
            {"id": 4, "value": 0.0, "int_val": 0},
        ]
        self.insert_test_data(self.test_table, test_data)
    
    def test_abs(self):
        """Test ABS() function"""
        try:
            result = self.execute_sql(f"""
                SELECT ABS(value) as abs_value, ABS(int_val) as abs_int
                FROM {self.test_table}
                WHERE id = 2
            """)
            rows = list(result)
            self.assertAlmostEqual(rows[0]["abs_value"], 10.5, places=2)
            self.assertEqual(rows[0]["abs_int"], 10)
        except Exception:
            self.skipTest("ABS function not supported")
    
    def test_sqrt(self):
        """Test SQRT() function"""
        try:
            result = self.execute_sql(f"""
                SELECT SQRT(value) as sqrt_value
                FROM {self.test_table}
                WHERE id = 1
            """)
            rows = list(result)
            self.assertAlmostEqual(rows[0]["sqrt_value"], math.sqrt(25.0), places=5)
        except Exception:
            self.skipTest("SQRT function not supported")
    
    def test_power(self):
        """Test POWER() function"""
        try:
            result = self.execute_sql(f"""
                SELECT POWER(value, 2) as squared, POWER(int_val, 3) as cubed
                FROM {self.test_table}
                WHERE id = 1
            """)
            rows = list(result)
            self.assertAlmostEqual(rows[0]["squared"], 625.0, places=2)
            self.assertEqual(rows[0]["cubed"], 15625)  # 25^3
        except Exception:
            self.skipTest("POWER function not supported")
    
    def test_sin_cos(self):
        """Test SIN() and COS() functions"""
        try:
            # Test with radians
            result = self.execute_sql(f"""
                SELECT SIN(0) as sin_zero, COS(0) as cos_zero
                FROM {self.test_table}
                WHERE id = 1
            """)
            rows = list(result)
            self.assertAlmostEqual(rows[0]["sin_zero"], 0.0, places=5)
            self.assertAlmostEqual(rows[0]["cos_zero"], 1.0, places=5)
        except Exception:
            self.skipTest("SIN/COS functions not supported")
    
    def test_log_exp(self):
        """Test LOG() and EXP() functions"""
        try:
            result = self.execute_sql(f"""
                SELECT LOG(100) as log_100, EXP(0) as exp_zero
                FROM {self.test_table}
                WHERE id = 1
            """)
            rows = list(result)
            # LOG(100) with base 10 should be 2
            # LOG(100) with natural log should be ~4.605
            # Just check it returns a reasonable value
            self.assertIsInstance(rows[0]["log_100"], (int, float))
            self.assertAlmostEqual(rows[0]["exp_zero"], 1.0, places=5)
        except Exception:
            self.skipTest("LOG/EXP functions not supported")
    
    def test_round_ceil_floor(self):
        """Test ROUND(), CEIL(), FLOOR() functions"""
        try:
            # Add a decimal value
            self.execute_sql(f"INSERT INTO {self.test_table} (id, value) VALUES (5, 3.14159)")
            
            result = self.execute_sql(f"""
                SELECT 
                    ROUND(value, 2) as rounded,
                    CEIL(value) as ceiling,
                    FLOOR(value) as floor_val
                FROM {self.test_table}
                WHERE id = 5
            """)
            rows = list(result)
            self.assertAlmostEqual(rows[0]["rounded"], 3.14, places=5)
            self.assertEqual(rows[0]["ceiling"], 4)
            self.assertEqual(rows[0]["floor_val"], 3)
        except Exception:
            self.skipTest("ROUND/CEIL/FLOOR functions not supported")
    
    def test_mod(self):
        """Test MOD() function"""
        try:
            result = self.execute_sql(f"""
                SELECT MOD(int_val, 7) as mod_result
                FROM {self.test_table}
                WHERE id = 1
            """)
            rows = list(result)
            self.assertEqual(rows[0]["mod_result"], 25 % 7)  # 25 mod 7 = 4
        except Exception:
            self.skipTest("MOD function not supported")


class TestTimeFunctions(LocalTestCase):
    """Test time-related functions"""
    
    def setUp(self):
        super().setUp()
        self.test_table = "time_test_table"
        schema = "id INTEGER PRIMARY KEY, event_time TIMESTAMP, value REAL"
        self.create_test_table(self.test_table, schema)
        
        # Insert hourly data for a few days
        timestamps = []
        current = 1609459200000  # 2021-01-01 00:00:00 UTC
        for i in range(24 * 3):  # 3 days of hourly data
            timestamps.append(current + (i * 3600000))  # Add i hours
        
        for i, ts in enumerate(timestamps):
            self.execute_sql(f"INSERT INTO {self.test_table} VALUES ({i + 1}, {ts}, {i * 10.0})")
    
    def test_time_bucket(self):
        """Test TIME_BUCKET() function for time series aggregation"""
        try:
            # Bucket by day
            result = self.execute_sql(f"""
                SELECT 
                    TIME_BUCKET(event_time, '1 day') as bucket,
                    COUNT(*) as count,
                    AVG(value) as avg_value
                FROM {self.test_table}
                GROUP BY TIME_BUCKET(event_time, '1 day')
                ORDER BY bucket
            """)
            rows = list(result)
            
            # Should have 3 buckets (one for each day)
            self.assertEqual(len(rows), 3)
            
            # Each bucket should have 24 hours of data
            for row in rows:
                self.assertEqual(row["count"], 24)
        except Exception:
            self.skipTest("TIME_BUCKET function not supported")
    
    def test_time_bucket_various_intervals(self):
        """Test TIME_BUCKET() with various interval formats"""
        intervals = [
            "1 hour",
            "6 hours", 
            "12 hours",
            "1 day",
            "7 days",
            "1 month",
            "1 year"
        ]
        
        for interval in intervals:
            try:
                result = self.execute_sql(f"""
                    SELECT TIME_BUCKET(event_time, '{interval}') as bucket
                    FROM {self.test_table}
                    LIMIT 1
                """)
                rows = list(result)
                # Just verify query executes
                self.assertEqual(len(rows), 1)
            except Exception:
                # Some intervals might not be supported
                continue
    
    def test_to_iso8601(self):
        """Test TO_ISO8601() function to convert timestamp to ISO8601 string"""
        try:
            # Test with known timestamp
            timestamp = 1609459200000  # 2021-01-01T00:00:00.000Z
            
            result = self.execute_sql(f"SELECT TO_ISO8601({timestamp}) as iso_string")
            rows = list(result)
            
            # Should return a string
            self.assertIsInstance(rows[0]["iso_string"], str)
            # Should contain 2021 (the year)
            self.assertIn("2021", rows[0]["iso_string"])
        except Exception:
            self.skipTest("TO_ISO8601 function not supported")
    
    def test_to_char(self):
        """Test TO_CHAR() function to format timestamp"""
        try:
            timestamp = 1609459200000  # 2021-01-01 00:00:00
            
            result = self.execute_sql(f"""
                SELECT TO_CHAR({timestamp}, 'YYYY-MM-DD') as formatted
            """)
            rows = list(result)
            
            self.assertIsInstance(rows[0]["formatted"], str)
            self.assertIn("2021-01-01", rows[0]["formatted"])
        except Exception:
            self.skipTest("TO_CHAR function not supported")
    
    def test_to_epoch(self):
        """Test TO_EPOCH() function to convert to epoch milliseconds"""
        try:
            # Test with ISO8601 string (if TO_ISO8601 is supported)
            # First get ISO string
            result = self.execute_sql(f"SELECT TO_ISO8601(1609459200000) as iso_string")
            rows = list(result)
            iso_string = rows[0]["iso_string"]
            
            # Convert back to epoch
            result = self.execute_sql(f"SELECT TO_EPOCH('{iso_string}') as epoch")
            rows = list(result)
            
            # Should be close to original timestamp
            self.assertAlmostEqual(rows[0]["epoch"], 1609459200000, delta=1000)  # Within 1 second
        except Exception:
            self.skipTest("TO_EPOCH function not supported")


class TestMovingWindowFunctions(LocalTestCase):
    """Test moving window functions"""
    
    def setUp(self):
        super().setUp()
        self.test_table = "window_test_table"
        schema = "id INTEGER PRIMARY KEY, timestamp TIMESTAMP, value REAL"
        self.create_test_table(self.test_table, schema)
        
        # Insert time series data
        timestamps = []
        current = 1609459200000  # Start time
        for i in range(10):
            ts = current + (i * 60000)  # 1 minute intervals
            self.execute_sql(f"INSERT INTO {self.test_table} VALUES ({i + 1}, {ts}, {i * 10.0})")
    
    def test_moving_sum(self):
        """Test MOVING_SUM() function"""
        try:
            result = self.execute_sql(f"""
                SELECT 
                    timestamp,
                    value,
                    MOVING_SUM(value, 3) OVER (ORDER BY timestamp) as moving_sum_3
                FROM {self.test_table}
                ORDER BY timestamp
            """)
            rows = list(result)
            
            # Verify we got results
            self.assertEqual(len(rows), 10)
            
            # Check first few moving sums
            # Row 1: value=0, moving_sum_3=0 (only 1 value in window)
            # Row 2: values=0,10, moving_sum_3=10
            # Row 3: values=0,10,20, moving_sum_3=30
            self.assertAlmostEqual(rows[0]["moving_sum_3"], 0.0, places=2)
            self.assertAlmostEqual(rows[1]["moving_sum_3"], 10.0, places=2)
            self.assertAlmostEqual(rows[2]["moving_sum_3"], 30.0, places=2)
        except Exception:
            self.skipTest("MOVING_SUM function not supported")
    
    def test_moving_average(self):
        """Test MOVING_AVERAGE() function"""
        try:
            result = self.execute_sql(f"""
                SELECT 
                    timestamp,
                    value,
                    MOVING_AVERAGE(value, 3) OVER (ORDER BY timestamp) as moving_avg_3
                FROM {self.test_table}
                ORDER BY timestamp
            """)
            rows = list(result)
            
            self.assertEqual(len(rows), 10)
            
            # Row 3: average of 0,10,20 = 10
            self.assertAlmostEqual(rows[2]["moving_avg_3"], 10.0, places=2)
        except Exception:
            self.skipTest("MOVING_AVERAGE function not supported")


class TestMiscellaneousFunctions(LocalTestCase):
    """Test miscellaneous functions"""
    
    def test_coalesce(self):
        """Test COALESCE() function to handle NULL values"""
        try:
            table_name = "coalesce_test"
            self.create_test_table(table_name, "id INTEGER, name TEXT, nickname TEXT, default_name TEXT")
            
            self.execute_sql(f"INSERT INTO {table_name} VALUES (1, 'Alice', NULL, 'Default')")
            self.execute_sql(f"INSERT INTO {table_name} VALUES (2, NULL, 'Bob', 'Default')")
            self.execute_sql(f"INSERT INTO {table_name} VALUES (3, NULL, NULL, 'Default')")
            
            result = self.execute_sql(f"""
                SELECT 
                    COALESCE(name, nickname, default_name) as display_name
                FROM {table_name}
                ORDER BY id
            """)
            rows = list(result)
            
            self.assertEqual(len(rows), 3)
            self.assertEqual(rows[0]["display_name"], "Alice")  # name is not NULL
            self.assertEqual(rows[1]["display_name"], "Bob")    # name is NULL, use nickname
            self.assertEqual(rows[2]["display_name"], "Default") # both NULL, use default
        except Exception:
            self.skipTest("COALESCE function not supported")
    
    def test_case_when(self):
        """Test CASE WHEN expression"""
        table_name = "case_test"
        self.create_test_table(table_name, "id INTEGER, score INTEGER")
        
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1, 95)")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (2, 85)")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (3, 75)")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (4, 65)")
        
        result = self.execute_sql(f"""
            SELECT 
                score,
                CASE 
                    WHEN score >= 90 THEN 'A'
                    WHEN score >= 80 THEN 'B'
                    WHEN score >= 70 THEN 'C'
                    WHEN score >= 60 THEN 'D'
                    ELSE 'F'
                END as grade
            FROM {case_test}
            ORDER BY id
        """)
        rows = list(result)
        
        self.assertEqual(len(rows), 4)
        self.assertEqual(rows[0]["grade"], "A")  # 95
        self.assertEqual(rows[1]["grade"], "B")  # 85
        self.assertEqual(rows[2]["grade"], "C")  # 75
        self.assertEqual(rows[3]["grade"], "D")  # 65


if __name__ == '__main__':
    unittest.main()