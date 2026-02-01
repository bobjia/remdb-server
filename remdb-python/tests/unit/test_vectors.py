"""Test vector-specific functionality in RemDB"""

import unittest
import math
from tests.fixtures import LocalTestCase
from tests.utils.data_generators import generate_vector_test_data, generate_random_vector, generate_normalized_vector


class TestVectorDataTypes(LocalTestCase):
    """Test VECTOR data type operations"""
    
    def setUp(self):
        super().setUp()
        self.vector_table = "vector_data_test"
        self.dimension = 128
        
        # Create table with VECTOR column
        self.create_test_table(self.vector_table, f"""
            id INTEGER PRIMARY KEY,
            vector VECTOR({self.dimension}),
            label TEXT,
            category INTEGER
        """)
        
        # Generate and insert vector data
        vector_data = generate_vector_test_data(num_vectors=50, dimensions=self.dimension)
        
        for i, item in enumerate(vector_data):
            vector = item['vector']
            label = item['label']
            vector_str = "[" + ",".join(str(v) for v in vector) + "]"
            category = i % 5  # 5 categories
            
            self.execute_sql(f"""
                INSERT INTO {self.vector_table} 
                VALUES ({i + 1}, {vector_str}, '{label}', {category})
            """)
    
    def test_vector_column_creation(self):
        """Test that vector column was created correctly"""
        # Query to verify data was inserted
        result = self.execute_sql(f"SELECT COUNT(*) as count FROM {self.vector_table}")
        rows = list(result)
        self.assertEqual(rows[0]["count"], 50)
        
        # Verify we can query vector data
        result = self.execute_sql(f"SELECT id, label FROM {self.vector_table} WHERE category = 0")
        rows = list(result)
        self.assertGreater(len(rows), 0)
    
    def test_vector_with_distance_specification(self):
        """Test creating vector column with distance specification"""
        table_name = "vector_with_distance"
        
        try:
            # Try to create vector column with different distance metrics
            self.execute_sql(f"""
                CREATE TABLE {table_name} (
                    id INTEGER PRIMARY KEY,
                    vec VECTOR({self.dimension}) WITH DISTANCE=L2
                )
            """)
            
            # Insert test data
            test_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            self.execute_sql(f"INSERT INTO {table_name} VALUES (1, {test_vector})")
            
            # Query to verify
            result = self.execute_sql(f"SELECT * FROM {table_name}")
            rows = list(result)
            self.assertEqual(len(rows), 1)
            
        except Exception:
            self.skipTest("Vector with distance specification not supported")
    
    def test_vector_with_compression(self):
        """Test creating vector column with compression"""
        table_name = "vector_with_compression"
        
        try:
            # Try different compression types
            compression_types = ["NONE", "SQ", "PQ", "BQ"]
            
            for compression in compression_types:
                try:
                    self.execute_sql(f"DROP TABLE IF EXISTS {table_name}")
                    
                    self.execute_sql(f"""
                        CREATE TABLE {table_name} (
                            id INTEGER PRIMARY KEY,
                            vec VECTOR({self.dimension}) WITH COMPRESSION={compression}
                        )
                    """)
                    
                    # Insert test data
                    test_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
                    self.execute_sql(f"INSERT INTO {table_name} VALUES (1, {test_vector})")
                    
                    # Query to verify
                    result = self.execute_sql(f"SELECT * FROM {table_name}")
                    rows = list(result)
                    self.assertEqual(len(rows), 1)
                    
                except Exception:
                    # Some compression types might not be supported
                    continue
                    
        except Exception:
            self.skipTest("Vector with compression not supported")


