package cn.totaltrust.remdb;

import java.sql.*;
import java.util.ArrayList;
import java.util.List;

public class RemDbTimeSeriesExample {
    public static void main(String[] args) throws ClassNotFoundException {
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

            // 1. 使用SQL语句创建时序表
            System.out.println("\n1. Creating time series table using SQL...");
            String createTimeSeriesSQL = "CREATE TABLE IF NOT EXISTS sensor_data " +
                                         "(timestamp TIMESTAMP, value FLOAT64, sensor_id TEXT, location TEXT)";
            stmt.executeUpdate(createTimeSeriesSQL);
            System.out.println("Table 'sensor_data' created or already exists");

            // 2. 插入单条数据
            System.out.println("\n2. Inserting single data...");
            long currentTime = System.currentTimeMillis();
            String insertSQL = String.format("INSERT INTO sensor_data (timestamp, value, sensor_id, location) " +
                                           "VALUES (%d, 23.5, 'sensor_001', 'room_101')", currentTime);
            int rowsInserted = stmt.executeUpdate(insertSQL);
            System.out.println("Inserted " + rowsInserted + " row(s)");

            // 4. 查询时序数据
            System.out.println("\n4. Querying time series data...");
            String selectSQL = "SELECT timestamp, value, sensor_id, location FROM sensor_data ORDER BY timestamp DESC";
            rs = stmt.executeQuery(selectSQL);

            System.out.println("Timestamp | Value | Sensor ID | Location");
            System.out.println("----------|-------|-----------|----------");
            while (rs.next()) {
                Timestamp ts = rs.getTimestamp("timestamp");
                double value = rs.getDouble("value");
                String sensorId = rs.getString("sensor_id");
                String location = rs.getString("location");
                System.out.printf("%d | %.1f   | %s      | %s\n", ts.getTime(), value, sensorId, location);
            }

            // 5. 批量写入数据
            System.out.println("\n5. Batch writing data...");
            StringBuilder batchInsertSql = new StringBuilder();
            batchInsertSql.append("INSERT IGNORE INTO sensor_data (timestamp, value, sensor_id, location) VALUES ");
            
            for (int i = 0; i < 10; i++) {
                long ts = currentTime + (i * 1000); // 每秒一条记录
                double value = 23.0 + Math.random() * 5.0;
                String sensorId = "sensor_001";
                String location = "room_101";
                
                if (i > 0) {
                    batchInsertSql.append(",");
                }
                batchInsertSql.append(String.format("(%d, %.1f, 'sensor_001', 'room_101')", ts, value));
            }
            
            int batchRowsInserted = stmt.executeUpdate(batchInsertSql.toString());
            System.out.println("Batch inserted " + batchRowsInserted + " row(s)");

            // 6. 使用时间范围查询
            System.out.println("\n6. Querying data with time range...");
            long startTime = currentTime;
            long endTime = currentTime + 10000; // 10秒范围
            String timeRangeSql = String.format("SELECT timestamp, value, sensor_id, location FROM sensor_data WHERE timestamp BETWEEN %d AND %d ORDER BY timestamp", 
                                              startTime, endTime);
            rs = stmt.executeQuery(timeRangeSql);

            System.out.println("Timestamp | Value | Sensor ID | Location");
            System.out.println("----------|-------|-----------|----------");
            int count = 0;
            while (rs.next()) {
                Timestamp ts = rs.getTimestamp("timestamp");
                double value = rs.getDouble("value");
                String sensorId = rs.getString("sensor_id");
                String location = rs.getString("location");
                System.out.printf("%d | %.1f   | %s      | %s\n", ts.getTime(), value, sensorId, location);
                count++;
            }
            System.out.println("Found " + count + " records in the time range");

            // 7. 使用标签查询
            System.out.println("\n7. Querying data by tag...");
            String tagQuerySql = "SELECT timestamp, value, sensor_id, location FROM sensor_data WHERE sensor_id = 'sensor_001' ORDER BY timestamp DESC LIMIT 5";
            rs = stmt.executeQuery(tagQuerySql);

            System.out.println("Timestamp | Value | Sensor ID | Location");
            System.out.println("----------|-------|-----------|----------");
            count = 0;
            while (rs.next()) {
                Timestamp ts = rs.getTimestamp("timestamp");
                double value = rs.getDouble("value");
                String sensorId = rs.getString("sensor_id");
                String location = rs.getString("location");
                System.out.printf("%d | %.1f   | %s      | %s\n", ts.getTime(), value, sensorId, location);
                count++;
            }
            System.out.println("Found " + count + " records with tag sensor_id=sensor_001");

        } catch (SQLException e) {
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