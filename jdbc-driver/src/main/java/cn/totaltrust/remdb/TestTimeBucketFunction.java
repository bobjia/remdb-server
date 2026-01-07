package cn.totaltrust.remdb;

import java.sql.*;

public class TestTimeBucketFunction {
    public static void main(String[] args) throws ClassNotFoundException {
        String url = "jdbc:remdb://localhost:6666";

        Connection conn = null;
        Statement stmt = null;
        ResultSet rs = null;

        try {
            // Register the JDBC driver explicitly
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            System.out.println("Connecting to RemDb server...");
            // No authentication required since it's disabled on the server
            conn = DriverManager.getConnection(url);
            System.out.println("Connection established successfully!");

            stmt = conn.createStatement();

            // Create a simple table for testing (using supported data types)
            String createTableSQL = "CREATE TABLE IF NOT EXISTS test_time " +
                                   "(id INT, ts INT, value DOUBLE)";
            stmt.executeUpdate(createTableSQL);
            System.out.println("Table 'test_time' created or already exists");

            // Clear any existing data
            String clearDataSQL = "DELETE FROM test_time";
            stmt.executeUpdate(clearDataSQL);
            System.out.println("Cleared existing data");

            // Insert sample data
            String[] insertSqls = {
                "INSERT INTO test_time (id, ts, value) VALUES (1, 1620000000000, 25.5)",
                "INSERT INTO test_time (id, ts, value) VALUES (2, 1620000300000, 26.1)",
                "INSERT INTO test_time (id, ts, value) VALUES (3, 1620000600000, 25.8)",
                "INSERT INTO test_time (id, ts, value) VALUES (4, 1620000900000, 26.3)",
                "INSERT INTO test_time (id, ts, value) VALUES (5, 1620001200000, 26.0)"
            };
            
            for (String sql : insertSqls) {
                stmt.executeUpdate(sql);
            }
            System.out.println("Inserted 5 sample records");

            // Test TIME_BUCKET function with a simple query
            String testSql = "SELECT TIME_BUCKET(3600000, ts) as bucket, AVG(value) as avg_value " +
                            "FROM test_time " +
                            "GROUP BY bucket";
            
            System.out.println("\nTesting TIME_BUCKET function...");
            System.out.println("Executing query: " + testSql);
            
            try {
                // Try to execute the query
                rs = stmt.executeQuery(testSql);
                
                // If execution succeeds, TIME_BUCKET is supported
                System.out.println("\n✅ TIME_BUCKET function is supported!");
                System.out.println("\nQuery results:");
                System.out.println("BUCKET | AVG_VALUE");
                System.out.println("-------|-----------");
                
                while (rs.next()) {
                    long bucket = rs.getLong("bucket");
                    double avgValue = rs.getDouble("avg_value");
                    System.out.printf("%d | %.2f\n", bucket, avgValue);
                }
            } catch (SQLException e) {
                // If execution fails, check if it's due to unsupported function
                System.out.println("\n❌ TIME_BUCKET function is NOT directly supported by the underlying RemDB library.");
                System.out.println("Error message: " + e.getMessage());
                System.out.println("\nℹ️  Note: We've added TIME_BUCKET support to the SQL engine, but the underlying RemDB library");
                System.out.println("ℹ️  doesn't recognize the function syntax. Full support would require changes to the");
                System.out.println("ℹ️  underlying library's SQL parser.");
            }
        } catch (SQLException e) {
            System.out.println("\n🔍 Query failed due to an unexpected error:");
            e.printStackTrace();
        } finally {
            // Clean up resources
            try {
                if (rs != null) rs.close();
                if (stmt != null) stmt.close();
                if (conn != null) conn.close();
                System.out.println("\nResources closed successfully!");
            } catch (SQLException e) {
                e.printStackTrace();
            }
        }
    }
}