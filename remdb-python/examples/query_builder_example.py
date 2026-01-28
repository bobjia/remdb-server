"""
Query Builder Example for RemDB

This example demonstrates how to use the QueryBuilder class in RemDB
to build and execute safe SQL queries, preventing SQL injection.
"""

import remdb


def main():
    """Main function"""
    # Connect to an in-memory database
    with remdb.connect() as db:
        try:
            # Get the table (assuming it already exists)
            table = db.get_table("products")
            
            # Example 1: Basic SELECT query
            print("=== Example 1: Basic SELECT query ===")
            builder1 = table.query()
            builder1.select("id", "name", "price").where("category = ?", "electronics").order("price").limit(10)
            sql, params = builder1.build()
            print(f"Generated SQL: {sql}")
            print(f"Parameters: {params}")
            
            # Execute the query
            result_set = builder1.execute(db)
            print(f"Found {result_set.get_rows_count()} products in electronics category:")
            for row in result_set:
                print(f"  ID: {row['id']}, Name: {row['name']}, Price: ${row['price']:.2f}")
            
            # Example 2: More complex query with multiple conditions
            print("\n=== Example 2: Complex query with multiple conditions ===")
            builder2 = table.query()
            builder2.select("id", "name", "price", "category")\
                   .where("price >= ?", 50)\
                   .where("price <= ?", 200)\
                   .where("category IN (?, ?)", "electronics", "home")\
                   .order("price", ascending=False)\
                   .limit(5)
            sql, params = builder2.build()
            print(f"Generated SQL: {sql}")
            print(f"Parameters: {params}")
            
            # Execute the query
            result_set = builder2.execute(db)
            print(f"Found {result_set.get_rows_count()} products in price range $50-$200:")
            for row in result_set:
                print(f"  ID: {row['id']}, Name: {row['name']}, Price: ${row['price']:.2f}, Category: {row['category']}")
            
            # Example 3: Query with all columns
            print("\n=== Example 3: Query with all columns ===")
            builder3 = table.query()
            builder3.select().where("name LIKE ?", "%phone%").limit(3)
            sql, params = builder3.build()
            print(f"Generated SQL: {sql}")
            print(f"Parameters: {params}")
            
        except remdb.NotFoundError:
            print("Table not found. Please create a table first.")
            print("Example schema:")
            print("CREATE TABLE products (")
            print("    id INTEGER PRIMARY KEY,")
            print("    name TEXT,")
            print("    price REAL,")
            print("    category TEXT")
            print(")")
        except Exception as e:
            print(f"Error: {e}")


if __name__ == "__main__":
    main()
