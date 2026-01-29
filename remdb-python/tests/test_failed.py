"""Test the failed test cases directly"""

import remdb

# Test zero_copy functionality
print("Testing zero_copy functionality...")
try:
    with remdb.connect(':memory:') as db:
        table = db.get_table("test_table")
        # Insert a record
        table.insert({"id": "1", "name": "test", "value": "value"})
        # Test zero_copy
        result = table.get("1", zero_copy=True)
        print(f"Zero copy result type: {type(result)}")
        print(f"Zero copy result: {result}")
except Exception as e:
    print(f"Error in zero_copy test: {e}")
    import traceback
    traceback.print_exc()

print("\nTesting query_builder functionality...")
try:
    with remdb.connect(':memory:') as db:
        table = db.get_table("test_table")
        # Test query builder
        builder = table.query()
        builder.select("id", "name").where("age > ?", 18).order("name").limit(10)
        sql, params = builder.build()
        print(f"Generated SQL: {sql}")
        print(f"Generated params: {params}")
except Exception as e:
    print(f"Error in query_builder test: {e}")
    import traceback
    traceback.print_exc()
