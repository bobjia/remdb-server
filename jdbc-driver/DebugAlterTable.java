import java.sql.*;

public class DebugAlterTable {
    public static void main(String[] args) {
        String jdbcUrl = "jdbc:remdb://localhost:6666/default";
        try {
            // Load the JDBC driver
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            
            // Establish a connection
            System.out.println("Connecting to RemDB server...");
            Connection connection = DriverManager.getConnection(jdbcUrl);
            System.out.println("Connection established successfully!");
            
            // Create a statement
            Statement statement = connection.createStatement();
            
            // Clean up any existing test table
            try {
                statement.executeUpdate("DROP TABLE IF EXISTS test_alter_add");
                System.out.println("Dropped existing test table if it existed");
            } catch (SQLException e) {
                System.out.println("Error dropping table: " + e.getMessage());
            }
            
            // Create test table
            System.out.println("Creating test table...");
            statement.executeUpdate("CREATE TABLE IF NOT EXISTS test_alter_add (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL)");
            System.out.println("Test table created successfully");
            
            // Add new column
            System.out.println("Adding new column 'age'...");
            statement.executeUpdate("ALTER TABLE test_alter_add ADD COLUMN age INTEGER");
            System.out.println("Column 'age' added successfully");
            
            // Add another column
            System.out.println("Adding new column 'email'...");
            statement.executeUpdate("ALTER TABLE test_alter_add ADD COLUMN email TEXT");
            System.out.println("Column 'email' added successfully");
            
            // Insert test data
            System.out.println("Inserting test data...");
            statement.executeUpdate("INSERT INTO test_alter_add (name, age, email) VALUES ('Bob', 30, 'bob@example.com')");
            System.out.println("Test data inserted successfully");
            
            // Query the data
            System.out.println("Querying test data...");
            ResultSet rs = statement.executeQuery("SELECT id, name, age, email FROM test_alter_add");
            
            while (rs.next()) {
                int id = rs.getInt("id");
                String name = rs.getString("name");
                int age = rs.getInt("age");
                String email = rs.getString("email");
                System.out.println("Row: id=" + id + ", name=" + name + ", age=" + age + ", email=" + email);
            }
            
            // Close resources
            rs.close();
            statement.close();
            connection.close();
            
            System.out.println("Test completed successfully!");
            
        } catch (ClassNotFoundException e) {
            System.out.println("Error loading JDBC driver: " + e.getMessage());
            e.printStackTrace();
        } catch (SQLException e) {
            System.out.println("SQL error: " + e.getMessage());
            e.printStackTrace();
        } catch (Exception e) {
            System.out.println("Error: " + e.getMessage());
            e.printStackTrace();
        }
    }
}