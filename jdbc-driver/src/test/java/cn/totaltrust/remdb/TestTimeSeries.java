package cn.totaltrust.remdb;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import java.sql.*;
import java.util.ArrayList;
import java.util.List;

public class TestTimeSeries extends TestBase {

    @Before
    public void setUp() throws SQLException {
        super.setUp();
        // 创建测试表
        createTestTimeSeriesTableCustom();
    }

    @After
    public void tearDown() throws SQLException {
        super.tearDown();
    }

    /**
     * 创建测试时序表
     * @throws SQLException 如果创建失败
     */
    private void createTestTimeSeriesTableCustom() throws SQLException {
        // 创建带有压缩和TTL的时序表
        executeSql("CREATE TIMESERIES TABLE IF NOT EXISTS test_timeseries_table (" +
                "timestamp TIMESTAMP, " +
                "value REAL, " +
                "sensor_id TEXT, " +
                "location TEXT " +
                ") WITH COMPRESSION = (algorithm='delta', enabled=true) " +
                ", WITH TTL = '7 days'");
    }

    /**
     * 测试创建时序表
     */
    @Test
    public void testCreateTimeSeriesTable() throws SQLException {
        // 验证时序表是否存在
        assert tableExists("test_timeseries_table");

        // 创建另一个时序表，使用不同的压缩算法
        executeSql("CREATE TIMESERIES TABLE IF NOT EXISTS test_timeseries_table2 (" +
                "timestamp TIMESTAMP, " +
                "value REAL, " +
                "sensor_id TEXT " +
                ") WITH COMPRESSION = (algorithm='delta-delta', enabled=true)");

        // 验证第二个时序表是否存在
        assert tableExists("test_timeseries_table2");

        // 删除测试表
        executeSql("DROP TABLE IF EXISTS test_timeseries_table2");
        assert !tableExists("test_timeseries_table2");
    }

    /**
     * 测试写入时序数据
     */
    @Test
    public void testWriteTimeSeriesData() throws SQLException {
        // 写入单条时序数据
        long now = System.currentTimeMillis();
        executeSql("INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" +
                now + ", 25.5, 'sensor_1', 'room_1')");

        // 验证数据是否写入成功
        verifyRowCount("SELECT * FROM test_timeseries_table", 1);

        // 批量写入时序数据
        List<String> insertSqls = new ArrayList<>();
        for (int i = 1; i <= 5; i++) {
            long timestamp = now + (i * 60000); // 每分钟一条数据
            double value = 25.0 + i * 0.1;
            insertSqls.add("INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" +
                    timestamp + ", " + value + ", 'sensor_1', 'room_1')");
        }
        executeBatch(insertSqls.toArray(new String[0]));

        // 验证批量写入是否成功
        verifyRowCount("SELECT * FROM test_timeseries_table", 6); // 1条单条 + 5条批量
    }

    /**
     * 测试查询时序数据
     */
    @Test
    public void testQueryTimeSeriesData() throws SQLException {
        // 写入测试数据
        long now = System.currentTimeMillis();
        executeBatch(new String[]{
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + (now - 3600000) + ", 24.5, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + (now - 1800000) + ", 25.0, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + now + ", 25.5, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + now + ", 26.0, 'sensor_2', 'room_2')"
        });

        // 测试基本查询
        ResultSet rs = executeQuery("SELECT * FROM test_timeseries_table WHERE sensor_id = 'sensor_1'");
        int count = 0;
        while (rs.next()) {
            String sensorId = rs.getString("sensor_id");
            assert sensorId.equals("sensor_1");
            count++;
        }
        rs.close();
        assert count == 3;

        // 测试时间范围查询
        rs = executeQuery("SELECT * FROM test_timeseries_table WHERE timestamp BETWEEN " + (now - 2000000) + " AND " + (now + 1000000));
        count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 3; // 最近的3条数据

