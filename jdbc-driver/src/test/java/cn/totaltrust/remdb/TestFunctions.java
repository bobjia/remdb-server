package cn.totaltrust.remdb;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import java.sql.*;

public class TestFunctions extends TestBase {

    @Before
    public void setUp() throws SQLException {
        super.setUp();
        // 创建测试表
        createTestTables();
        // 插入测试数据
        insertTestData();
    }

    @After
    public void tearDown() throws SQLException {
        super.tearDown();
    }

    /**
     * 创建测试表
     * @throws SQLException 如果创建失败
     */
    private void createTestTables() throws SQLException {
        // 创建用户表
        executeSql("CREATE TABLE IF NOT EXISTS test_functions_users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, email TEXT)");
        // 创建传感器数据表
        executeSql("CREATE TABLE IF NOT EXISTS test_functions_sensor (id INTEGER PRIMARY KEY AUTOINCREMENT, sensor_id INTEGER, temperature REAL, humidity REAL, timestamp TIMESTAMP)");
    }

    /**
     * 插入测试数据
     * @throws SQLException 如果插入失败
     */
    private void insertTestData() throws SQLException {
        // 插入用户数据
        executeBatch(new String[]{
            "INSERT INTO test_functions_users (name, age, email) VALUES ('Alice', 25, 'alice@example.com')",
            "INSERT INTO test_functions_users (name, age, email) VALUES ('Bob', 30, 'bob@example.com')",
            "INSERT INTO test_functions_users (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')",
            "INSERT INTO test_functions_users (name, age, email) VALUES ('David', 20, 'david@example.com')",
            "INSERT INTO test_functions_users (name, age, email) VALUES ('Eve', 40, 'eve@example.com')"
        });

        // 插入传感器数据
        long now = System.currentTimeMillis();
        executeBatch(new String[]{
            "INSERT INTO test_functions_sensor (sensor_id, temperature, humidity, timestamp) VALUES (1, 25.5, 60.0, " + now + ")",
            "INSERT INTO test_functions_sensor (sensor_id, temperature, humidity, timestamp) VALUES (1, 26.0, 59.5, " + (now + 3600000) + ")",
            "INSERT INTO test_functions_sensor (sensor_id, temperature, humidity, timestamp) VALUES (1, 25.8, 59.8, " + (now + 7200000) + ")",
            "INSERT INTO test_functions_sensor (sensor_id, temperature, humidity, timestamp) VALUES (2, 24.5, 62.0, " + now + ")",
            "INSERT INTO test_functions_sensor (sensor_id, temperature, humidity, timestamp) VALUES (2, 24.8, 61.5, " + (now + 3600000) + ")"
        });
    }

    /**
     * 测试聚合函数 - COUNT
     */
    @Test
    public void testCountFunction() throws SQLException {
        // 测试 COUNT(*)
        ResultSet rs = executeQuery("SELECT COUNT(*) AS total_users FROM test_functions_users");
        while (rs.next()) {
            int count = rs.getInt("total_users");
            assert count == 5;
        }
        rs.close();

        // 测试 COUNT 特定列
        rs = executeQuery("SELECT COUNT(name) AS name_count FROM test_functions_users");
        while (rs.next()) {
            int count = rs.getInt("name_count");
            assert count == 5;
        }
        rs.close();

        // 测试 COUNT 与 WHERE 条件
        rs = executeQuery("SELECT COUNT(*) AS adult_count FROM test_functions_users WHERE age >= 18");
        while (rs.next()) {
            int count = rs.getInt("adult_count");
            assert count == 5;
        }
        rs.close();
    }

    /**
     * 测试聚合函数 - SUM
     */
    @Test
    public void testSumFunction() throws SQLException {
        // 测试 SUM
        ResultSet rs = executeQuery("SELECT SUM(age) AS total_age FROM test_functions_users");
        while (rs.next()) {
            int sum = rs.getInt("total_age");
            assert sum == 150; // 25 + 30 + 35 + 20 + 40
        }
        rs.close();

        // 测试 SUM 与 WHERE 条件
        rs = executeQuery("SELECT SUM(age) AS adult_age FROM test_functions_users WHERE age >= 25");
        while (rs.next()) {
            int sum = rs.getInt("adult_age");
            assert sum == 130; // 25 + 30 + 35 + 40
        }
        rs.close();

        // 测试 SUM 浮点数
        rs = executeQuery("SELECT SUM(temperature) AS total_temp FROM test_functions_sensor");
        while (rs.next()) {
            double sum = rs.getDouble("total_temp");
            assert sum == 126.6; // 25.5 + 26.0 + 25.8 + 24.5 + 24.8
        }
        rs.close();
    }

