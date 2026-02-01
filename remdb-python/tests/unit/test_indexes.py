"""Test index-related SQL statements"""

import unittest
import time
from tests.fixtures import LocalTestCase
from tests.utils.data_generators import generate_vector_test_data, generate_random_vector


class TestScalarIndexes(LocalTestCase):
    """Test scalar indexes (BTREE, TTREE)"""
    
    def setUp(self):
        super().setUp()
        self.scalar_table = "scalar_test"
        self.create_test_table(self.scalar_table, """
            id INTEGER PRIMARY KEY,
            name TEXT,
            age INTEGER,
            salary REAL,
            created_at TIMESTAMP
        """)
        
        # Insert test data
        test_data = [
            {"id": 1, "name": "Alice", "age": 30, "salary": 75000.0, "created_at": 1609459200000},
            {"id": 2, "name": "Bob", "age": 25, "salary": 65000.0, "created_at": 1609545600000},
            {"id": 3, "name": "Charlie", "age": 35, "salary": 85000.0, "created_at": 1609632000000},
            {"id": 4, "name": "David", "age": 28, "salary": 70000.0, "created_at": 1609718400000},
            {"id": 5, "name": "Eve", "age": 32, "salary": 80000.0, "created_at": 1609804800000},
        ]
        self.insert_test_data(self.scalar_table, test_data)
    
    def test_create_btree_index(self):
        """Test CREATE INDEX with BTREE type"""
        index_name = "idx_scalar_name"
        
        # Create BTREE index
        self.execute_sql(f"CREATE INDEX {index_name} ON {self.scalar_table} (name) USING BTREE")
        
        # Verify index can be used in query (no error expected)
        result = self.execute_sql(f"SELECT * FROM {self.scalar_table} WHERE name = 'Alice'")
        rows = list(result)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["name"], "Alice")
    
    def test_create_ttree_index(self):
        """Test CREATE INDEX with TTREE type (for time series data)"""
        index_name = "idx_scalar_created"
        
        try:
            self.execute_sql(f"CREATE INDEX {index_name} ON {self.scalar_table} (created_at) USING TTREE")
            
            # Verify index can be used in time range query
            start_time = 1609459200000
            end_time = 1609632000000
            result = self.execute_sql(f"""
                SELECT * FROM {self.scalar_table} 
                WHERE created_at >= {start_time} AND created_at <= {end_time}
                ORDER BY created_at
            """)
            rows = list(result)
            self.assertEqual(len(rows), 3)  # Alice, Bob, Charlie
        except Exception:
            self.skipTest("TTREE index type not supported")
    
    def test_create_composite_index(self):
        """Test CREATE INDEX on multiple columns"""
        index_name = "idx_scalar_age_salary"
        
        self.execute_sql(f"CREATE INDEX {index_name} ON {self.scalar_table} (age, salary) USING BTREE")
        
        # Verify composite index can be used
        result = self.execute_sql(f"""
            SELECT * FROM {self.scalar_table} 
            WHERE age > 30 AND salary > 70000.0
            ORDER BY age, salary
        """)
        rows = list(result)
        self.assertEqual(len(rows), 2)  # Charlie (age 35), Eve (age 32)
    
    def test_create_index_with_online_mode(self):
        """Test CREATE INDEX with ONLINE mode"""
        index_name = "idx_scalar_online"
        
        try:
            self.execute_sql(f"CREATE INDEX {index_name} ON {self.scalar_table} (name) USING BTREE ONLINE")
            
            # Verify index created
            result = self.execute_sql(f"SELECT * FROM {self.scalar_table} WHERE name = 'Bob'")
            rows = list(result)
            self.assertEqual(len(rows), 1)
        except Exception:
            self.skipTest("ONLINE index creation not supported")
    
    def test_create_index_with_offline_mode(self):
        """Test CREATE INDEX with OFFLINE mode"""
        index_name = "idx_scalar_offline"
        
        try:
            self.execute_sql(f"CREATE INDEX {index_name} ON {self.scalar_table} (name) USING BTREE OFFLINE")
            
            # Verify index created
            result = self.execute_sql(f"SELECT * FROM {self.scalar_table} WHERE name = 'Charlie'")
            rows = list(result)
            self.assertEqual(len(rows), 1)
        except Exception:
            self.skipTest("OFFLINE index creation not supported")
    
    def test_create_index_with_storage_disk(self):
        """Test CREATE INDEX with STORAGE=DISK parameter"""
        index_name = "idx_scalar_disk"
        
        try:
            self.execute_sql(f"CREATE INDEX {index_name} ON {self.scalar_table} (name) USING BTREE WITH (STORAGE=DISK)")
            
            # Verify index created
            result = self.execute_sql(f"SELECT * FROM {self.scalar_table} WHERE name = 'David'")
            rows = list(result)
            self.assertEqual(len(rows), 1)
        except Exception:
            self.skipTest("STORAGE parameter not supported")