        // 测试按时间排序查询
        rs = executeQuery("SELECT * FROM test_timeseries_table ORDER BY timestamp DESC");
        long previousTimestamp = Long.MAX_VALUE;
        count = 0;
        while (rs.next()) {
            long timestamp = rs.getLong("timestamp");
            assert timestamp <= previousTimestamp;
            previousTimestamp = timestamp;
            count++;
        }
        rs.close();
        assert count == 4;
    }

    /**
     * 测试时序数据聚合
     */
    @Test
    public void testTimeSeriesAggregation() throws SQLException {
        // 写入测试数据
        long now = System.currentTimeMillis();
        executeBatch(new String[]{
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + now + ", 25.5, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + (now + 60000) + ", 26.0, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + (now + 120000) + ", 25.8, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + now + ", 24.5, 'sensor_2', 'room_2')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + (now + 60000) + ", 24.8, 'sensor_2', 'room_2')"
        });

        // 测试按传感器ID聚合
        ResultSet rs = executeQuery("SELECT sensor_id, AVG(value) AS avg_value, MIN(value) AS min_value, MAX(value) AS max_value FROM test_timeseries_table GROUP BY sensor_id");
        int count = 0;
        while (rs.next()) {
            String sensorId = rs.getString("sensor_id");
            double avgValue = rs.getDouble("avg_value");
            if (sensorId.equals("sensor_1")) {
                assert Math.abs(avgValue - 25.7667) < 0.0001; // (25.5 + 26.0 + 25.8) / 3
            } else if (sensorId.equals("sensor_2")) {
                assert Math.abs(avgValue - 24.65) < 0.0001; // (24.5 + 24.8) / 2
            }
            count++;
        }
        rs.close();
        assert count == 2;

        // 测试使用TIME_BUCKET聚合
        rs = executeQuery("SELECT TIME_BUCKET('1h', timestamp) AS time_window, AVG(value) AS avg_value FROM test_timeseries_table GROUP BY time_window");
        count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 1; // 所有数据都在同一个小时窗口内
    }

    /**
     * 测试时序数据标签查询
     */
    @Test
    public void testTimeSeriesTagQuery() throws SQLException {
        // 写入测试数据，带有不同的标签
        long now = System.currentTimeMillis();
        executeBatch(new String[]{
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + now + ", 25.5, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + (now + 60000) + ", 26.0, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + now + ", 24.5, 'sensor_2', 'room_2')",
            "INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + (now + 60000) + ", 24.8, 'sensor_2', 'room_2')"
        });

        // 测试按location标签查询
        ResultSet rs = executeQuery("SELECT * FROM test_timeseries_table WHERE location = 'room_1'");
        int count = 0;
        while (rs.next()) {
            String location = rs.getString("location");
            assert location.equals("room_1");
            count++;
        }
        rs.close();
        assert count == 2;

        // 测试按多个标签组合查询
        rs = executeQuery("SELECT * FROM test_timeseries_table WHERE sensor_id = 'sensor_1' AND location = 'room_1'");
        count = 0;
        while (rs.next()) {
            String sensorId = rs.getString("sensor_id");
            String location = rs.getString("location");
            assert sensorId.equals("sensor_1");
            assert location.equals("room_1");
            count++;
        }
        rs.close();
        assert count == 2;
    }

    /**
     * 测试时序数据的批量写入和查询性能
     */
    @Test
    public void testTimeSeriesBatchPerformance() throws SQLException {
        // 批量写入100条数据
        long now = System.currentTimeMillis();
        List<String> insertSqls = new ArrayList<>();
        for (int i = 0; i < 100; i++) {
            long timestamp = now + (i * 1000); // 每秒一条数据
            double value = 25.0 + (i % 10) * 0.1; // 模拟波动值
            String sensorId = "sensor_" + (i % 5 + 1); // 5个不同的传感器
            String location = "room_" + (i % 3 + 1); // 3个不同的位置
            insertSqls.add("INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" +
                    timestamp + ", " + value + ", '" + sensorId + "', '" + location + "')");
        }

        // 执行批量插入
        long startTime = System.currentTimeMillis();
        executeBatch(insertSqls.toArray(new String[0]));
        long endTime = System.currentTimeMillis();

        // 验证批量插入是否成功
        verifyRowCount("SELECT * FROM test_timeseries_table", 100);

        // 验证插入性能（应该在合理范围内）
        assert (endTime - startTime) < 5000; // 5秒内完成

        // 测试批量查询性能
        startTime = System.currentTimeMillis();
        ResultSet rs = executeQuery("SELECT sensor_id, AVG(value) AS avg_value FROM test_timeseries_table GROUP BY sensor_id");
        int count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        endTime = System.currentTimeMillis();

        // 验证查询结果
        assert count == 5; // 5个传感器

        // 验证查询性能（应该在合理范围内）
        assert (endTime - startTime) < 2000; // 2秒内完成
    }
}