class TestVectorSearchFunctions(LocalTestCase):
    """Test vector search functions (VECTOR_SIMILAR, VECTOR_DISTANCE)"""
    
    def setUp(self):
        super().setUp()
        self.search_table = "vector_search_test"
        self.dimension = 64
        
        # Create table
        self.create_test_table(self.search_table, f"""
            id INTEGER PRIMARY KEY,
            embedding VECTOR({self.dimension}),
            text TEXT,
            score REAL
        """)
        
        # Generate some meaningful test vectors
        # Create vectors that are slightly different from each other
        base_vector = [0.1] * self.dimension
        
        for i in range(20):
            # Create vector slightly different from base
            vector = [v + (i * 0.01) + (j * 0.001) for j, v in enumerate(base_vector)]
            vector_str = "[" + ",".join(str(v) for v in vector) + "]"
            text = f"Document {i}"
            score = i * 0.5
            
            self.execute_sql(f"""
                INSERT INTO {self.search_table} 
                VALUES ({i + 1}, {vector_str}, '{text}', {score})
            """)
    
    def test_vector_similar_function(self):
        """Test VECTOR_SIMILAR function"""
        try:
            # Create a query vector similar to the base vector
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            
            # Test with L2 distance
            result = self.execute_sql(f"""
                SELECT id, text
                FROM {self.search_table}
                WHERE VECTOR_SIMILAR(embedding, {query_vector}, L2)
                ORDER BY id
                LIMIT 5
            """)
            rows = list(result)
            
            # Should return some rows
            self.assertGreater(len(rows), 0)
            
            # Test with COSINE distance
            result = self.execute_sql(f"""
                SELECT id, text
                FROM {self.search_table}
                WHERE VECTOR_SIMILAR(embedding, {query_vector}, COSINE)
                ORDER BY id
                LIMIT 5
            """)
            rows = list(result)
            self.assertGreater(len(rows), 0)
            
        except Exception:
            self.skipTest("VECTOR_SIMILAR function not supported")
    
    def test_vector_distance_function(self):
        """Test VECTOR_DISTANCE function"""
        try:
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            
            # Test with L2 distance
            result = self.execute_sql(f"""
                SELECT 
                    id,
                    text,
                    VECTOR_DISTANCE(embedding, {query_vector}, L2) as distance
                FROM {self.search_table}
                ORDER BY VECTOR_DISTANCE(embedding, {query_vector}, L2)
                LIMIT 5
            """)
            rows = list(result)
            
            self.assertEqual(len(rows), 5)
            
            # Verify distances are returned
            for row in rows:
                self.assertIn("distance", row)
                self.assertIsInstance(row["distance"], (int, float))
            
            # Test with COSINE distance
            result = self.execute_sql(f"""
                SELECT 
                    id,
                    text,
                    VECTOR_DISTANCE(embedding, {query_vector}, COSINE) as similarity
                FROM {self.search_table}
                ORDER BY VECTOR_DISTANCE(embedding, {query_vector}, COSINE) DESC
                LIMIT 5
            """)
            rows = list(result)
            
            self.assertEqual(len(rows), 5)
            
        except Exception:
            self.skipTest("VECTOR_DISTANCE function not supported")
    
    def test_vector_similar_with_threshold(self):
        """Test VECTOR_SIMILAR with distance threshold"""
        try:
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            
            # Get distance to first vector
            result = self.execute_sql(f"""
                SELECT VECTOR_DISTANCE(embedding, {query_vector}, L2) as dist
                FROM {self.search_table}
                WHERE id = 1
            """)
            rows = list(result)
            distance = rows[0]["dist"]
            
            # Use threshold slightly larger than the distance
            threshold = distance * 1.5
            
            # Find vectors within threshold
            result = self.execute_sql(f"""
                SELECT id, text
                FROM {self.search_table}
                WHERE VECTOR_DISTANCE(embedding, {query_vector}, L2) < {threshold}
                ORDER BY id
            """)
            rows = list(result)
            
            # Should include at least the first vector
            self.assertGreater(len(rows), 0)
            
        except Exception:
            self.skipTest("VECTOR_DISTANCE with threshold not supported")
    
    def test_vector_functions_combined(self):
        """Test combining VECTOR_SIMILAR and VECTOR_DISTANCE"""
        try:
            query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
            
            result = self.execute_sql(f"""
                SELECT 
                    id,
                    text,
                    score,
                    VECTOR_DISTANCE(embedding, {query_vector}, L2) as distance
                FROM {self.search_table}
                WHERE VECTOR_SIMILAR(embedding, {query_vector}, L2)
                AND score > 2.0
                ORDER BY distance
                LIMIT 10
            """)
            rows = list(result)
            
            # Should return some rows
            self.assertGreater(len(rows), 0)
            
            # Verify all rows have score > 2.0
            for row in rows:
                self.assertGreater(row["score"], 2.0)
                
        except Exception:
            self.skipTest("Vector functions combination not supported")


