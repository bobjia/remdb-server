import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.Statement;
import java.sql.ResultSet;
import java.sql.SQLException;

public class SimpleTest {
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
            
            // Execute a simple query
            System.out.println("Executing query: SELECT 1");
            ResultSet resultSet = statement.executeQuery("SELECT 1");
            
            // Process the result
            if (resultSet.next()) {
                int result = resultSet.getInt(1);
                System.out.println("Query executed successfully, result: " + result);
            }
            
            // Close resources
            resultSet.close();
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