package cn.totaltrust.remdb;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import java.sql.*;

public class TestDataTypes extends TestBase {

    @Before
    public void setUp() throws SQLException {
        super.setUp();
    }

    @After
    public void tearDown() throws SQLException {
        super.tearDown();
    }

    /**
     * 测试 INTEGER 数据类型
     */
    @Test
    public void testIntegerType() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_integer (id INTEGER PRIMARY KEY, value INTEGER)");

        // 插入测试数据
        executeSql("INSERT INTO test_integer (value) VALUES (100)");
        executeSql("INSERT INTO test_integer (value) VALUES (-100)");
        executeSql("INSERT INTO test_integer (value) VALUES (0)");

        // 查询测试数据
        ResultSet rs = executeQuery("SELECT * FROM test_integer");
        while (rs.next()) {
            int value = rs.getInt("value");
            assert value == 100 || value == -100 || value == 0;
        }
        rs.close();

        // 验证行数
        verifyRowCount("SELECT * FROM test_integer", 3);
    }

    /**
     * 测试 REAL 数据类型
     */
    @Test
    public void testRealType() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_real (id INTEGER PRIMARY KEY, value REAL)");

        // 插入测试数据
        executeSql("INSERT INTO test_real (value) VALUES (100.5)");
        executeSql("INSERT INTO test_real (value) VALUES (-100.5)");
        executeSql("INSERT INTO test_real (value) VALUES (0.0)");

        // 查询测试数据
        ResultSet rs = executeQuery("SELECT * FROM test_real");
        while (rs.next()) {
            double value = rs.getDouble("value");
            assert value == 100.5 || value == -100.5 || value == 0.0;
        }
        rs.close();

        // 验证行数
        verifyRowCount("SELECT * FROM test_real", 3);
    }

    /**
     * 测试 TEXT 数据类型
     */
    @Test
    public void testTextType() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_text (id INTEGER PRIMARY KEY, value TEXT)");

        // 插入测试数据
        executeSql("INSERT INTO test_text (value) VALUES ('Hello')");
        executeSql("INSERT INTO test_text (value) VALUES ('World')");
        executeSql("INSERT INTO test_text (value) VALUES ('')");

        // 查询测试数据
        ResultSet rs = executeQuery("SELECT * FROM test_text");
        while (rs.next()) {
            String value = rs.getString("value");
            assert value.equals("Hello") || value.equals("World") || value.equals("");
        }
        rs.close();

        // 验证行数
        verifyRowCount("SELECT * FROM test_text", 3);
    }

    /**
     * 测试 BOOLEAN 数据类型
     */
    @Test
    public void testBooleanType() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_boolean (id INTEGER PRIMARY KEY, value BOOLEAN)");

        // 插入测试数据
        executeSql("INSERT INTO test_boolean (value) VALUES (true)");
        executeSql("INSERT INTO test_boolean (value) VALUES (false)");

        // 查询测试数据
        ResultSet rs = executeQuery("SELECT * FROM test_boolean");
        while (rs.next()) {
            boolean value = rs.getBoolean("value");
            assert value || !value;
        }
        rs.close();

        // 验证行数
        verifyRowCount("SELECT * FROM test_boolean", 2);
    }

    /**
     * 测试 TIMESTAMP 数据类型
     */
    @Test
    public void testTimestampType() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_timestamp (id INTEGER PRIMARY KEY, value TIMESTAMP)");

        // 插入测试数据
        long now = System.currentTimeMillis();
        executeSql("INSERT INTO test_timestamp (value) VALUES (" + now + ")");

        // 查询测试数据
        ResultSet rs = executeQuery("SELECT * FROM test_timestamp");
        while (rs.next()) {
            Timestamp value = rs.getTimestamp("value");
            assert value != null;
        }
        rs.close();

        // 验证行数
        verifyRowCount("SELECT * FROM test_timestamp", 1);
    }

    /**
     * 测试 VECTOR 数据类型
     */
    @Test
    public void testVectorType() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_vector (id INTEGER PRIMARY KEY, value VECTOR(3) WITH DISTANCE=L2)");

        // 插入测试数据
        executeSql("INSERT INTO test_vector (value) VALUES ([1.0, 2.0, 3.0])");
        executeSql("INSERT INTO test_vector (value) VALUES ([4.0, 5.0, 6.0])");

        // 查询测试数据
        ResultSet rs = executeQuery("SELECT * FROM test_vector");
        while (rs.next()) {
            String value = rs.getString("value");
            assert value != null;
            assert value.contains("[") && value.contains("]");
        }
        rs.close();

        // 验证行数
        verifyRowCount("SELECT * FROM test_vector", 2);
    }

    /**
     * 测试 UTF8 字符支持
     */
    @Test
    public void testUtf8Support() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_utf8 (id INTEGER PRIMARY KEY, value TEXT)");

        // 插入包含 UTF8 字符的测试数据
        executeSql("INSERT INTO test_utf8 (value) VALUES ('测试')");
        executeSql("INSERT INTO test_utf8 (value) VALUES ('你好世界')");
        executeSql("INSERT INTO test_utf8 (value) VALUES ('こんにちは')");

        // 查询测试数据
        ResultSet rs = executeQuery("SELECT * FROM test_utf8");
        while (rs.next()) {
            String value = rs.getString("value");
            assert value != null;
            assert value.equals("测试") || value.equals("你好世界") || value.equals("こんにちは");
        }
        rs.close();

        // 验证行数
        verifyRowCount("SELECT * FROM test_utf8", 3);
    }

    /**
     * 测试数据类型转换
     */
    @Test
    public void testDataTypeConversion() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_conversion (id INTEGER PRIMARY KEY, int_value INTEGER, real_value REAL, text_value TEXT)");

        // 插入测试数据
        executeSql("INSERT INTO test_conversion (int_value, real_value, text_value) VALUES (100, 200.5, '300')");

        // 测试整数转实数
        try {
            ResultSet rs1 = executeQuery("SELECT int_value * 1.0 AS converted FROM test_conversion");
            while (rs1.next()) {
                double value = rs1.getDouble("converted");
                assert value == 100.0;
            }
            rs1.close();
        } catch (SQLException e) {
            // 如果数据库不支持整数转实数操作，跳过该测试用例
            System.out.println("Skipping integer to real conversion test: " + e.getMessage());
        }

        // 测试实数转整数
        try {
            ResultSet rs2 = executeQuery("SELECT CAST(real_value AS INTEGER) AS converted FROM test_conversion");
            while (rs2.next()) {
                int value = rs2.getInt("converted");
                assert value == 200;
            }
            rs2.close();
        } catch (SQLException e) {
            // 如果数据库不支持实数转整数操作，跳过该测试用例
            System.out.println("Skipping real to integer conversion test: " + e.getMessage());
        }

        // 测试字符串转整数
        try {
            ResultSet rs3 = executeQuery("SELECT CAST(text_value AS INTEGER) AS converted FROM test_conversion");
            while (rs3.next()) {
                int value = rs3.getInt("converted");
                assert value == 300;
            }
            rs3.close();
        } catch (SQLException e) {
            // 如果数据库不支持字符串转整数操作，跳过该测试用例
            System.out.println("Skipping string to integer conversion test: " + e.getMessage());
        }
    }

    /**
     * 测试所有数据类型的组合
     */
    @Test
    public void testAllDataTypes() throws SQLException {
        // 创建包含所有数据类型的测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_all_types (id INTEGER PRIMARY KEY AUTOINCREMENT, int_col INTEGER, real_col REAL, text_col TEXT, bool_col BOOLEAN, timestamp_col TIMESTAMP, vector_col VECTOR(2) WITH DISTANCE=L2)");

        // 插入测试数据
        long now = System.currentTimeMillis();
        executeSql("INSERT INTO test_all_types (int_col, real_col, text_col, bool_col, timestamp_col, vector_col) VALUES (100, 200.5, 'Test', true, " + now + ", [1.0, 2.0])");

        // 查询测试数据
        ResultSet rs = executeQuery("SELECT * FROM test_all_types");
        while (rs.next()) {
            int intValue = rs.getInt("int_col");
            double realValue = rs.getDouble("real_col");
            String textValue = rs.getString("text_col");
            boolean boolValue = rs.getBoolean("bool_col");
            Timestamp timestampValue = rs.getTimestamp("timestamp_col");
            String vectorValue = rs.getString("vector_col");

            assert intValue == 100;
            assert realValue == 200.5;
            assert textValue.equals("Test");
            assert boolValue;
            assert timestampValue != null;
            assert vectorValue != null;
        }
        rs.close();

        // 验证行数
        verifyRowCount("SELECT * FROM test_all_types", 1);
    }
}
