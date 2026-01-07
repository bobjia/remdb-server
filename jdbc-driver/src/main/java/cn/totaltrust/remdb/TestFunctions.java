package cn.totaltrust.remdb;

import java.sql.*;

public class TestFunctions {
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

            // Create tables for testing
            createTestTables(stmt);
            
            // Insert sample data
            insertTestData(stmt);
            
            // Test string functions
            testStringFunctions(stmt);
            
            // Test math functions
            testMathFunctions(stmt);
            
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
    
    private static void createTestTables(Statement stmt) throws SQLException {
        // Create table for string function tests
        String createStringTable = "CREATE TABLE IF NOT EXISTS test_strings " +
                                  "(id INT, str1 VARCHAR, str2 VARCHAR)";
        stmt.executeUpdate(createStringTable);
        System.out.println("Table 'test_strings' created or already exists");
        
        // Create table for math function tests
        String createMathTable = "CREATE TABLE IF NOT EXISTS test_math " +
                                "(id INT, num1 DOUBLE, num2 DOUBLE, num3 INT)";
        stmt.executeUpdate(createMathTable);
        System.out.println("Table 'test_math' created or already exists");
        
        // Clear existing data
        stmt.executeUpdate("DELETE FROM test_strings");
        stmt.executeUpdate("DELETE FROM test_math");
        System.out.println("Cleared existing data from test tables");
    }
    
    private static void insertTestData(Statement stmt) throws SQLException {
        // Insert data for string tests
        String[] stringInserts = {
            "INSERT INTO test_strings (id, str1, str2) VALUES (1, 'Hello', 'World')",
            "INSERT INTO test_strings (id, str1, str2) VALUES (2, 'RemDB', 'JDBC')",
            "INSERT INTO test_strings (id, str1, str2) VALUES (3, 'Test', 'Function')",
            "INSERT INTO test_strings (id, str1, str2) VALUES (4, 'String', 'Operations')",
            "INSERT INTO test_strings (id, str1, str2) VALUES (5, 'UPPERlower', 'MiXeDcAsE')"
        };
        
        for (String sql : stringInserts) {
            stmt.executeUpdate(sql);
        }
        System.out.println("Inserted 5 sample records into test_strings");
        
        // Insert data for math tests
        String[] mathInserts = {
            "INSERT INTO test_math (id, num1, num2, num3) VALUES (1, -10.5, 25.0, 7)",
            "INSERT INTO test_math (id, num1, num2, num3) VALUES (2, 3.14159, 4.0, 3)",
            "INSERT INTO test_math (id, num1, num2, num3) VALUES (3, 100.0, 0.5, 5)",
            "INSERT INTO test_math (id, num1, num2, num3) VALUES (4, 0.0, 10.0, 2)",
            "INSERT INTO test_math (id, num1, num2, num3) VALUES (5, 2.71828, 1.0, 10)"
        };
        
        for (String sql : mathInserts) {
            stmt.executeUpdate(sql);
        }
        System.out.println("Inserted 5 sample records into test_math");
    }
    
    private static void testStringFunctions(Statement stmt) throws SQLException {
        System.out.println("\n=== Testing String Functions ===");
        
        // Test CONCAT function
        System.out.println("\n1. Testing CONCAT function:");
        String concatSql = "SELECT id, str1, str2, CONCAT(str1, ' ', str2) as concatenated FROM test_strings";
        executeQuery(stmt, concatSql, "ID, STR1, STR2, CONCATENATED");
        
        // Test SUBSTRING function
        System.out.println("\n2. Testing SUBSTRING function:");
        String substringSql = "SELECT id, str1, SUBSTRING(str1, 1, 3) as substr FROM test_strings";
        executeQuery(stmt, substringSql, "ID, STR1, SUBSTRING");
        
        // Test UPPER function
        System.out.println("\n3. Testing UPPER function:");
        String upperSql = "SELECT id, str1, UPPER(str1) as uppercase FROM test_strings";
        executeQuery(stmt, upperSql, "ID, STR1, UPPERCASE");
        
        // Test LOWER function
        System.out.println("\n4. Testing LOWER function:");
        String lowerSql = "SELECT id, str2, LOWER(str2) as lowercase FROM test_strings";
        executeQuery(stmt, lowerSql, "ID, STR2, LOWERCASE");
        
        // Test combined string functions
        System.out.println("\n5. Testing combined string functions:");
        String combinedSql = "SELECT id, str1, str2, " +
                           "UPPER(CONCAT(SUBSTRING(str1, 1, 2), SUBSTRING(str2, 1, 2))) as combined " +
                           "FROM test_strings";
        executeQuery(stmt, combinedSql, "ID, STR1, STR2, COMBINED");
    }
    
