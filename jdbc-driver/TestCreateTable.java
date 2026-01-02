import java.sql.*;

public class TestCreateTable {
    public static void main(String[] args) {
        String url = "jdbc:remdb://localhost:6666";
        String user = "";
        String password = "";

        try {
            // Load the RemDb driver explicitly
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            
            System.out.println("RemDb driver loaded successfully");
            
            try (Connection conn = DriverManager.getConnection(url, user, password);
                 Statement stmt = conn.createStatement()) {

                System.out.println("Connected to RemDb server");
                
                // Test CREATE TABLE with AUTOINCREMENT
                String createTableSQL = "CREATE TABLE iot_devices (id INT AUTOINCREMENT PRIMARY KEY,  device_id VARCHAR(50),  timestamp BIGINT,  temperature DOUBLE,  humidity DOUBLE,  pressure DOUBLE,  battery_level INT)";
                System.out.println("Executing: " + createTableSQL);
                
                long startTime = System.currentTimeMillis();
                stmt.executeUpdate(createTableSQL);
                long endTime = System.currentTimeMillis();
                
                System.out.println("CREATE TABLE executed successfully in " + (endTime - startTime) + " ms");
                
            }
        } catch (ClassNotFoundException e) {
            System.err.println("Could not load RemDb driver: " + e.getMessage());
            e.printStackTrace();
        } catch (SQLException e) {
            e.printStackTrace();
        }
    }
}