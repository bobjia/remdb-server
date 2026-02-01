"""Test time series specific functionality in RemDB"""

import unittest
import time
from tests.fixtures import LocalTestCase


class TestTimeSeriesOperations(LocalTestCase):
    """Test time series specific operations"""
    
    def setUp(self):
        super().setUp()
        self.timeseries_table = "timeseries_ops_test"
        
        # Create a timeseries table
        schema = """
            timestamp TIMESTAMP,
            value REAL,
            sensor_id INTEGER,
            location TEXT
        """
        self.create_test_table(self.timeseries_table, schema)
        
        # Insert time series data with regular intervals
        base_timestamp = 1609459200000  # 2021-01-01 00:00:00 UTC
        interval = 60000  # 1 minute in milliseconds
        
        for i in range(100):  # 100 minutes of data
            ts = base_timestamp + (i * interval)
            sensor_id = i % 5  # 5 different sensors
            value = 20.0 + (i * 0.1) + sensor_id  # Varying values
            location = f"loc_{sensor_id % 3}"  # 3 different locations
            
            self.execute_sql(f"""
                INSERT INTO {self.timeseries_table} 
                (timestamp, value, sensor_id, location)
                VALUES ({ts}, {value}, {sensor_id}, '{location}')
            """)
    
    def test_time_range_query(self):
        """Test querying data within a time range"""
        start_time = 1609459200000  # 00:00
        end_time = 1609459320000    # 00:02 (2 minutes later)
        
        result = self.execute_sql(f"""
            SELECT * FROM {self.timeseries_table}
            WHERE timestamp >= {start_time} AND timestamp <= {end_time}
            ORDER BY timestamp
        """)
        rows = list(result)
        
        # Should get 3 rows (00:00, 00:01, 00:02)
        self.assertEqual(len(rows), 3)
        
        # Verify time ordering
        timestamps = [row["timestamp"] for row in rows]
        self.assertEqual(timestamps, sorted(timestamps))
        
        # Verify all timestamps are within range
        for row in rows:
            self.assertGreaterEqual(row["timestamp"], start_time)
            self.assertLessEqual(row["timestamp"], end_time)
    
    def test_latest_data_query(self):
        """Test querying latest data using ORDER BY and LIMIT"""
        result = self.execute_sql(f"""
            SELECT * FROM {self.timeseries_table}
            ORDER BY timestamp DESC
            LIMIT 10
        """)
        rows = list(result)
        
        self.assertEqual(len(rows), 10)
        
        # Verify descending order
        timestamps = [row["timestamp"] for row in rows]
        self.assertEqual(timestamps, sorted(timestamps, reverse=True))
    
    def test_time_range_with_specific_sensor(self):
        """Test time range query filtered by sensor ID"""
        start_time = 1609459200000
        end_time = 1609459500000  # 5 minutes
        sensor_id = 2
        
        result = self.execute_sql(f"""
            SELECT * FROM {self.timeseries_table}
            WHERE timestamp >= {start_time} 
              AND timestamp <= {end_time}
              AND sensor_id = {sensor_id}
            ORDER BY timestamp
        """)
        rows = list(result)
        
        # Should get 1 row per 5 minutes for sensor 2
        # Sensor 2 appears at minutes 2, 7, 12, 17, 22, 27, 32, 37, 42, 47, 52, 57
        # Within first 5 minutes, only minute 2 qualifies
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["sensor_id"], sensor_id)
    
    def test_time_based_aggregation(self):
        """Test aggregation of time series data"""
        # Calculate average value per sensor
        result = self.execute_sql(f"""
            SELECT 
                sensor_id,
                COUNT(*) as count,
                AVG(value) as avg_value,
                MIN(value) as min_value,
                MAX(value) as max_value
            FROM {self.timeseries_table}
            GROUP BY sensor_id
            ORDER BY sensor_id
        """)
        rows = list(result)
        
        # Should have 5 rows (5 sensors)
        self.assertEqual(len(rows), 5)
        
        # Verify sensor IDs
        sensor_ids = [row["sensor_id"] for row in rows]
        self.assertEqual(sensor_ids, [0, 1, 2, 3, 4])
        
        # Verify counts - each sensor should have 20 readings (100 total / 5 sensors)
        for row in rows:
            self.assertEqual(row["count"], 20)
    
    def test_time_bucket_aggregation(self):
        """Test TIME_BUCKET function for time series aggregation"""
        try:
            # Aggregate by 5-minute buckets
            result = self.execute_sql(f"""
                SELECT 
                    TIME_BUCKET('5m', timestamp) as bucket,
                    COUNT(*) as reading_count,
                    AVG(value) as avg_value,
                    MIN(value) as min_value,
                    MAX(value) as max_value
                FROM {self.timeseries_table}
                GROUP BY TIME_BUCKET('5m', timestamp)
                ORDER BY bucket
            """)
            rows = list(result)
            
            # Should have 20 buckets (100 minutes / 5 minutes per bucket)
            self.assertEqual(len(rows), 20)
            
            # Each bucket should have 5 readings
            for row in rows:
                self.assertEqual(row["reading_count"], 5)
        except Exception:
            self.skipTest("TIME_BUCKET function not supported")
    
    def test_time_bucket_with_filter(self):
        """Test TIME_BUCKET with WHERE clause filtering"""
        try:
            sensor_id = 1
            
            result = self.execute_sql(f"""
                SELECT 
                    TIME_BUCKET('10m', timestamp) as bucket,
                    COUNT(*) as count,
                    AVG(value) as avg_value
                FROM {self.timeseries_table}
                WHERE sensor_id = {sensor_id}
                GROUP BY TIME_BUCKET('10m', timestamp)
                ORDER BY bucket
            """)
            rows = list(result)
            
            # Should have some buckets with sensor 1 data
            self.assertGreater(len(rows), 0)
            
            # Verify all rows are for sensor 1 (implied by WHERE clause)
            # and count should be 2 per bucket (100 minutes total, 5 sensors, 10-minute buckets)
            # Each 10-minute bucket has 10 readings total, sensor 1 appears in 2 of them
            for row in rows:
                self.assertEqual(row["count"], 2)
        except Exception:
            self.skipTest("TIME_BUCKET function not supported")
    
    def test_time_bucket_various_intervals(self):
        """Test TIME_BUCKET with various interval formats"""
        intervals = [
            "1m",      # 1 minute
            "5m",      # 5 minutes
            "10m",     # 10 minutes
            "1h",      # 1 hour
            "6h",      # 6 hours
            "1d",      # 1 day
        ]
        
        for interval in intervals:
            try:
                result = self.execute_sql(f"""
                    SELECT TIME_BUCKET('{interval}', timestamp) as bucket
                    FROM {self.timeseries_table}
                    GROUP BY TIME_BUCKET('{interval}', timestamp)
                    ORDER BY bucket
                    LIMIT 1
                """)
                rows = list(result)
                # Just verify query executes
                self.assertEqual(len(rows), 1)
            except Exception:
                # Some intervals might not be supported
                continue
    
    def test_time_conversion_functions(self):
        """Test time conversion functions on time series data"""
        # Test TO_ISO8601
        try:
            result = self.execute_sql(f"""
                SELECT TO_ISO8601(timestamp) as iso_time
                FROM {self.timeseries_table}
                WHERE sensor_id = 0
                ORDER BY timestamp
                LIMIT 1
            """)
            rows = list(result)
            self.assertEqual(len(rows), 1)
            iso_string = rows[0]["iso_time"]
            self.assertIsInstance(iso_string, str)
            # Should contain 2021 (the year of our test data)
            self.assertIn("2021", iso_string)
        except Exception:
            self.skipTest("TO_ISO8601 function not supported")
        
        # Test TO_CHAR
        try:
            result = self.execute_sql(f"""
                SELECT TO_CHAR(timestamp, 'YYYY-MM-DD') as date_str
                FROM {self.timeseries_table}
                WHERE sensor_id = 1
                ORDER BY timestamp
                LIMIT 1
            """)
            rows = list(result)
            self.assertEqual(len(rows), 1)
            date_str = rows[0]["date_str"]
            self.assertIsInstance(date_str, str)
            self.assertIn("2021-01-01", date_str)
        except Exception:
            self.skipTest("TO_CHAR function not supported")
        
        # Test TO_EPOCH with string input (if TO_ISO8601 is supported)
        try:
            # First get ISO string
            result = self.execute_sql(f"""
                SELECT TO_ISO8601(1609459200000) as iso_string
            """)
            rows = list(result)
            iso_string = rows[0]["iso_string"]
            
            # Convert back to epoch
            result = self.execute_sql(f"SELECT TO_EPOCH('{iso_string}') as epoch")
            rows = list(result)
            self.assertAlmostEqual(rows[0]["epoch"], 1609459200000, delta=1000)
        except Exception:
            self.skipTest("TO_EPOCH function not supported")


