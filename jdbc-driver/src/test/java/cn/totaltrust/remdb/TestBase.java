package cn.totaltrust.remdb;

import java.sql.*;
import java.util.Properties;

public class TestBase {
    protected Connection connection;
    protected Statement statement;

    // 测试数据库连接URL
    protected static final String JDBC_URL = "jdbc:remdb://localhost:6666/default";
    // 测试数据库名称
    protected static final String TEST_DATABASE = "default";

    /**
     * 初始化数据库连接
     * @throws SQLException 如果连接失败
     */
    protected void setUp() throws SQLException {
        try {
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
        } catch (ClassNotFoundException e) {
            throw new SQLException("RemDbDriver not found", e);
        }
        // 建立数据库连接
        connection = DriverManager.getConnection(JDBC_URL);
        statement = connection.createStatement();
        
        // 确保测试数据库存在
        createTestDatabase();
        
        // 使用测试数据库
        useTestDatabase();
        
        // 清理测试数据
        cleanupTestData();
    }

    /**
     * 关闭数据库连接
     * @throws SQLException 如果关闭失败
     */
    protected void tearDown() throws SQLException {
        if (statement != null) {
            statement.close();
        }
        if (connection != null) {
            connection.close();
        }
    }

    /**
     * 创建测试数据库
     * @throws SQLException 如果创建失败
     */
    protected void createTestDatabase() throws SQLException {
        try {
            statement.executeUpdate("CREATE DATABASE IF NOT EXISTS " + TEST_DATABASE);
        } catch (SQLException e) {
            // 忽略数据库已存在的错误
            if (!e.getMessage().contains("DatabaseExists") && !e.getMessage().contains("already exists")) {
                throw e;
            }
        }
    }

    /**
     * 使用测试数据库
     * @throws SQLException 如果切换失败
     */
    protected void useTestDatabase() throws SQLException {
        // 数据库已在连接URL中指定，不需要再执行USE DATABASE
        // statement.executeUpdate("USE DATABASE " + TEST_DATABASE);
    }

    /**
     * 清理测试数据
     * @throws SQLException 如果清理失败
     */
    protected void cleanupTestData() throws SQLException {
        // 清理测试表
        String[] testTables = {
            "test_users",
            "test_products",
            "test_sensor_data",
            "test_vectors",
            "test_timeseries",
            "test_create_table",
            "test_alter_add",
            "test_alter_modify",
            "test_alter_drop",
            "test_drop_table",
            "test_drop_if_exists",
            "test_timeseries_table",
            "test_constraints",
            "test_composite_pk"
        };

        for (String table : testTables) {
            try {
                statement.executeUpdate("DROP TABLE IF EXISTS " + table);
            } catch (SQLException e) {
                // 忽略表不存在的错误
                if (!e.getMessage().contains("does not exist")) {
                    throw e;
                }
            }
        }
    }

    /**
     * 执行SQL语句
     * @param sql SQL语句
     * @throws SQLException 如果执行失败
     */
    protected void executeSql(String sql) throws SQLException {
        statement.executeUpdate(sql);
    }

    /**
     * 执行SQL更新语句
     * @param sql SQL语句
     * @throws SQLException 如果执行失败
     */
    protected void executeUpdate(String sql) throws SQLException {
        statement.executeUpdate(sql);
    }

    /**
     * 执行查询语句
     * @param sql SQL查询语句
     * @return 结果集
     * @throws SQLException 如果查询失败
     */
    protected ResultSet executeQuery(String sql) throws SQLException {
        return statement.executeQuery(sql);
    }

    /**
     * 验证查询结果行数
     * @param sql SQL查询语句
     * @param expectedCount 期望的行数
     * @throws SQLException 如果查询失败
     */
    protected void verifyRowCount(String sql, int expectedCount) throws SQLException {
        ResultSet rs = executeQuery(sql);
        int actualCount = 0;
        while (rs.next()) {
            actualCount++;
        }
        rs.close();
        assert actualCount == expectedCount : "Expected " + expectedCount + " rows, got " + actualCount;
    }

    /**
     * 验证表是否存在
     * @param tableName 表名
     * @return 是否存在
     * @throws SQLException 如果查询失败
     */
    protected boolean tableExists(String tableName) throws SQLException {
        try {
            ResultSet rs = executeQuery("DESCRIBE " + tableName);
            rs.close();
            return true;
        } catch (SQLException e) {
            return false;
        }
    }

    /**
     * 批量执行SQL语句
     * @param sqls SQL语句数组
     * @throws SQLException 如果执行失败
     */
    protected void executeBatch(String[] sqls) throws SQLException {
        for (String sql : sqls) {
            executeSql(sql);
        }
    }

    /**
     * 创建测试用户表
     * @throws SQLException 如果创建失败
     */
    protected void createTestUsersTable() throws SQLException {
        executeSql("CREATE TABLE IF NOT EXISTS test_users (" +
                "id INTEGER PRIMARY KEY AUTOINCREMENT, " +
                "name TEXT NOT NULL, " +
                "age INTEGER, " +
                "email TEXT UNIQUE, " +
                "created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP" +
                ")");
    }

