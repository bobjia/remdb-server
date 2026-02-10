import remdb

# Test basic functionality
print("Testing RemDB Python bindings...")

# Test connection
with remdb.connect("test_db.rdb") as db:
    print("✓ Successfully connected to database")
    
    # Test table creation
    db.execute_query("CREATE TABLE IF NOT EXISTS test_table (id INT PRIMARY KEY, name VARCHAR(50), value DOUBLE)")
    print("✓ Successfully created table")
    
    # Test insertion
    db.execute_query("INSERT INTO test_table (id, name, value) VALUES (1, 'test', 3.14)")
    print("✓ Successfully inserted data")
    
    # Test query
    result = db.execute_query("SELECT * FROM test_table")
    print("✓ Successfully executed query")
    
    # Test result retrieval
    for row in result:
        print(f"✓ Retrieved data: {row}")

print("\nAll tests passed! RemDB Python bindings are working correctly.")