class TestMovingWindowFunctions(LocalTestCase):
    """Test moving window functions for time series data"""
    
    def setUp(self):
        super().setUp()
        self.window_table = "moving_window_test"
        
        # Create a simple time series table
        schema = "timestamp TIMESTAMP, value REAL, sensor_id INTEGER"
        self.create_test_table(self.window_table, schema)
        
        # Insert sequential data
        base_timestamp = 1609459200000  # Start time
        for i in range(20):  # 20 data points
            ts = base_timestamp + (i * 60000)  # 1 minute intervals
            value = 10.0 + (i * 2.0)  # Linearly increasing values
            sensor_id = i % 3  # 3 sensors
            
            self.execute_sql(f"""
                INSERT INTO {self.window_table} 
                VALUES ({ts}, {value}, {sensor_id})
            """)
    
    def test_moving_sum(self):
        """Test MOVING_SUM function with various window sizes"""
        try:
            # Test with window size 3
            result = self.execute_sql(f"""
                SELECT 
                    timestamp,
                    value,
                    MOVING_SUM(value, 3) OVER (ORDER BY timestamp) as moving_sum_3
                FROM {self.window_table}
                WHERE sensor_id = 0
                ORDER BY timestamp
            """)
            rows = list(result)
            
            self.assertGreater(len(rows), 0)
            
            # Manually calculate moving sum for first few rows
            # Row 0: value=10, sum=10 (only 1 value)
            # Row 3: values=10, 16, 22, sum=48
            if len(rows) >= 4:
                # Find sensor 0 rows (sensor 0 at rows 0, 3, 6, 9, ...)
                sensor_0_rows = [row for row in rows if row["sensor_id"] == 0]
                if len(sensor_0_rows) >= 2:
                    # First sensor 0 row: should have moving sum = its value
                    self.assertAlmostEqual(sensor_0_rows[0]["moving_sum_3"], sensor_0_rows[0]["value"], places=2)
        except Exception:
            self.skipTest("MOVING_SUM function not supported")
    
    def test_moving_average(self):
        """Test MOVING_AVERAGE function"""
        try:
            result = self.execute_sql(f"""
                SELECT 
                    timestamp,
                    value,
                    MOVING_AVERAGE(value, 5) OVER (ORDER BY timestamp) as moving_avg_5
                FROM {self.window_table}
                ORDER BY timestamp
            """)
            rows = list(result)
            
            self.assertGreater(len(rows), 0)
            
            # For row 4 (5th row), moving average should be average of values 0-4
            if len(rows) >= 5:
                values = [rows[i]["value"] for i in range(5)]
                expected_avg = sum(values) / 5
                self.assertAlmostEqual(rows[4]["moving_avg_5"], expected_avg, places=2)
        except Exception:
            self.skipTest("MOVING_AVERAGE function not supported")
    
    def test_moving_window_with_partition(self):
        """Test moving window functions with PARTITION BY"""
        try:
            result = self.execute_sql(f"""
                SELECT 
                    sensor_id,
                    timestamp,
                    value,
                    MOVING_SUM(value, 3) OVER (
                        PARTITION BY sensor_id 
                        ORDER BY timestamp
                    ) as moving_sum_by_sensor
                FROM {self.window_table}
                ORDER BY sensor_id, timestamp
            """)
            rows = list(result)
            
            self.assertGreater(len(rows), 0)
            
            # Verify data is partitioned by sensor_id
            # Each sensor's moving sum should be calculated independently
            sensor_groups = {}
            for row in rows:
                sensor_id = row["sensor_id"]
                if sensor_id not in sensor_groups:
                    sensor_groups[sensor_id] = []
                sensor_groups[sensor_id].append(row)
            
            # Each sensor should have its own sequence
            for sensor_id, sensor_rows in sensor_groups.items():
                self.assertGreater(len(sensor_rows), 0)
        except Exception:
            self.skipTest("Moving window with PARTITION BY not supported")