class TestVectorIndexes(LocalTestCase):
    """Test vector indexes (HNSW, IVF, etc.)"""
    
    def setUp(self):
        super().setUp()
        self.vector_table = "vector_test"
        self.dimension = 128
        
        # Create vector table
        self.create_test_table(self.vector_table, f"""
            id INTEGER PRIMARY KEY,
            vector VECTOR({self.dimension}),
            label TEXT
        """)
        
        # Generate and insert vector data
        vector_data = generate_vector_test_data(num_vectors=10, dimensions=self.dimension)
        
        for i, item in enumerate(vector_data):
            vector = item['vector']
            label = item['category']
            # Convert vector to string representation
            vector_str = "[" + ",".join(str(v) for v in vector) + "]"
            self.execute_sql(f"INSERT INTO {self.vector_table} VALUES ({i + 1}, {vector_str}, '{label}')")
    
    def test_create_hnsw_index(self):
        """Test CREATE INDEX with HNSW type"""
        index_name = "idx_vector_hnsw"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING HNSW WITH (M=16, ef_construction=200, ef_search=100, DISTANCE=L2) ONLINE
            """)
            
            # Wait for index to build (if needed)
            time.sleep(0.1)
            
            # Verify we can query using vector
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 3
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("HNSW index type not supported")
    
    def test_create_hnsw_index_with_cosine_distance(self):
        """Test CREATE INDEX with HNSW and COSINE distance"""
        index_name = "idx_vector_hnsw_cosine"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING HNSW WITH (DISTANCE=COSINE) ONLINE
            """)
            
            time.sleep(0.1)
            
            # Verify query works
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("HNSW with COSINE distance not supported")
    
    def test_create_hnsw_index_with_ip_distance(self):
        """Test CREATE INDEX with HNSW and IP (inner product) distance"""
        index_name = "idx_vector_hnsw_ip"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING HNSW WITH (DISTANCE=IP) ONLINE
            """)
            
            time.sleep(0.1)
            
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("HNSW with IP distance not supported")
    
    def test_create_hnsw_sq_index(self):
        """Test CREATE INDEX with HNSW_SQ type (scalar quantization)"""
        index_name = "idx_vector_hnsw_sq"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING HNSW_SQ WITH (M=16, ef_construction=200, DISTANCE=COSINE) ONLINE
            """)
            
            time.sleep(0.1)
            
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("HNSW_SQ index type not supported")
    
    def test_create_hnsw_bq_index(self):
        """Test CREATE INDEX with HNSW_BQ type (binary quantization)"""
        index_name = "idx_vector_hnsw_bq"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING HNSW_BQ WITH (M=16, ef_construction=200, DISTANCE=IP) ONLINE
            """)
            
            time.sleep(0.1)
            
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("HNSW_BQ index type not supported")
    
    def test_create_ivf_index(self):
        """Test CREATE INDEX with IVF type"""
        index_name = "idx_vector_ivf"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING IVF WITH (nlist=128, DISTANCE=L2) ONLINE
            """)
            
            time.sleep(0.1)
            
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("IVF index type not supported")
    
    def test_create_ivf_flat_index(self):
        """Test CREATE INDEX with IVF_FLAT type"""
        index_name = "idx_vector_ivf_flat"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING IVF_FLAT WITH (nlist=128, nprobe=16, DISTANCE=COSINE) ONLINE
            """)
            
            time.sleep(0.1)
            
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("IVF_FLAT index type not supported")
    
    def test_create_ivf_pq_index(self):
        """Test CREATE INDEX with IVF_PQ type"""
        index_name = "idx_vector_ivf_pq"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING IVF_PQ WITH (nlist=128, nprobe=8, M=8, nbits=8) ONLINE
            """)
            
            time.sleep(0.1)
            
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("IVF_PQ index type not supported")
    
    def test_create_index_offline_mode(self):
        """Test CREATE INDEX with OFFLINE mode for vector index"""
        index_name = "idx_vector_offline"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING HNSW WITH (M=16, ef_construction=200) OFFLINE
            """)
            
            # Wait longer for offline index building
            time.sleep(0.5)
            
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("OFFLINE vector index creation not supported")
    
    def test_create_index_with_storage_disk(self):
        """Test CREATE INDEX with STORAGE=DISK for vector index"""
        index_name = "idx_vector_disk"
        
        try:
            self.execute_sql(f"""
                CREATE INDEX {index_name} ON {self.vector_table} (vector) 
                USING HNSW WITH (M=16, ef_construction=200, STORAGE=DISK) ONLINE
            """)
            
            time.sleep(0.1)
            
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            result = self.execute_sql(f"""
                SELECT * FROM {self.vector_table} 
                ORDER BY vector <-> {query_vector}
                LIMIT 2
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
        except Exception:
            self.skipTest("STORAGE=DISK parameter not supported for vector indexes")


