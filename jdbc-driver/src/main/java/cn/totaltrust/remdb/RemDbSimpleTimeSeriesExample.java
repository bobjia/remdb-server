package cn.totaltrust.remdb;

import java.sql.*;
import java.util.Calendar;

public class RemDbSimpleTimeSeriesExample {
    public static void main(String[] args) {
        String url = "jdbc:remdb://localhost:6666";
        String user = "root";
        String password = "admin";

        Connection conn = null;
        Statement stmt = null;
        ResultSet rs = null;

        try {
            System.out.println("Connecting to RemDb server...");
            // 显式加载RemDb驱动
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            conn = DriverManager.getConnection(url, user, password);
            System.out.println("Connection established successfully!");

            stmt = conn.createStatement();

            // 1. 创建普通表（支持时序数据存储）
            System.out.println("\n1. Creating table...");
            String createTableSQL = "CREATE TABLE IF NOT EXISTS sensor_data " +
                                   "(id INT PRIMARY KEY, timestamp TIMESTAMP, value DOUBLE, sensor_id VARCHAR(50))";
            stmt.executeUpdate(createTableSQL);
            System.out.println("Table 'sensor_data' created successfully!");

            // 2. 插入单条数据
            System.out.println("\n2. Inserting single data...");
            long currentTime = System.currentTimeMillis();
            String insertSQL = String.format("INSERT INTO sensor_data (id, timestamp, value, sensor_id) " +
                                           "VALUES (1, %d, 23.5, 'sensor_001')", currentTime);
            stmt.executeUpdate(insertSQL);
            System.out.println("Inserted single record successfully!");

            // 3. 查询数据
            System.out.println("\n3. Querying data...");
            String selectSQL = "SELECT id, timestamp, value, sensor_id FROM sensor_data";
            rs = stmt.executeQuery(selectSQL);

            System.out.println("ID | Timestamp | Value | Sensor ID");
            System.out.println("---|------------|-------|-----------");
            while (rs.next()) {
                int id = rs.getInt("id");
                Timestamp ts = rs.getTimestamp("timestamp");
                double value = rs.getDouble("value");
                String sensorId = rs.getString("sensor_id");
                System.out.printf("%d  | %d        | %.1f   | %s\n", id, ts.getTime(), value, sensorId);
            }
            rs.close();

            // 4. 插入多条数据（模拟时序数据）
            System.out.println("\n4. Inserting multiple time series data...");
            for (int i = 2; i <= 5; i++) {
                long ts = currentTime + (i * 1000); // 每秒一条记录
                double value = 20.0 + Math.random() * 10.0;
                String insertMultipleSQL = String.format("INSERT INTO sensor_data (id, timestamp, value, sensor_id) " +
                                                      "VALUES (%d, %d, %.1f, 'sensor_001')", i, ts, value);
                stmt.executeUpdate(insertMultipleSQL);
            }
            System.out.println("Inserted 4 more records successfully!");

            // 5. 查询所有数据（按时间排序）
            System.out.println("\n5. Querying all data ordered by timestamp...");
            String selectAllSQL = "SELECT id, timestamp, value, sensor_id FROM sensor_data ORDER BY timestamp";
            rs = stmt.executeQuery(selectAllSQL);

            System.out.println("ID | Timestamp | Value | Sensor ID");
            System.out.println("---|------------|-------|-----------");
            while (rs.next()) {
                int id = rs.getInt("id");
                Timestamp ts = rs.getTimestamp("timestamp");
                double value = rs.getDouble("value");
                String sensorId = rs.getString("sensor_id");
                System.out.printf("%d  | %d        | %.1f   | %s\n", id, ts.getTime(), value, sensorId);
            }
            rs.close();

            // 6. 使用时间范围查询
            System.out.println("\n6. Querying data with time range...");
            long startTime = currentTime;
            long endTime = currentTime + 3000; // 3秒范围
            String timeRangeSQL = String.format("SELECT id, timestamp, value, sensor_id FROM sensor_data " +
                                              "WHERE timestamp BETWEEN %d AND %d ORDER BY timestamp", startTime, endTime);
            rs = stmt.executeQuery(timeRangeSQL);

            System.out.println("ID | Timestamp | Value | Sensor ID");
            System.out.println("---|------------|-------|-----------");
            while (rs.next()) {
                int id = rs.getInt("id");
                Timestamp ts = rs.getTimestamp("timestamp");
                double value = rs.getDouble("value");
                String sensorId = rs.getString("sensor_id");
                System.out.printf("%d  | %d        | %.1f   | %s\n", id, ts.getTime(), value, sensorId);
            }
            rs.close();

            System.out.println("\nAll operations completed successfully!");

        } catch (ClassNotFoundException e) {
            System.err.println("Driver not found: " + e.getMessage());
        } catch (SQLException e) {
            System.err.println("SQL error: " + e.getMessage());
            e.printStackTrace();
        } finally {
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