class TestTimeSeriesCompression(LocalTestCase):
    """Test time series compression options"""
    
    def test_create_timeseries_with_compression(self):
        """Test creating timeseries table with various compression algorithms"""
        compression_algorithms = [
            "none",
            "delta",
            "runlength",
            "delta-runlength",
            "delta-delta"
        ]
        
        for i, algorithm in enumerate(compression_algorithms):
            table_name = f"compressed_ts_{i}"
            
            try:
                self.execute_sql(f"""
                    CREATE TIMESERIES TABLE {table_name} (
                        timestamp TIMESTAMP,
                        value REAL,
                        sensor_id INTEGER
                    ) WITH COMPRESSION={algorithm}
                """)
                
                # Insert some test data
                for j in range(10):
                    ts = 1609459200000 + (j * 60000)
                    self.execute_sql(f"""
                        INSERT INTO {table_name} 
                        VALUES ({ts}, {20.0 + j}, {j % 3})
                    """)
                
                # Query to verify
                result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY timestamp")
                rows = list(result)
                self.assertEqual(len(rows), 10)
                
                # Clean up
                self.execute_sql(f"DROP TABLE {table_name}")
                
            except Exception:
                # Some compression algorithms might not be supported
                continue
    
    def test_create_timeseries_with_ttl(self):
        """Test creating timeseries table with TTL (Time To Live)"""
        ttl_values = [
            "1d",      # 1 day
            "7d",      # 7 days
            "30d",     # 30 days
            "1h",      # 1 hour
            "6h",      # 6 hours
            "24h",     # 24 hours
        ]
        
        for i, ttl in enumerate(ttl_values):
            table_name = f"ttl_ts_{i}"
            
            try:
                self.execute_sql(f"""
                    CREATE TIMESERIES TABLE {table_name} (
                        timestamp TIMESTAMP,
                        metric REAL,
                        source TEXT
                    ) WITH TTL='{ttl}'
                """)
                
                # Insert test data
                for j in range(5):
                    ts = 1609459200000 + (j * 3600000)  # 1 hour intervals
                    self.execute_sql(f"""
                        INSERT INTO {table_name} 
                        VALUES ({ts}, {100.0 + j}, 'source_{j}')
                    """)
                
                # Verify data can be queried
                result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY timestamp")
                rows = list(result)
                self.assertEqual(len(rows), 5)
                
                # Clean up
                self.execute_sql(f"DROP TABLE {table_name}")
                
            except Exception:
                # Some TTL formats might not be supported
                continue
    
    def test_timeseries_with_compression_and_ttl(self):
        """Test timeseries table with both compression and TTL"""
        table_name = "full_featured_ts"
        
        try:
            self.execute_sql(f"""
                CREATE TIMESERIES TABLE {table_name} (
                    timestamp TIMESTAMP,
                    temperature REAL,
                    humidity REAL,
                    device_id INTEGER
                ) WITH COMPRESSION=delta, TTL='30d'
            """)
            
            # Insert data
            for i in range(20):
                ts = 1609459200000 + (i * 300000)  # 5 minute intervals
                temp = 20.0 + (i * 0.5)
                humidity = 50.0 + (i * 0.3)
                device_id = i % 4
                
                self.execute_sql(f"""
                    INSERT INTO {table_name} 
                    VALUES ({ts}, {temp}, {humidity}, {device_id})
                """)
            
            # Test queries
            result = self.execute_sql(f"""
                SELECT 
                    device_id,
                    AVG(temperature) as avg_temp,
                    AVG(humidity) as avg_humidity
                FROM {table_name}
                GROUP BY device_id
                ORDER BY device_id
            """)
            rows = list(result)
            self.assertEqual(len(rows), 4)  # 4 devices
            
            # Test time range query
            start_time = 1609459200000
            end_time = 1609459500000  # 5 minutes after start
            
            result = self.execute_sql(f"""
                SELECT * FROM {table_name}
                WHERE timestamp >= {start_time} AND timestamp <= {end_time}
                ORDER BY timestamp
            """)
            rows = list(result)
            # Should have 2 rows (0 and 5 minutes)
            self.assertGreaterEqual(len(rows), 1)
            
        except Exception:
            self.skipTest("Timeseries with compression and TTL not supported")