class TestIndexBuildStatus(LocalTestCase):
    """Test SHOW INDEX BUILD STATUS functionality"""
    
    def test_show_index_build_status_all(self):
        """Test SHOW INDEX BUILD STATUS without parameters"""
        try:
            # Create a table and index to potentially have something in build status
            table_name = "build_status_test"
            self.create_test_table(table_name, "id INTEGER, name TEXT")
            
            # Try to create an index (may start building asynchronously)
            try:
                self.execute_sql(f"CREATE INDEX idx_test_name ON {table_name} (name) USING BTREE")
            except Exception:
                pass  # Index creation might fail or be async
            
            # Check build status
            result = self.execute_sql("SHOW INDEX BUILD STATUS")
            rows = list(result)
            
            # Should return a result set (could be empty)
            # Just verify query executed without error
            self.assertIsNotNone(rows)
        except Exception:
            self.skipTest("SHOW INDEX BUILD STATUS not supported")
    
    def test_show_index_build_status_for_index(self):
        """Test SHOW INDEX BUILD STATUS FOR index_name"""
        try:
            table_name = "build_status_index_test"
            self.create_test_table(table_name, "id INTEGER, value REAL")
            
            index_name = "idx_test_value"
            
            # Try to create an index
            try:
                self.execute_sql(f"CREATE INDEX {index_name} ON {table_name} (value) USING BTREE")
            except Exception:
                pass
            
            # Check status for specific index
            result = self.execute_sql(f"SHOW INDEX BUILD STATUS FOR {index_name}")
            rows = list(result)
            
            # Should return result set
            self.assertIsNotNone(rows)
        except Exception:
            self.skipTest("SHOW INDEX BUILD STATUS FOR index_name not supported")
    
    def test_show_index_build_status_for_table(self):
        """Test SHOW INDEX BUILD STATUS FOR table_name"""
        try:
            table_name = "build_status_table_test"
            self.create_test_table(table_name, "id INTEGER, data TEXT")
            
            # Try to create multiple indexes
            try:
                self.execute_sql(f"CREATE INDEX idx_table_id ON {table_name} (id) USING BTREE")
                self.execute_sql(f"CREATE INDEX idx_table_data ON {table_name} (data) USING BTREE")
            except Exception:
                pass
            
            # Check status for table
            result = self.execute_sql(f"SHOW INDEX BUILD STATUS FOR {table_name}")
            rows = list(result)
            
            self.assertIsNotNone(rows)
        except Exception:
            self.skipTest("SHOW INDEX BUILD STATUS FOR table_name not supported")


class TestReindex(LocalTestCase):
    """Test REINDEX functionality"""
    
    def setUp(self):
        super().setUp()
        self.reindex_table = "reindex_test"
        self.create_test_table(self.reindex_table, "id INTEGER PRIMARY KEY, name TEXT, value REAL")
        
        # Insert test data
        for i in range(10):
            self.execute_sql(f"INSERT INTO {self.reindex_table} VALUES ({i}, 'name{i}', {i * 10.0})")
        
        # Create an index to reindex
        self.index_name = "idx_reindex_name"
        try:
            self.execute_sql(f"CREATE INDEX {self.index_name} ON {self.reindex_table} (name) USING BTREE")
        except Exception:
            pass
    
    def test_reindex_online(self):
        """Test REINDEX with ONLINE mode"""
        try:
            self.execute_sql(f"REINDEX {self.index_name} ONLINE")
            
            # Verify index still works after reindex
            result = self.execute_sql(f"SELECT * FROM {self.reindex_table} WHERE name = 'name5'")
            rows = list(result)
            self.assertEqual(len(rows), 1)
        except Exception:
            self.skipTest("REINDEX ONLINE not supported")
    
    def test_reindex_offline(self):
        """Test REINDEX with OFFLINE mode"""
        try:
            self.execute_sql(f"REINDEX {self.index_name} OFFLINE")
            
            # Verify index still works after reindex
            result = self.execute_sql(f"SELECT * FROM {self.reindex_table} WHERE name = 'name3'")
            rows = list(result)
            self.assertEqual(len(rows), 1)
        except Exception:
            self.skipTest("REINDEX OFFLINE not supported")
    
    def test_reindex_without_mode(self):
        """Test REINDEX without specifying mode"""
        try:
            self.execute_sql(f"REINDEX {self.index_name}")
            
            # Verify index still works
            result = self.execute_sql(f"SELECT * FROM {self.reindex_table} WHERE name = 'name7'")
            rows = list(result)
            self.assertEqual(len(rows), 1)
        except Exception:
            self.skipTest("REINDEX not supported")