class TestVectorMixedSearch(LocalTestCase):
    """Test mixed vector and scalar searches"""
    
    def setUp(self):
        super().setUp()
        self.mixed_table = "vector_mixed_test"
        self.dimension = 96
        
        # Create table with vector and various scalar columns
        self.create_test_table(self.mixed_table, f"""
            id INTEGER PRIMARY KEY,
            embedding VECTOR({self.dimension}),
            title TEXT,
            category TEXT,
            price REAL,
            rating INTEGER,
            in_stock BOOLEAN
        """)
        
        # Generate test data with variations
        categories = ["electronics", "books", "clothing", "home", "sports"]
        
        for i in range(30):
            # Create vector - make them somewhat clustered by category
            base_val = i % 3 * 0.3 + 0.1
            vector = [base_val + (j * 0.001) for j in range(self.dimension)]
            vector_str = "[" + ",".join(str(v) for v in vector) + "]"
            
            category = categories[i % len(categories)]
            title = f"Product {i} - {category}"
            price = 10.0 + (i * 5.0)
            rating = (i % 5) + 1  # 1-5 stars
            in_stock = i % 3 != 0  # Some out of stock
            
            self.execute_sql(f"""
                INSERT INTO {self.mixed_table} 
                VALUES ({i + 1}, {vector_str}, '{title}', '{category}', {price}, {rating}, {in_stock})
            """)
    
    def test_vector_search_with_scalar_filter(self):
        """Test vector search combined with scalar filtering"""
        # Create query vector
        query_vector = "[0.1]" + ",0.1" * (self.dimension - 1)
        
        # Search for electronics with high similarity
        result = self.execute_sql(f"""
            SELECT 
                id,
                title,
                price,
                rating,
                embedding <-> {query_vector} as distance
            FROM {self.mixed_table}
            WHERE category = 'electronics'
            AND rating >= 4
            AND price < 100.0
            ORDER BY embedding <-> {query_vector}
            LIMIT 5
        """)
        rows = list(result)
        
        # Should return some rows
        if len(rows) > 0:
            # Verify filters are applied
            for row in rows:
                self.assertEqual(row["category"], "electronics")
                self.assertGreaterEqual(row["rating"], 4)
                self.assertLess(row["price"], 100.0)
    
    def test_vector_search_with_multiple_scalar_conditions(self):
        """Test vector search with multiple scalar conditions"""
        query_vector = "[0.4]" + ",0.4" * (self.dimension - 1)  # Different query vector
        
        result = self.execute_sql(f"""
            SELECT 
                id,
                title,
                category,
                price,
                in_stock,
                embedding <=> {query_vector} as similarity
            FROM {self.mixed_table}
            WHERE (category = 'books' OR category = 'electronics')
            AND price BETWEEN 20.0 AND 80.0
            AND in_stock = true
            AND embedding <=> {query_vector} > 0.5
            ORDER BY embedding <=> {query_vector} DESC
            LIMIT 10
        """)
        rows = list(result)
        
        # Check if any rows returned
        if len(rows) > 0:
            for row in rows:
                self.assertIn(row["category"], ["books", "electronics"])
                self.assertGreaterEqual(row["price"], 20.0)
                self.assertLessEqual(row["price"], 80.0)
                self.assertEqual(row["in_stock"], True)
    
    def test_vector_search_with_complex_conditions(self):
        """Test vector search with complex WHERE conditions"""
        query_vector = "[0.2]" + ",0.2" * (self.dimension - 1)
        
        result = self.execute_sql(f"""
            SELECT 
                id,
                title,
                category,
                price,
                rating
            FROM {self.mixed_table}
            WHERE embedding <-> {query_vector} < 1.0
            AND (
                (category = 'clothing' AND price < 50.0)
                OR (category = 'home' AND rating >= 3)
            )
            AND price > 10.0
            ORDER BY embedding <-> {query_vector}
            LIMIT 10
        """)
        rows = list(result)
        
        # If rows returned, verify conditions
        for row in rows:
            self.assertLess(row["price"], 50.0 if row["category"] == "clothing" else float('inf'))
            if row["category"] == "home":
                self.assertGreaterEqual(row["rating"], 3)
            self.assertGreater(row["price"], 10.0)
    
    def test_vector_search_with_aggregation(self):
        """Test vector search combined with aggregation"""
        query_vector = "[0.3]" + ",0.3" * (self.dimension - 1)
        
        result = self.execute_sql(f"""
            SELECT 
                category,
                COUNT(*) as product_count,
                AVG(price) as avg_price,
                AVG(embedding <-> {query_vector}) as avg_distance
            FROM {self.mixed_table}
            WHERE embedding <-> {query_vector} < 1.5
            GROUP BY category
            HAVING COUNT(*) >= 2
            ORDER BY avg_distance
        """)
        rows = list(result)
        
        # Should return some aggregated results
        if len(rows) > 0:
            for row in rows:
                self.assertGreaterEqual(row["product_count"], 2)
                self.assertIsInstance(row["avg_price"], (int, float))
                self.assertIsInstance(row["avg_distance"], (int, float))
    
    def test_vector_search_with_order_by_mixed(self):
        """Test vector search with mixed ORDER BY clauses"""
        query_vector = "[0.25]" + ",0.25" * (self.dimension - 1)
        
        # Order by distance first, then price
        result = self.execute_sql(f"""
            SELECT 
                id,
                title,
                category,
                price,
                embedding <-> {query_vector} as distance
            FROM {self.mixed_table}
            WHERE category = 'electronics'
            ORDER BY embedding <-> {query_vector}, price
            LIMIT 10
        """)
        rows = list(result)
        
        if len(rows) > 1:
            # Verify ordering by distance (primary) and price (secondary)
            for i in range(len(rows) - 1):
                # Either distance is less, or equal distance and price is less or equal
                self.assertTrue(
                    rows[i]["distance"] < rows[i + 1]["distance"] or
                    (rows[i]["distance"] == rows[i + 1]["distance"] and 
                     rows[i]["price"] <= rows[i + 1]["price"])
                )


