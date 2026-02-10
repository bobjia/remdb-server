import java.sql.*;

public class DebugColumnOrder {
    public static void main(String[] args) {
        String jdbcUrl = "jdbc:remdb://localhost:6666/default";
        try {
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            Connection connection = DriverManager.getConnection(jdbcUrl);
            Statement statement = connection.createStatement();
            
            // Clean up any existing test table
            try {
                statement.executeUpdate("DROP TABLE IF EXISTS test_column_order");
            } catch (SQLException e) {
                System.out.println("Error dropping table: " + e.getMessage());
            }
            
            // Create table with initial columns
            System.out.println("Creating table...");
            statement.executeUpdate("CREATE TABLE IF NOT EXISTS test_column_order (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL)");
            
            // Add new columns
            System.out.println("Adding columns...");
            statement.executeUpdate("ALTER TABLE test_column_order ADD COLUMN age INTEGER");
            statement.executeUpdate("ALTER TABLE test_column_order ADD COLUMN email TEXT");
            
            // Insert data
            System.out.println("Inserting data...");
            statement.executeUpdate("INSERT INTO test_column_order (name, age, email) VALUES ('Bob', 30, 'bob@example.com')");
            
            // Test 1: SELECT *
            System.out.println("\n=== Test 1: SELECT * ===");
            ResultSet rs1 = statement.executeQuery("SELECT * FROM test_column_order");
            ResultSetMetaData meta1 = rs1.getMetaData();
            System.out.println("Columns:");
            for (int i = 1; i <= meta1.getColumnCount(); i++) {
                System.out.println(i + ": " + meta1.getColumnName(i));
            }
            while (rs1.next()) {
                System.out.println("\nRow data:");
                for (int i = 1; i <= meta1.getColumnCount(); i++) {
                    System.out.println(meta1.getColumnName(i) + ": " + rs1.getString(i));
                }
            }
            rs1.close();
            
            // Test 2: SELECT age, email
            System.out.println("\n=== Test 2: SELECT age, email ===");
            ResultSet rs2 = statement.executeQuery("SELECT age, email FROM test_column_order WHERE name = 'Bob'");
            ResultSetMetaData meta2 = rs2.getMetaData();
            System.out.println("Columns:");
            for (int i = 1; i <= meta2.getColumnCount(); i++) {
                System.out.println(i + ": " + meta2.getColumnName(i));
            }
            while (rs2.next()) {
                System.out.println("\nRow data:");
                for (int i = 1; i <= meta2.getColumnCount(); i++) {
                    System.out.println(meta2.getColumnName(i) + ": " + rs2.getString(i));
                }
                // Test the specific calls from the test
                int age = rs2.getInt("age");
                String email = rs2.getString("email");
                System.out.println("\nSpecific calls:");
                System.out.println("rs2.getInt('age'): " + age);
                System.out.println("rs2.getString('email'): " + email);
                System.out.println("age == 30: " + (age == 30));
                System.out.println("email.equals('bob@example.com'): " + email.equals("bob@example.com"));
            }
            rs2.close();
            
            // Test 3: SELECT with different order
            System.out.println("\n=== Test 3: SELECT email, age ===");
            ResultSet rs3 = statement.executeQuery("SELECT email, age FROM test_column_order WHERE name = 'Bob'");
            ResultSetMetaData meta3 = rs3.getMetaData();
            System.out.println("Columns:");
            for (int i = 1; i <= meta3.getColumnCount(); i++) {
                System.out.println(i + ": " + meta3.getColumnName(i));
            }
            while (rs3.next()) {
                System.out.println("\nRow data:");
                for (int i = 1; i <= meta3.getColumnCount(); i++) {
                    System.out.println(meta3.getColumnName(i) + ": " + rs3.getString(i));
                }
                // Test with column names
                int age = rs3.getInt("age");
                String email = rs3.getString("email");
                System.out.println("\nUsing column names:");
                System.out.println("rs3.getInt('age'): " + age);
                System.out.println("rs3.getString('email'): " + email);
            }
            rs3.close();
            
            // Clean up
            statement.executeUpdate("DROP TABLE test_column_order");
            statement.close();
            connection.close();
            System.out.println("\nTest completed!");
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}