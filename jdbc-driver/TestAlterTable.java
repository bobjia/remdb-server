import java.sql.*;

public class TestAlterTable {
    public static void main(String[] args) {
        String jdbcUrl = "jdbc:remdb://localhost:6666/test_db";
        
        try {
            // Register driver
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            
            // Start server in a new process
            Process serverProcess = Runtime.getRuntime().exec("cargo run --manifest-path=../Cargo.toml");
            
            // Give server time to start
            Thread.sleep(2000);
            
            // Connect to database
            System.out.println("Connecting to database...");
            Connection connection = DriverManager.getConnection(jdbcUrl);
            Statement statement = connection.createStatement();
            
            // Drop table if it exists
            try {
                statement.executeUpdate("DROP TABLE IF EXISTS employees");
            } catch (Exception e) {
                // Ignore errors
            }
            
            // Create table
            System.out.println("Creating table...");
            statement.executeUpdate("CREATE TABLE employees (id INT PRIMARY KEY, name VARCHAR(50))");
            
            // Insert data
            System.out.println("Inserting data...");
            statement.executeUpdate("INSERT INTO employees (id, name) VALUES (1, 'Bob')");
            
            // Add column
            System.out.println("Adding email column...");
            statement.executeUpdate("ALTER TABLE employees ADD COLUMN email VARCHAR(100)");
            
            // Update data
            System.out.println("Updating email...");
            statement.executeUpdate("UPDATE employees SET email = 'bob@example.com' WHERE id = 1");
            
            // Query data
            System.out.println("Querying data...");
            ResultSet resultSet = statement.executeQuery("SELECT id, name, email FROM employees");
            
            while (resultSet.next()) {
                int id = resultSet.getInt("id");
                String name = resultSet.getString("name");
                String email = resultSet.getString("email");
                
                System.out.println("ID: " + id);
                System.out.println("Name: " + name);
                System.out.println("Email: " + email);
                System.out.println("Email is null: " + (email == null));
                System.out.println("Email equals 'bob@example.com': " + "bob@example.com".equals(email));
            }
            
            // Clean up
            resultSet.close();
            statement.close();
            connection.close();
            
            // Stop server
            serverProcess.destroy();
            
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}