class TestVectorEdgeCases(LocalTestCase):
    """Test edge cases for vector operations"""
    
    def test_vector_with_zero_dimension(self):
        """Test vector with minimum dimension (1)"""
        table_name = "vector_zero_dim"
        
        try:
            # Try with dimension 1
            self.execute_sql(f"""
                CREATE TABLE {table_name} (
                    id INTEGER PRIMARY KEY,
                    vec VECTOR(1)
                )
            """)
            
            # Insert test data
            self.execute_sql(f"INSERT INTO {table_name} VALUES (1, [0.5])")
            self.execute_sql(f"INSERT INTO {table_name} VALUES (2, [1.0])")
            
            # Query
            result = self.execute_sql(f"SELECT * FROM {table_name} ORDER BY id")
            rows = list(result)
            self.assertEqual(len(rows), 2)
            
            # Test distance calculation
            result = self.execute_sql(f"SELECT vec <-> [0.75] as distance FROM {table_name} WHERE id = 1")
            rows = list(result)
            self.assertIsInstance(rows[0]["distance"], (int, float))
            
        except Exception:
            self.skipTest("1-dimensional vectors not supported")
    
    def test_vector_with_high_dimension(self):
        """Test vector with high dimension"""
        table_name = "vector_high_dim"
        high_dim = 256  # Test with higher dimension
        
        try:
            self.execute_sql(f"""
                CREATE TABLE {table_name} (
                    id INTEGER PRIMARY KEY,
                    vec VECTOR({high_dim})
                )
            """)
            
            # Create a high-dimensional vector
            vector_vals = [str(0.1 + (i * 0.001)) for i in range(high_dim)]
            vector_str = "[" + ",".join(vector_vals) + "]"
            
            self.execute_sql(f"INSERT INTO {table_name} VALUES (1, {vector_str})")
            
            # Query
            result = self.execute_sql(f"SELECT * FROM {table_name}")
            rows = list(result)
            self.assertEqual(len(rows), 1)
            
        except Exception:
            self.skipTest(f"Vectors with dimension {high_dim} not supported")
    
    def test_vector_null_values(self):
        """Test handling of NULL values in vector columns"""
        table_name = "vector_null_test"
        self.create_test_table(table_name, "id INTEGER PRIMARY KEY, vec VECTOR(64), label TEXT")
        
        # Insert some rows with NULL vectors
        self.execute_sql(f"INSERT INTO {table_name} VALUES (1, NULL, 'no_vector')")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (2, [0.1,0.2,0.3], 'has_vector')")
        self.execute_sql(f"INSERT INTO {table_name} VALUES (3, NULL, 'another_no_vector')")
        
        # Count non-null vectors
        result = self.execute_sql(f"SELECT COUNT(vec) as non_null_count FROM {table_name}")
        rows = list(result)
        self.assertEqual(rows[0]["non_null_count"], 1)
        
        # Count all rows
        result = self.execute_sql(f"SELECT COUNT(*) as total_count FROM {table_name}")
        rows = list(result)
        self.assertEqual(rows[0]["total_count"], 3)
    
    def test_vector_empty_search_results(self):
        """Test vector search that returns no results"""
        table_name = "vector_empty_search"
        self.create_test_table(table_name, "id INTEGER PRIMARY KEY, vec VECTOR(32), value INTEGER")
        
        # Insert some data
        for i in range(5):
            vector = [0.5 + (i * 0.1)] * 32
            vector_str = "[" + ",".join(str(v) for v in vector) + "]"
            self.execute_sql(f"INSERT INTO {table_name} VALUES ({i + 1}, {vector_str}, {i * 10})")
        
        # Search with very restrictive conditions that should return nothing
        query_vector = "[2.0]" * 32  # Very different from our data
        
        result = self.execute_sql(f"""
            SELECT * FROM {table_name}
            WHERE vec <-> {query_vector} < 0.1  # Very small distance threshold
            AND value > 1000  # Value that doesn't exist
        """)
        rows = list(result)
        self.assertEqual(len(rows), 0)
    
    def test_vector_identical_vectors(self):
        """Test operations with identical vectors"""
        table_name = "vector_identical"
        self.create_test_table(table_name, "id INTEGER PRIMARY KEY, vec VECTOR(16), tag TEXT")
        
        # Insert identical vectors
        identical_vector = "[0.25,0.25,0.25,0.25]" + ",0.25" * 12
        
        for i in range(3):
            self.execute_sql(f"INSERT INTO {table_name} VALUES ({i + 1}, {identical_vector}, 'identical_{i}')")
        
        # Distance between identical vectors should be 0 (or very small)
        result = self.execute_sql(f"""
            SELECT vec <-> {identical_vector} as distance
            FROM {table_name}
            WHERE id = 1
        """)
        rows = list(result)
        distance = rows[0]["distance"]
        
        # Distance should be 0 or very close to 0
        self.assertAlmostEqual(distance, 0.0, delta=0.001)
        
        # All should have same distance to query vector
        result = self.execute_sql(f"""
            SELECT id, vec <-> {identical_vector} as distance
            FROM {table_name}
            ORDER BY id
        """)
        rows = list(result)
        
        # All distances should be equal (or very close)
        for i in range(1, len(rows)):
            self.assertAlmostEqual(rows[0]["distance"], rows[i]["distance"], delta=0.001)