    private static void testMathFunctions(Statement stmt) throws SQLException {
        System.out.println("\n=== Testing Math Functions ===");
        
        // Test ABS function
        System.out.println("\n1. Testing ABS function:");
        String absSql = "SELECT id, num1, ABS(num1) as absolute FROM test_math";
        executeQuery(stmt, absSql, "ID, NUM1, ABSOLUTE");
        
        // Test SQRT function
        System.out.println("\n2. Testing SQRT function:");
        String sqrtSql = "SELECT id, num2, SQRT(num2) as square_root FROM test_math WHERE num2 >= 0";
        executeQuery(stmt, sqrtSql, "ID, NUM2, SQUARE_ROOT");
        
        // Test POWER function
        System.out.println("\n3. Testing POWER function:");
        String powerSql = "SELECT id, num1, num2, POWER(num1, 2) as pow2, POWER(num1, num2) as powN FROM test_math WHERE num1 > 0";
        executeQuery(stmt, powerSql, "ID, NUM1, NUM2, POW2, POWN");
        
        // Test SIN and COS functions
        System.out.println("\n4. Testing SIN and COS functions:");
        String trigSql = "SELECT id, num1, SIN(num1) as sine, COS(num1) as cosine FROM test_math";
        executeQuery(stmt, trigSql, "ID, NUM1, SINE, COSINE");
        
        // Test LOG and EXP functions
        System.out.println("\n5. Testing LOG and EXP functions:");
        String logExpSql = "SELECT id, num1, LOG(num1) as logarithm, EXP(num1) as exponential FROM test_math WHERE num1 > 0";
        executeQuery(stmt, logExpSql, "ID, NUM1, LOGARITHM, EXPONENTIAL");
        
        // Test ROUND, CEIL, FLOOR functions
        System.out.println("\n6. Testing ROUND, CEIL, FLOOR functions:");
        String roundSql = "SELECT id, num1, " +
                        "ROUND(num1) as rounded, " +
                        "CEIL(num1) as ceiling, " +
                        "FLOOR(num1) as floor " +
                        "FROM test_math";
        executeQuery(stmt, roundSql, "ID, NUM1, ROUNDED, CEILING, FLOOR");
        
        // Test MOD function
        System.out.println("\n7. Testing MOD function:");
        String modSql = "SELECT id, num1, num3, MOD(num1, num3) as modulus FROM test_math WHERE num3 != 0";
        executeQuery(stmt, modSql, "ID, NUM1, NUM3, MODULUS");
        
        // Test combined math functions
        System.out.println("\n8. Testing combined math functions:");
        String combinedSql = "SELECT id, num1, num2, " +
                           "ROUND(SQRT(ABS(POWER(num1, 2) + POWER(num2, 2))), 2) as combined_result " +
                           "FROM test_math";
        executeQuery(stmt, combinedSql, "ID, NUM1, NUM2, COMBINED_RESULT");
    }
    
    private static void executeQuery(Statement stmt, String sql, String columns) throws SQLException {
        System.out.println("Executing: " + sql);
        System.out.println("Columns: " + columns);
        System.out.println("-" .repeat(80));
        
        try (ResultSet rs = stmt.executeQuery(sql)) {
            while (rs.next()) {
                // Print all columns in the result set
                ResultSetMetaData metaData = rs.getMetaData();
                int columnCount = metaData.getColumnCount();
                
                for (int i = 1; i <= columnCount; i++) {
                    String columnName = metaData.getColumnName(i);
                    Object value = rs.getObject(i);
                    
                    if (value instanceof Double) {
                        System.out.printf("%s: %.4f  ", columnName, (Double) value);
                    } else {
                        System.out.printf("%s: %s  ", columnName, value);
                    }
                }
                System.out.println();
            }
        } catch (SQLException e) {
            System.out.println("Error: Function not supported: " + e.getMessage());
        }
        System.out.println("-" .repeat(80));
    }
}