import java.sql.*;

public class SimpleJdbcTest {
    public static void main(String[] args) {
        String jdbcUrl = "jdbc:remdb://localhost:6666/default";
        
        try {
            // Load the JDBC driver
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            System.out.println("Driver loaded successfully");
            
            // Establish a connection
            Connection connection = DriverManager.getConnection(jdbcUrl);
            System.out.println("Connection established successfully");
            
            // Create a statement
            Statement statement = connection.createStatement();
            
            // Execute a simple SQL statement
            ResultSet resultSet = statement.executeQuery("SELECT 1");
            
            // Process the result
            if (resultSet.next()) {
                int result = resultSet.getInt(1);
                System.out.println("Query executed successfully, result: " + result);
            }
            
            // Clean up
            resultSet.close();
            statement.close();
            connection.close();
            System.out.println("Connection closed successfully");
            
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}