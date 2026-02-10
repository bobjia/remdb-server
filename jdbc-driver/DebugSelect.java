import java.sql.*;

public class DebugSelect {
    public static void main(String[] args) {
        String jdbcUrl = "jdbc:remdb://localhost:6666/test_db";
        
        try {
            // Register driver
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            
            // Connect to database
            System.out.println("Connecting to database...");
            Connection connection = DriverManager.getConnection(jdbcUrl);
            Statement statement = connection.createStatement();
            
            // Drop table if it exists
            try {
                statement.executeUpdate("DROP TABLE IF EXISTS test_alter_add");
            } catch (Exception e) {
                // Ignore errors
            }
            
            // Create table
            System.out.println("Creating table...");
            statement.executeUpdate("CREATE TABLE IF NOT EXISTS test_alter_add (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL)");
            
            // Add new columns
            System.out.println("Adding columns...");
            statement.executeUpdate("ALTER TABLE test_alter_add ADD COLUMN age INTEGER");
            statement.executeUpdate("ALTER TABLE test_alter_add ADD COLUMN email TEXT");
            
            // Insert data
            System.out.println("Inserting data...");
            statement.executeUpdate("INSERT INTO test_alter_add (name, age, email) VALUES ('Bob', 30, 'bob@example.com')");
            
            // Debug 1: Select all columns
            System.out.println("\nDebug 1: Select all columns");
            ResultSet rs1 = statement.executeQuery("SELECT * FROM test_alter_add");
            ResultSetMetaData meta1 = rs1.getMetaData();
            int columnCount1 = meta1.getColumnCount();
            System.out.println("Column count: " + columnCount1);
            for (int i = 1; i <= columnCount1; i++) {
                System.out.println("Column " + i + ": " + meta1.getColumnName(i));
            }
            while (rs1.next()) {
                for (int i = 1; i <= columnCount1; i++) {
                    System.out.println(meta1.getColumnName(i) + ": " + rs1.getString(i));
                }
            }
            rs1.close();
            
            // Debug 2: Select specific columns
            System.out.println("\nDebug 2: Select specific columns");
            ResultSet rs2 = statement.executeQuery("SELECT age, email FROM test_alter_add WHERE name = 'Bob'");
            ResultSetMetaData meta2 = rs2.getMetaData();
            int columnCount2 = meta2.getColumnCount();
            System.out.println("Column count: " + columnCount2);
            for (int i = 1; i <= columnCount2; i++) {
                System.out.println("Column " + i + ": " + meta2.getColumnName(i));
            }
            while (rs2.next()) {
                int age = rs2.getInt("age");
                String email = rs2.getString("email");
                System.out.println("age: " + age);
                System.out.println("email: " + email);
                System.out.println("email is null: " + (email == null));
                System.out.println("email equals 'bob@example.com': " + "bob@example.com".equals(email));
                System.out.println("email length: " + (email != null ? email.length() : 0));
                System.out.println("email chars:");
                if (email != null) {
                    for (int i = 0; i < email.length(); i++) {
                        char c = email.charAt(i);
                        System.out.println("  Index " + i + ": '" + c + "' (ASCII: " + (int)c + ")");
                    }
                }
            }
            rs2.close();
            
            // Clean up
            statement.close();
            connection.close();
            
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}