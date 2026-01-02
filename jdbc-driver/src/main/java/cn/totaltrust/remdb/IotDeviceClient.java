package cn.totaltrust.remdb;

import java.sql.*;
import java.util.Random;

public class IotDeviceClient {
    private static final String URL = "jdbc:remdb://localhost:6666";
    private static final String USER = "admin";
    private static final String PASSWORD = "admin";
    
    private Connection conn;
    private Statement stmt;
    
    public void initialize() throws SQLException {
        try {
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
        } catch (ClassNotFoundException e) {
            throw new SQLException("RemDbDriver not found", e);
        }
        System.out.println("Connecting to RemDb server...");
        conn = DriverManager.getConnection(URL, USER, PASSWORD);
        stmt = conn.createStatement();
        System.out.println("Connection established successfully!");
    }
    
    public void createIotTable() throws SQLException {
        String createTableSQL = "CREATE TABLE iot_devices " +
                               "(id INT AUTOINCREMENT PRIMARY KEY, " +
                               " device_id VARCHAR(50), " +
                               " created_at BIGINT, " +
                               " temperature DOUBLE, " +
                               " humidity DOUBLE, " +
                               " pressure DOUBLE, " +
                               " battery_level INT)";
        stmt.executeUpdate(createTableSQL);
        System.out.println("IoT table created or already exists");
    }
    
    public void insertIotData(String deviceId, long timestamp, double temperature, 
                             double humidity, double pressure, int batteryLevel) throws SQLException {
        String insertSQL = String.format("INSERT INTO iot_devices (device_id, created_at, temperature, humidity, pressure, battery_level) " +
                                       "VALUES (\"%s\", %d, %.2f, %.2f, %.2f, %d)",
                                       deviceId, timestamp, temperature, humidity, pressure, batteryLevel);
        int rowsInserted = stmt.executeUpdate(insertSQL);
        System.out.printf("Inserted %d row(s): Device=%s, Temp=%.2f°C, Humidity=%.2f%%, Pressure=%.2f hPa, Battery=%d%%\n",
                         rowsInserted, deviceId, temperature, humidity, pressure, batteryLevel);
    }
    
    public void queryIotData(int limit) throws SQLException {
        String selectSQL = String.format("SELECT * FROM iot_devices ORDER BY created_at DESC LIMIT %d", limit);
        ResultSet rs = stmt.executeQuery(selectSQL);
        
        System.out.println("\nQuery results (latest " + limit + " records):");
        System.out.println("ID | Device ID | created_at | Temperature | Humidity | Pressure | Battery");
        System.out.println("---|-----------|-----------|-------------|----------|----------|--------");
        
        while (rs.next()) {
            int id = rs.getInt("id");
            String deviceId = rs.getString("device_id");
            long timestamp = rs.getLong("created_at");
            double temperature = rs.getDouble("temperature");
            double humidity = rs.getDouble("humidity");
            double pressure = rs.getDouble("pressure");
            int batteryLevel = rs.getInt("battery_level");
            
            System.out.printf("%d  | %s | %d | %.2f°C       | %.2f%%    | %.2f hPa  | %d%%\n",
                             id, deviceId, timestamp, temperature, humidity, pressure, batteryLevel);
        }
        rs.close();
    }
    
    public void updateIotData(int id, double temperature) throws SQLException {
        String updateSQL = String.format("UPDATE iot_devices SET temperature = %.2f WHERE id = %d", temperature, id);
        int rowsUpdated = stmt.executeUpdate(updateSQL);
        System.out.printf("Updated %d row(s) for ID %d\n", rowsUpdated, id);
    }
    
    public void deleteIotData(int id) throws SQLException {
        String deleteSQL = String.format("DELETE FROM iot_devices WHERE id = %d", id);
        int rowsDeleted = stmt.executeUpdate(deleteSQL);
        System.out.printf("Deleted %d row(s) for ID %d\n", rowsDeleted, id);
    }
    
    public void continuousWrite(String deviceId, long intervalMs, int durationSeconds) throws SQLException, InterruptedException {
        System.out.printf("\nStarting continuous data write for device %s, interval: %d ms, duration: %d seconds\n",
                         deviceId, intervalMs, durationSeconds);
        
        Random random = new Random();
        long endTime = System.currentTimeMillis() + (durationSeconds * 1000);
        int count = 0;
        
        while (System.currentTimeMillis() < endTime) {
            long timestamp = System.currentTimeMillis();
            double temperature = 20.0 + (random.nextDouble() * 10.0);
            double humidity = 40.0 + (random.nextDouble() * 40.0);
            double pressure = 980.0 + (random.nextDouble() * 40.0);
            int batteryLevel = 70 + random.nextInt(30);
            
            insertIotData(deviceId, timestamp, temperature, humidity, pressure, batteryLevel);
            count++;
            
            Thread.sleep(intervalMs);
        }
        
        System.out.printf("\nContinuous write completed. Total records written: %d\n", count);
    }
    
    public void close() throws SQLException {
        if (stmt != null) stmt.close();
        if (conn != null) conn.close();
        System.out.println("Resources closed successfully!");
    }
    
    public static void main(String[] args) {
        IotDeviceClient client = new IotDeviceClient();
        
        try {
            client.initialize();
            
            client.createIotTable();
            
            System.out.println("\n2. Inserting single record...");
            client.insertIotData("device-001", System.currentTimeMillis(), 25.5, 60.2, 1005.3, 85);
            
            System.out.println("\n3. Querying data...");
            client.queryIotData(5);
            
            System.out.println("\n4. Updating record...");
            client.updateIotData(1, 26.8);
            
            System.out.println("\n5. Querying updated data...");
            client.queryIotData(5);
            
            System.out.println("\n6. Continuous write test (1 record/second for 5 seconds)...");
            client.continuousWrite("device-001", 1000, 5);
            
            System.out.println("\n7. High frequency write test (1 record/10ms for 2 seconds)...");
            client.continuousWrite("device-001", 10, 2);
            
            System.out.println("\n8. Final query (latest 10 records)...");
            client.queryIotData(10);
            
        } catch (SQLException e) {
            e.printStackTrace();
        } catch (InterruptedException e) {
            e.printStackTrace();
            Thread.currentThread().interrupt();
        } finally {
            try {
                client.close();
            } catch (SQLException e) {
                e.printStackTrace();
            }
        }
    }
}