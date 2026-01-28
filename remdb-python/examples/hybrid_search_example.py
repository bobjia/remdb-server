"""
Hybrid Search Example for RemDB

This example demonstrates how to use the hybrid search functionality in RemDB,
which combines vector search with scalar filtering.
"""

import remdb


def main():
    """Main function"""
    # Connect to an in-memory database
    with remdb.connect() as db:
        try:
            # Get the table (assuming it already exists with vector and scalar data)
            table = db.get_table("products")
            
            # Example: Search for similar products in a specific price range
            # Assume we have a product embedding vector
            query_vector = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
            
            # Define scalar filter: products with price between $10 and $50
            filter_expr = "price >= 10 AND price <= 50"
            
            # Perform hybrid search
            print("Performing hybrid search...")
            results = table.hybrid_search("embedding", query_vector, filter_expr, k=5)
            
            # Display results
            print(f"Found {len(results)} results:")
            for i, result in enumerate(results, 1):
                product_id = result["id"]
                distance = result["distance"]
                print(f"{i}. Product ID: {product_id}, Distance: {distance:.4f}")
                
                # Optionally, get the full product details
                product = table.get(product_id)
                if product:
                    print(f"   Product details: {product}")
                    
        except remdb.NotFoundError:
            print("Table not found. Please create a table with vector and scalar data first.")
            print("Example schema:")
            print("CREATE TABLE products (")
            print("    id INTEGER PRIMARY KEY,")
            print("    name TEXT,")
            print("    price REAL,")
            print("    category TEXT,")
            print("    embedding VECTOR(10)")  # 10-dimensional vector
            print(")")
        except Exception as e:
            print(f"Error: {e}")


if __name__ == "__main__":
    main()