    /**
     * 测试聚合函数 - AVG
     */
    @Test
    public void testAvgFunction() throws SQLException {
        // 测试 AVG
        ResultSet rs = executeQuery("SELECT AVG(age) AS avg_age FROM test_functions_users");
        while (rs.next()) {
            double avg = rs.getDouble("avg_age");
            assert avg == 30.0; // 150 / 5
        }
        rs.close();

        // 测试 AVG 浮点数
        rs = executeQuery("SELECT AVG(temperature) AS avg_temp FROM test_functions_sensor");
        while (rs.next()) {
            double avg = rs.getDouble("avg_temp");
            assert Math.abs(avg - 25.32) < 0.01; // 126.6 / 5
        }
        rs.close();
    }

    /**
     * 测试聚合函数 - MIN 和 MAX
     */
    @Test
    public void testMinMaxFunctions() throws SQLException {
        // 测试 MIN 和 MAX
        ResultSet rs = executeQuery("SELECT MIN(age) AS min_age, MAX(age) AS max_age FROM test_functions_users");
        while (rs.next()) {
            int minAge = rs.getInt("min_age");
            int maxAge = rs.getInt("max_age");
            assert minAge == 20;
            assert maxAge == 40;
        }
        rs.close();

        // 测试 MIN 和 MAX 浮点数
        rs = executeQuery("SELECT MIN(temperature) AS min_temp, MAX(temperature) AS max_temp FROM test_functions_sensor");
        while (rs.next()) {
            double minTemp = rs.getDouble("min_temp");
            double maxTemp = rs.getDouble("max_temp");
            assert minTemp == 24.5;
            assert maxTemp == 26.0;
        }
        rs.close();
    }

    /**
     * 测试字符串函数
     */
    @Test
    public void testStringFunctions() throws SQLException {
        // 测试 CONCAT 函数
        ResultSet rs = executeQuery("SELECT CONCAT(name, ' ', email) AS full_info FROM test_functions_users WHERE id = 1");
        while (rs.next()) {
            String fullInfo = rs.getString("full_info");
            assert fullInfo.equals("Alice alice@example.com");
        }
        rs.close();

        // 测试 UPPER 函数
        rs = executeQuery("SELECT UPPER(name) AS upper_name FROM test_functions_users WHERE id = 2");
        while (rs.next()) {
            String upperName = rs.getString("upper_name");
            assert upperName.equals("BOB");
        }
        rs.close();

        // 测试 LOWER 函数
        rs = executeQuery("SELECT LOWER(email) AS lower_email FROM test_functions_users WHERE id = 3");
        while (rs.next()) {
            String lowerEmail = rs.getString("lower_email");
            assert lowerEmail.equals("charlie@example.com");
        }
        rs.close();

        // 测试 SUBSTRING 函数
        rs = executeQuery("SELECT SUBSTRING(name, 1, 2) AS name_prefix FROM test_functions_users WHERE id = 4");
        while (rs.next()) {
            String namePrefix = rs.getString("name_prefix");
            assert namePrefix.equals("Da");
        }
        rs.close();
    }