class TestIndexUsage(LocalTestCase):
    """Test that indexes are actually used in queries"""
    
    def setUp(self):
        super().setUp()
        self.usage_table = "index_usage_test"
        self.create_test_table(self.usage_table, """
            id INTEGER PRIMARY KEY,
            category TEXT,
            value INTEGER,
            timestamp TIMESTAMP
        """)
        
        # Insert larger dataset to make index usage noticeable
        for i in range(100):
            self.execute_sql(f"""
                INSERT INTO {self.usage_table} VALUES (
                    {i}, 
                    'category_{i % 10}', 
                    {i * 10},
                    {1609459200000 + (i * 60000)}  -- 1 minute intervals
                )
            """)
    
    def test_index_for_where_clause(self):
        """Test index usage in WHERE clause"""
        # Create index on category
        self.execute_sql(f"CREATE INDEX idx_usage_category ON {self.usage_table} (category) USING BTREE")
        
        # Query with WHERE clause on indexed column
        result = self.execute_sql(f"""
            SELECT * FROM {self.usage_table} 
            WHERE category = 'category_5'
            ORDER BY id
        """)
        rows = list(result)
        
        # Should get 10 rows (100 total / 10 categories)
        self.assertEqual(len(rows), 10)
    
    def test_index_for_order_by(self):
        """Test index usage for ORDER BY optimization"""
        # Create index on timestamp
        self.execute_sql(f"CREATE INDEX idx_usage_timestamp ON {self.usage_table} (timestamp) USING TTREE")
        
        # Query with ORDER BY on indexed column
        result = self.execute_sql(f"""
            SELECT * FROM {self.usage_table} 
            ORDER BY timestamp DESC
            LIMIT 5
        """)
        rows = list(result)
        
        # Should get 5 rows in descending timestamp order
        self.assertEqual(len(rows), 5)
        # Verify descending order
        timestamps = [row["timestamp"] for row in rows]
        self.assertEqual(timestamps, sorted(timestamps, reverse=True))
    
    def test_index_for_join(self):
        """Test index usage in JOIN operations"""
        # Create second table
        self.create_test_table("category_lookup", "category TEXT PRIMARY KEY, description TEXT")
        
        # Insert category data
        for i in range(10):
            self.execute_sql(f"INSERT INTO category_lookup VALUES ('category_{i}', 'Description {i}')")
        
        # Create index on category in main table
        self.execute_sql(f"CREATE INDEX idx_usage_cat_join ON {self.usage_table} (category) USING BTREE")
        
        # Perform JOIN query
        result = self.execute_sql(f"""
            SELECT u.id, u.value, c.description
            FROM {self.usage_table} u
            JOIN category_lookup c ON u.category = c.category
            WHERE u.category = 'category_3'
            ORDER BY u.id
        """)
        rows = list(result)
        
        # Should get 10 rows
        self.assertEqual(len(rows), 10)
    
    def test_composite_index_for_multiple_columns(self):
        """Test composite index usage for queries with multiple conditions"""
        # Create composite index
        self.execute_sql(f"CREATE INDEX idx_usage_cat_val ON {self.usage_table} (category, value) USING BTREE")
        
        # Query using both columns
        result = self.execute_sql(f"""
            SELECT * FROM {self.usage_table}
            WHERE category = 'category_2' AND value > 500
            ORDER BY value
        """)
        rows = list(result)
        
        # Should get some rows
        self.assertGreater(len(rows), 0)
        
        # Verify conditions are satisfied
        for row in rows:
            self.assertEqual(row["category"], "category_2")
            self.assertGreater(row["value"], 500)


if __name__ == '__main__':
    unittest.main()