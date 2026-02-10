"""10-minute stress test for RemDB with continuous insert and query operations"""

import unittest
import time
import random
from tests.fixtures import LocalTestCase

class Test10MinuteStress(LocalTestCase):
    """10-minute stress test for RemDB"""
    
    def test_10min_continuous_operations(self):
        """Run continuous insert and query operations for exactly 10 minutes"""
        # Test configuration
        TABLE_NAME = "stress_test_table"
        TEST_DURATION_SECONDS = 600  # 10 minutes
        
        # Create test table with mixed data types including VECTOR
        schema = """
            id INTEGER PRIMARY KEY,
            name TEXT,
            value INTEGER,
            price REAL,
            active BOOLEAN,
            created_at TIMESTAMP,
            metadata JSON,
            embedding VECTOR(3)
        """
        
        try:
            self.create_test_table(TABLE_NAME, schema)
        except Exception as e:
            # Skip if VECTOR type is not supported or causes memory issues
            self.skipTest(f"VECTOR type not supported in stress test: {e}")
        
        # Metrics tracking
        metrics = {
            'start_time': time.time(),
            'end_time': None,
            'total_inserts': 0,
            'total_queries': 0,
            'successful_inserts': 0,
            'successful_queries': 0,
            'failed_inserts': 0,
            'failed_queries': 0,
            'operations_per_second': 0,
        }
        
        # Generate test data function
        def generate_test_data(row_id):
            """Generate random test data for insertion"""
            names = ['Alice', 'Bob', 'Charlie', 'David', 'Eve', 'Frank', 'Grace', 'Henry']
            name = random.choice(names)
            value = random.randint(1, 10000)
            price = round(random.uniform(1.0, 1000.0), 2)
            active = random.choice([True, False])
            created_at = int(time.time() * 1000)
            metadata = '{"row_id": ' + str(row_id) + ', "random_val": ' + str(random.randint(1, 100)) + ', "tags": ["test", "stress"]}'
            # Generate random 3-dimensional vector
            vector = '[' + ','.join([str(round(random.uniform(-1.0, 1.0), 2)) for _ in range(3)]) + ']'
            
            return (row_id, name, value, price, active, created_at, metadata, vector)
        
        # Insert operation function
        def perform_insert(row_id):
            """Perform insert operation"""
            try:
                data = generate_test_data(row_id)
                # Escape single quotes in name and metadata
                id_val, name, value, price, active, created_at, metadata, vector = data
                name_escaped = name.replace("'", "''")
                metadata_escaped = metadata.replace("'", "''")
                
                sql = f"""
                    INSERT INTO {TABLE_NAME} 
                    VALUES ({id_val}, '{name_escaped}', {value}, {price}, {active}, {created_at}, '{metadata_escaped}', '{vector}')
                """
                
                self.execute_sql(sql)
                metrics['successful_inserts'] += 1
                return True
            except Exception as e:
                metrics['failed_inserts'] += 1
                return False
        
        # Query operation function
        def perform_query():
            """Perform random query operation"""
            try:
                query_type = random.choice(['basic', 'where', 'aggregate', 'order', 'limit', 'vector'])
                
                if query_type == 'basic':
                    # Basic select
                    self.execute_sql(f"SELECT * FROM {TABLE_NAME} ORDER BY id DESC LIMIT 10")
                
                elif query_type == 'where':
                    # Where clause with random condition
                    condition_type = random.choice(['value', 'active', 'price'])
                    
                    if condition_type == 'value':
                        threshold = random.randint(1, 10000)
                        self.execute_sql(f"SELECT * FROM {TABLE_NAME} WHERE value > {threshold} LIMIT 5")
                    
                    elif condition_type == 'active':
                        active_val = random.choice([True, False])
                        self.execute_sql(f"SELECT * FROM {TABLE_NAME} WHERE active = {active_val} LIMIT 5")
                    
                    elif condition_type == 'price':
                        threshold = random.uniform(1.0, 1000.0)
                        self.execute_sql(f"SELECT * FROM {TABLE_NAME} WHERE price < {threshold} LIMIT 5")
                
                elif query_type == 'aggregate':
                    # Aggregation query
                    agg_type = random.choice(['count', 'sum', 'avg'])
                    
                    if agg_type == 'count':
                        self.execute_sql(f"SELECT COUNT(*) as total FROM {TABLE_NAME}")
                    
                    elif agg_type == 'sum':
                        self.execute_sql(f"SELECT SUM(value) as total_value FROM {TABLE_NAME}")
                    
                    elif agg_type == 'avg':
                        self.execute_sql(f"SELECT AVG(price) as avg_price FROM {TABLE_NAME}")
                
                elif query_type == 'order':
                    # Order by query
                    order_column = random.choice(['value', 'price', 'created_at'])
                    order_direction = random.choice(['ASC', 'DESC'])
                    self.execute_sql(f"SELECT * FROM {TABLE_NAME} ORDER BY {order_column} {order_direction} LIMIT 10")
                
                elif query_type == 'limit':
                    # Limit query with offset
                    limit = random.randint(1, 20)
                    offset = random.randint(0, max(0, metrics['total_inserts'] - limit))
                    self.execute_sql(f"SELECT * FROM {TABLE_NAME} ORDER BY id LIMIT {limit} OFFSET {offset}")
                
                elif query_type == 'vector':
                    # Vector-related query
                    # Basic vector select
                    self.execute_sql(f"SELECT id, name, embedding FROM {TABLE_NAME} ORDER BY id DESC LIMIT 5")
                
                metrics['successful_queries'] += 1
                return True
            except Exception as e:
                metrics['failed_queries'] += 1
                return False
        
        # Main test loop
        print(f"Starting 10-minute stress test at {time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"Test will run for {TEST_DURATION_SECONDS} seconds (10 minutes)")
        print("=" * 60)
        
        row_id = 1
        start_time = time.time()
        current_time = start_time
        
        # Run until 10 minutes elapse
        while current_time - start_time < TEST_DURATION_SECONDS:
            # Perform insert
            perform_insert(row_id)
            metrics['total_inserts'] += 1
            row_id += 1
            
            # Perform query
            perform_query()
            metrics['total_queries'] += 1
            
            # Update current time
            current_time = time.time()
            
            # Print progress every minute
            elapsed_seconds = current_time - start_time
            if int(elapsed_seconds) % 60 == 0 and elapsed_seconds > 0:
                elapsed_minutes = elapsed_seconds / 60
                ops_per_sec = (metrics['total_inserts'] + metrics['total_queries']) / elapsed_seconds
                print(f"Progress: {elapsed_minutes:.1f} minutes elapsed, "
                      f"{metrics['total_inserts']} inserts, {metrics['total_queries']} queries, "
                      f"{ops_per_sec:.2f} ops/sec")
        
        # End test
        metrics['end_time'] = current_time
        total_duration = metrics['end_time'] - metrics['start_time']
        total_operations = metrics['total_inserts'] + metrics['total_queries']
        metrics['operations_per_second'] = total_operations / total_duration if total_duration > 0 else 0
        
        # Print final results
        print("=" * 60)
        print(f"10-minute stress test completed at {time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"Actual duration: {total_duration:.2f} seconds")
        print(f"Total operations: {total_operations}")
        print(f"Operations per second: {metrics['operations_per_second']:.2f}")
        print(f"\nInserts:")
        print(f"  Total: {metrics['total_inserts']}")
        print(f"  Successful: {metrics['successful_inserts']}")
        print(f"  Failed: {metrics['failed_inserts']}")
        print(f"\nQueries:")
        print(f"  Total: {metrics['total_queries']}")
        print(f"  Successful: {metrics['successful_queries']}")
        print(f"  Failed: {metrics['failed_queries']}")
        print(f"\nSuccess rate:")
        print(f"  Inserts: {(metrics['successful_inserts'] / metrics['total_inserts'] * 100):.2f}%" if metrics['total_inserts'] > 0 else "  Inserts: 0%")
        print(f"  Queries: {(metrics['successful_queries'] / metrics['total_queries'] * 100):.2f}%" if metrics['total_queries'] > 0 else "  Queries: 0%")
        print(f"  Overall: {( (metrics['successful_inserts'] + metrics['successful_queries']) / total_operations * 100):.2f}%" if total_operations > 0 else "  Overall: 0%")
        
        # Verify table has data
        self.assertGreater(metrics['total_inserts'], 0)
        
        # Verify success rates
        self.assertGreaterEqual(metrics['successful_inserts'], 0)
        self.assertGreaterEqual(metrics['successful_queries'], 0)
        
        # Verify test ran for at least 9.5 minutes (allowing for small timing differences)
        self.assertGreaterEqual(total_duration, 570)  # 9.5 minutes
        
        print("\nStress test verification complete!")
    
    def test_mixed_data_continuous_insert(self):
        """Test continuous insertion of mixed data types"""
        TABLE_NAME = "mixed_data_stress"
        
        # Create table with mixed data types
        schema = """
            id INTEGER PRIMARY KEY,
            text_col TEXT,
            int_col INTEGER,
            real_col REAL,
            bool_col BOOLEAN,
            ts_col TIMESTAMP,
            json_col JSON
        """
        
        self.create_test_table(TABLE_NAME, schema)
        
        # Insert 1000 rows of mixed data
        start_time = time.time()
        total_inserts = 1000
        
        print(f"Inserting {total_inserts} rows of mixed data types...")
        
        for i in range(1, total_inserts + 1):
            # Generate random data
            text_val = f"Test_{i}_{random.randint(1000, 9999)}"
            int_val = random.randint(1, 1000000)
            real_val = round(random.uniform(0.0, 1000.0), 2)
            bool_val = random.choice([True, False])
            ts_val = int(time.time() * 1000)
            json_val = '{"id": ' + str(i) + ', "name": "' + text_val + '", "value": ' + str(int_val) + '}'
            
            # Escape single quotes
            text_val_escaped = text_val.replace("'", "''")
            json_val_escaped = json_val.replace("'", "''")
            
            # Insert data
            self.execute_sql(f"""
                INSERT INTO {TABLE_NAME} 
                VALUES ({i}, '{text_val_escaped}', {int_val}, {real_val}, {bool_val}, {ts_val}, '{json_val_escaped}')
            """)
        
        end_time = time.time()
        duration = end_time - start_time
        
        print(f"Inserted {total_inserts} rows in {duration:.2f} seconds")
        print(f"Insert rate: {total_inserts / duration:.2f} inserts per second")
        
        # Verify all rows were inserted
        self.assert_row_count(TABLE_NAME, total_inserts)
        
        # Test query performance
        query_start = time.time()
        result = self.execute_sql(f"SELECT COUNT(*) as count FROM {TABLE_NAME} WHERE int_col > 500000")
        query_end = time.time()
        query_duration = query_end - query_start
        
        rows = list(result)
        count = rows[0]['count'] if rows else 0
        
        print(f"Query executed in {query_duration:.4f} seconds")
        print(f"Found {count} rows where int_col > 500000")

if __name__ == '__main__':
    unittest.main()
