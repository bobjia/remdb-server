#!/usr/bin/env python3
"""
Test script for RemDB Python bindings
"""

import remdb

# Test basic functionality
def test_basic():
    print("Testing basic functionality...")
    
    # Connect to in-memory database
    conn = remdb.connect("")
    print("Connected to database")
    
    # Get or create a table
    table = conn.get_table("test_table")
    print("Got table: test_table")
    
    # Insert a record
    record = {"id": "1", "name": "test", "value": "123"}
    success = table.insert(record)
    print(f"Inserted record: {success}")
    
    # Get the record
    retrieved = table.get("1")
    print(f"Retrieved record: {retrieved}")
    
    # Test vector search
    print("Testing vector search...")
    query_vector = [0.1, 0.2, 0.3]
    results = table.vector_search("embedding", query_vector, k=5)
    print(f"Vector search results: {results}")
    
    # Test hybrid search
    print("Testing hybrid search...")
    filter_expr = "value > 100"
    hybrid_results = table.hybrid_search("embedding", query_vector, filter_expr, k=5)
    print(f"Hybrid search results: {hybrid_results}")
    
    # Close connection
    conn.close()
    print("Closed connection")

if __name__ == "__main__":
    test_basic()
    print("All tests completed!")