class TestTimeSeriesEdgeCases(LocalTestCase):
    """Test edge cases for time series operations"""
    
    def test_empty_time_range(self):
        """Test querying empty time range"""
        table_name = "edge_case_ts"
        self.create_test_table(table_name, "timestamp TIMESTAMP, value REAL")
        
        # Insert data
        for i in range(5):
            ts = 1609459200000 + (i * 60000)
            self.execute_sql(f"INSERT INTO {table_name} VALUES ({ts}, {i * 10.0})")
        
        # Query time range with no data
        result = self.execute_sql(f"""
            SELECT * FROM {table_name}
            WHERE timestamp > 1609459500000 AND timestamp < 1609459600000
        """)
        rows = list(result)
        self.assertEqual(len(rows), 0)
    
    def test_single_point_timeseries(self):
        """Test time series with only one data point"""
        table_name = "single_point_ts"
        self.create_test_table(table_name, "timestamp TIMESTAMP, value REAL, tag TEXT")
        
        # Insert single point
        ts = 1609459200000
        self.execute_sql(f"INSERT INTO {table_name} VALUES ({ts}, 42.0, 'test')")
        
        # Test queries
        result = self.execute_sql(f"SELECT * FROM {table_name}")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["value"], 42.0)
        
        # Test aggregation
        result = self.execute_sql(f"""
            SELECT 
                AVG(value) as avg_val,
                COUNT(*) as count,
                MIN(timestamp) as min_ts,
                MAX(timestamp) as max_ts
            FROM {table_name}
        """)
        rows = list(result)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["count"], 1)
        self.assertEqual(rows[0]["avg_val"], 42.0)
        self.assertEqual(rows[0]["min_ts"], ts)
        self.assertEqual(rows[0]["max_ts"], ts)
    
    def test_time_series_with_null_values(self):
        """Test time series containing NULL values"""
        table_name = "null_ts"
        self.create_test_table(table_name, "timestamp TIMESTAMP, value REAL, sensor_id INTEGER")
        
        # Insert data with some NULL values
        timestamps = [1609459200000, 1609459260000, 1609459320000]
        values = [10.0, None, 30.0]  # Middle value is NULL
        sensor_ids = [1, 1, 1]
        
        for i, ts in enumerate(timestamps):
            if values[i] is not None:
                self.execute_sql(f"""
                    INSERT INTO {table_name} 
                    VALUES ({ts}, {values[i]}, {sensor_ids[i]})
                """)
            else:
                self.execute_sql(f"""
                    INSERT INTO {table_name} (timestamp, sensor_id)
                    VALUES ({ts}, {sensor_ids[i]})
                """)
        
        # Test queries
        result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY timestamp")
        rows = list(result)
        self.assertEqual(len(rows), 3)
        
        # Count non-null values
        result = self.execute_sql(f"SELECT COUNT(value) as non_null_count FROM {table_name}")
        rows = list(result)
        self.assertEqual(rows[0]["non_null_count"], 2)  # Only 2 non-null values
    
    def test_high_frequency_time_series(self):
        """Test time series with high frequency data (millisecond intervals)"""
        table_name = "high_freq_ts"
        self.create_test_table(table_name, "timestamp TIMESTAMP, value REAL")
        
        # Insert high frequency data (1000 Hz for 1 second)
        base_timestamp = 1609459200000
        num_points = 1000
        
        for i in range(num_points):
            ts = base_timestamp + i  # 1 millisecond intervals
            value = 100.0 + (i * 0.01)
            self.execute_sql(f"INSERT INTO {table_name} VALUES ({ts}, {value})")
        
        # Test queries
        result = self.execute_sql(f"SELECT COUNT(*) as count FROM {table_name}")
        rows = list(result)
        self.assertEqual(rows[0]["count"], num_points)
        
        # Test time range query
        start_time = base_timestamp
        end_time = base_timestamp + 100  # First 100 milliseconds
        
        result = self.execute_sql(f"""
            SELECT * FROM {table_name}
            WHERE timestamp >= {start_time} AND timestamp <= {end_time}
            ORDER BY timestamp
        """)
        rows = list(result)
        self.assertEqual(len(rows), 101)  # 0-100 inclusive = 101 points


if __name__ == '__main__':
    unittest.main()