    /**
     * 测试数学函数
     */
    @Test
    public void testMathFunctions() throws SQLException {
        // 测试 ABS 函数
        ResultSet rs = executeQuery("SELECT ABS(-10) AS abs_value");
        while (rs.next()) {
            int absValue = rs.getInt("abs_value");
            assert absValue == 10;
        }
        rs.close();

        // 测试 SQRT 函数
        rs = executeQuery("SELECT SQRT(16) AS sqrt_value");
        while (rs.next()) {
            double sqrtValue = rs.getDouble("sqrt_value");
            assert sqrtValue == 4.0;
        }
        rs.close();

        // 测试 POWER 函数
        rs = executeQuery("SELECT POWER(2, 3) AS power_value");
        while (rs.next()) {
            double powerValue = rs.getDouble("power_value");
            assert powerValue == 8.0;
        }
        rs.close();

        // 测试 ROUND 函数
        rs = executeQuery("SELECT ROUND(3.14159, 2) AS round_value");
        while (rs.next()) {
            double roundValue = rs.getDouble("round_value");
            assert roundValue == 3.14;
        }
        rs.close();

        // 测试 CEIL 函数
        rs = executeQuery("SELECT CEIL(3.14) AS ceil_value");
        while (rs.next()) {
            double ceilValue = rs.getDouble("ceil_value");
            assert ceilValue == 4.0;
        }
        rs.close();

        // 测试 FLOOR 函数
        rs = executeQuery("SELECT FLOOR(3.99) AS floor_value");
        while (rs.next()) {
            double floorValue = rs.getDouble("floor_value");
            assert floorValue == 3.0;
        }
        rs.close();

        // 测试 MOD 函数
        rs = executeQuery("SELECT MOD(10, 3) AS mod_value");
        while (rs.next()) {
            int modValue = rs.getInt("mod_value");
            assert modValue == 1;
        }
        rs.close();
    }

    /**
     * 测试时间函数
     */
    @Test
    public void testTimeFunctions() throws SQLException {
        // 测试 TIME_BUCKET 函数
        ResultSet rs = executeQuery("SELECT TIME_BUCKET('1h', timestamp) AS time_window FROM test_functions_sensor GROUP BY time_window");
        int count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 3; // 3个不同的时间窗口

        // 测试 TO_ISO8601 函数
        rs = executeQuery("SELECT TO_ISO8601(timestamp) AS iso_time FROM test_functions_sensor LIMIT 1");
        while (rs.next()) {
            String isoTime = rs.getString("iso_time");
            assert isoTime != null;
            assert isoTime.length() > 0;
        }
        rs.close();

        // 测试 TO_CHAR 函数
        rs = executeQuery("SELECT TO_CHAR(timestamp, 'YYYY-MM-DD') AS date_str FROM test_functions_sensor LIMIT 1");
        while (rs.next()) {
            String dateStr = rs.getString("date_str");
            assert dateStr != null;
            assert dateStr.length() == 10; // YYYY-MM-DD 格式
        }
        rs.close();

        // 测试 TO_EPOCH 函数
        rs = executeQuery("SELECT TO_EPOCH(timestamp) AS epoch_sec FROM test_functions_sensor LIMIT 1");
        while (rs.next()) {
            double epochSec = rs.getDouble("epoch_sec");
            assert epochSec > 0;
        }
        rs.close();
    }

    /**
     * 测试滑动窗口函数
     */
    @Test
    public void testMovingWindowFunctions() throws SQLException {
        // 测试 MOVING_SUM 函数
        ResultSet rs = executeQuery("SELECT MOVING_SUM(temperature, 3) AS moving_sum FROM test_functions_sensor WHERE sensor_id = 1");
        int count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 3; // 3条传感器1的数据

        // 测试 MOVING_AVERAGE 函数
        rs = executeQuery("SELECT MOVING_AVERAGE(temperature, 3) AS moving_avg FROM test_functions_sensor WHERE sensor_id = 1");
        count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 3; // 3条传感器1的数据
    }

    /**
     * 测试函数组合
     */
    @Test
    public void testFunctionCombination() throws SQLException {
        // 测试函数组合
        ResultSet rs = executeQuery("SELECT UPPER(CONCAT(name, ' ', email)) AS full_info_upper FROM test_functions_users WHERE id = 1");
        while (rs.next()) {
            String fullInfoUpper = rs.getString("full_info_upper");
            assert fullInfoUpper.equals("ALICE ALICE@EXAMPLE.COM");
        }
        rs.close();

        // 测试聚合函数与数学函数组合
        rs = executeQuery("SELECT ROUND(AVG(temperature), 2) AS avg_temp_round FROM test_functions_sensor");
        while (rs.next()) {
            double avgTempRound = rs.getDouble("avg_temp_round");
            assert Math.abs(avgTempRound - 25.32) < 0.01;
        }
        rs.close();
    }
}