    /**
     * 插入测试用户数据
     * @throws SQLException 如果插入失败
     */
    protected void insertTestUsersData() throws SQLException {
        executeBatch(new String[]{
            "INSERT INTO test_users (name, age, email) VALUES ('Alice', 25, 'alice@example.com')",
            "INSERT INTO test_users (name, age, email) VALUES ('Bob', 30, 'bob@example.com')",
            "INSERT INTO test_users (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')"
        });
    }

    /**
     * 创建测试产品表
     * @throws SQLException 如果创建失败
     */
    protected void createTestProductsTable() throws SQLException {
        executeSql("CREATE TABLE IF NOT EXISTS test_products (" +
                "id INTEGER PRIMARY KEY AUTOINCREMENT, " +
                "name TEXT NOT NULL, " +
                "price REAL, " +
                "description TEXT, " +
                "in_stock BOOLEAN DEFAULT true" +
                ")");
    }

    /**
     * 插入测试产品数据
     * @throws SQLException 如果插入失败
     */
    protected void insertTestProductsData() throws SQLException {
        executeBatch(new String[]{
            "INSERT INTO test_products (name, price, description) VALUES ('Product A', 100.0, 'Description for Product A')",
            "INSERT INTO test_products (name, price, description) VALUES ('Product B', 200.0, 'Description for Product B')",
            "INSERT INTO test_products (name, price, description) VALUES ('Product C', 300.0, 'Description for Product C')"
        });
    }

    /**
     * 创建测试传感器数据表
     * @throws SQLException 如果创建失败
     */
    protected void createTestSensorDataTable() throws SQLException {
        executeSql("CREATE TABLE IF NOT EXISTS test_sensor_data (" +
                "id INTEGER PRIMARY KEY AUTOINCREMENT, " +
                "sensor_id INTEGER, " +
                "temperature REAL, " +
                "humidity REAL, " +
                "timestamp TIMESTAMP" +
                ")");
    }

    /**
     * 插入测试传感器数据
     * @throws SQLException 如果插入失败
     */
    protected void insertTestSensorData() throws SQLException {
        long now = System.currentTimeMillis();
        executeBatch(new String[]{
            "INSERT INTO test_sensor_data (sensor_id, temperature, humidity, timestamp) VALUES (1, 25.5, 60.0, " + now + ")",
            "INSERT INTO test_sensor_data (sensor_id, temperature, humidity, timestamp) VALUES (1, 26.0, 59.5, " + (now + 60000) + ")",
            "INSERT INTO test_sensor_data (sensor_id, temperature, humidity, timestamp) VALUES (2, 24.5, 62.0, " + now + ")"
        });
    }

    /**
     * 创建测试向量表
     * @throws SQLException 如果创建失败
     */
    protected void createTestVectorsTable() throws SQLException {
        executeSql("CREATE TABLE IF NOT EXISTS test_vectors (" +
                "id INTEGER PRIMARY KEY AUTOINCREMENT, " +
                "vec VECTOR(3) WITH DISTANCE=L2, " +
                "meta TEXT" +
                ")");
    }

    /**
     * 插入测试向量数据
     * @throws SQLException 如果插入失败
     */
    protected void insertTestVectorsData() throws SQLException {
        executeBatch(new String[]{
            "INSERT INTO test_vectors (vec, meta) VALUES ([1.0, 2.0, 3.0], 'Vector 1')",
            "INSERT INTO test_vectors (vec, meta) VALUES ([4.0, 5.0, 6.0], 'Vector 2')",
            "INSERT INTO test_vectors (vec, meta) VALUES ([7.0, 8.0, 9.0], 'Vector 3')"
        });
    }

    /**
     * 创建测试时序表
     * @throws SQLException 如果创建失败
     */
    protected void createTestTimeSeriesTable() throws SQLException {
        executeSql("CREATE TIMESERIES TABLE IF NOT EXISTS test_timeseries (" +
                "timestamp TIMESTAMP, " +
                "value REAL, " +
                "sensor_id TEXT, " +
                "location TEXT" +
                ") WITH COMPRESSION = (algorithm='delta', enabled=true)" +
                ", WITH TTL = '7 days'");
    }

    /**
     * 插入测试时序数据
     * @throws SQLException 如果插入失败
     */
    protected void insertTestTimeSeriesData() throws SQLException {
        long now = System.currentTimeMillis();
        executeBatch(new String[]{
            "INSERT INTO test_timeseries (timestamp, value, sensor_id, location) VALUES (" + now + ", 25.5, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries (timestamp, value, sensor_id, location) VALUES (" + (now + 60000) + ", 26.0, 'sensor_1', 'room_1')",
            "INSERT INTO test_timeseries (timestamp, value, sensor_id, location) VALUES (" + (now + 120000) + ", 25.8, 'sensor_2', 'room_2')"
        });
    }
}