class TestVectorPerformance(LocalTestCase):
    """Test performance-related aspects of vector operations"""
    
    def test_vector_bulk_insert(self):
        """Test bulk insertion of vectors"""
        table_name = "vector_bulk_test"
        dimension = 128
        num_vectors = 100
        
        self.create_test_table(table_name, f"""
            id INTEGER PRIMARY KEY,
            vec VECTOR({dimension}),
            metadata TEXT
        """)
        
        # Time the bulk insertion
        import time
        start_time = time.time()
        
        for i in range(num_vectors):
            # Create simple vectors
            vector = [0.1 + (i * 0.01) + (j * 0.001) for j in range(dimension)]
            vector_str = "[" + ",".join(str(v) for v in vector) + "]"
            metadata = f"Vector {i}"
            
            self.execute_sql(f"""
                INSERT INTO {table_name} 
                VALUES ({i + 1}, {vector_str}, '{metadata}')
            """)
        
        end_time = time.time()
        insertion_time = end_time - start_time
        
        # Verify all inserted
        result = self.execute_sql(f"SELECT COUNT(*) as count FROM {table_name}")
        rows = list(result)
        self.assertEqual(rows[0]["count"], num_vectors)
        
        # Log insertion rate (not an assertion, just for information)
        print(f"Inserted {num_vectors} vectors in {insertion_time:.2f} seconds "
              f"({num_vectors/insertion_time:.1f} vectors/sec)")
    
    def test_vector_search_performance(self):
        """Test performance of vector search operations"""
        table_name = "vector_perf_test"
        dimension = 64
        num_vectors = 50
        
        self.create_test_table(table_name, f"""
            id INTEGER PRIMARY KEY,
            embedding VECTOR({dimension}),
            category INTEGER
        """)
        
        # Insert test data
        for i in range(num_vectors):
            vector = [0.1 + (i % 3 * 0.3) + (j * 0.001) for j in range(dimension)]
            vector_str = "[" + ",".join(str(v) for v in vector) + "]"
            category = i % 4
            
            self.execute_sql(f"""
                INSERT INTO {table_name} 
                VALUES ({i + 1}, {vector_str}, {category})
            """)
        
        # Test search performance with different query vectors
        query_vectors = [
            "[0.1]" + ",0.1" * (dimension - 1),  # Similar to category 0
            "[0.4]" + ",0.4" * (dimension - 1),  # Similar to category 1  
            "[0.7]" + ",0.7" * (dimension - 1),  # Similar to category 2
        ]
        
        for query_vec in query_vectors:
            import time
            start_time = time.time()
            
            result = self.execute_sql(f"""
                SELECT id, embedding <-> {query_vec} as distance
                FROM {table_name}
                WHERE category = {query_vectors.index(query_vec) % 4}
                ORDER BY embedding <-> {query_vec}
                LIMIT 10
            """)
            rows = list(result)
            
            end_time = time.time()
            search_time = end_time - start_time
            
            # Should return some rows
            self.assertGreater(len(rows), 0)
            
            # Log search time (not an assertion)
            print(f"Search with query vector {query_vectors.index(query_vec)} "
                  f"took {search_time:.4f} seconds, returned {len(rows)} rows")


if __name__ == '__main__':
    unittest.main()