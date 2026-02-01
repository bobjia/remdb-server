package cn.totaltrust.remdb;

import org.junit.Test;

import java.sql.SQLException;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class TestIntegration extends TestBase {

    @Test
    public void testComplexQueryWithMultipleTables() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS departments (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, location TEXT)");
        executeUpdate("CREATE TABLE IF NOT EXISTS employees (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, department_id INTEGER, salary REAL, hire_date TIMESTAMP)");

        // 插入测试数据
        executeUpdate("INSERT INTO departments (name, location) VALUES ('Engineering', 'Beijing')");
        executeUpdate("INSERT INTO departments (name, location) VALUES ('Marketing', 'Shanghai')");
        executeUpdate("INSERT INTO employees (name, department_id, salary, hire_date) VALUES ('Alice', 1, 10000.0, '2023-01-01')");
        executeUpdate("INSERT INTO employees (name, department_id, salary, hire_date) VALUES ('Bob', 1, 12000.0, '2023-02-01')");
        executeUpdate("INSERT INTO employees (name, department_id, salary, hire_date) VALUES ('Charlie', 2, 8000.0, '2023-03-01')");

        // 测试复杂查询：连接多个表，使用聚合函数，分组和排序
        var resultSet = executeQuery("" +
                "SELECT d.name AS department, COUNT(e.id) AS employee_count, AVG(e.salary) AS avg_salary " +
                "FROM departments d " +
                "LEFT JOIN employees e ON d.id = e.department_id " +
                "GROUP BY d.id " +
                "ORDER BY employee_count DESC"
        );

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals("Engineering", resultSet.getString("department"));
        assertEquals(2, resultSet.getInt("employee_count"));
        assertEquals(11000.0, resultSet.getDouble("avg_salary"), 0.001);

        assertTrue(resultSet.next());
        assertEquals("Marketing", resultSet.getString("department"));
        assertEquals(1, resultSet.getInt("employee_count"));
        assertEquals(8000.0, resultSet.getDouble("avg_salary"), 0.001);

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE employees");
        executeUpdate("DROP TABLE departments");
    }

    @Test
    public void testOrderProcessingSystem() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS customers (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, email TEXT)");
        executeUpdate("CREATE TABLE IF NOT EXISTS products (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, price REAL, stock INTEGER)");
        executeUpdate("CREATE TABLE IF NOT EXISTS orders (id INTEGER PRIMARY KEY AUTOINCREMENT, customer_id INTEGER, order_date TIMESTAMP, total_amount REAL)");
        executeUpdate("CREATE TABLE IF NOT EXISTS order_items (id INTEGER PRIMARY KEY AUTOINCREMENT, order_id INTEGER, product_id INTEGER, quantity INTEGER, unit_price REAL)");

        // 插入测试数据
        executeUpdate("INSERT INTO customers (name, email) VALUES ('Customer1', 'customer1@example.com')");
        executeUpdate("INSERT INTO customers (name, email) VALUES ('Customer2', 'customer2@example.com')");
        executeUpdate("INSERT INTO products (name, price, stock) VALUES ('Product1', 100.0, 100)");
        executeUpdate("INSERT INTO products (name, price, stock) VALUES ('Product2', 200.0, 50)");
        executeUpdate("INSERT INTO orders (customer_id, order_date, total_amount) VALUES (1, '2023-01-01', 300.0)");
        executeUpdate("INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES (1, 1, 1, 100.0)");
        executeUpdate("INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES (1, 2, 1, 200.0)");

        // 测试订单处理系统的复杂查询
        var resultSet = executeQuery("" +
                "SELECT c.name AS customer_name, o.order_date, o.total_amount, " +
                "       GROUP_CONCAT(p.name || ' x ' || oi.quantity) AS products " +
                "FROM customers c " +
                "JOIN orders o ON c.id = o.customer_id " +
                "JOIN order_items oi ON o.id = oi.order_id " +
                "JOIN products p ON oi.product_id = p.id " +
                "GROUP BY o.id " +
                "ORDER BY o.order_date DESC"
        );

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals("Customer1", resultSet.getString("customer_name"));
        assertEquals(300.0, resultSet.getDouble("total_amount"), 0.001);
        String products = resultSet.getString("products");
        assertTrue(products.contains("Product1 x 1"));
        assertTrue(products.contains("Product2 x 1"));

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE order_items");
        executeUpdate("DROP TABLE orders");
        executeUpdate("DROP TABLE products");
        executeUpdate("DROP TABLE customers");
    }

    @Test
    public void testTimeSeriesWithAggregations() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS sensor_data (id INTEGER PRIMARY KEY AUTOINCREMENT, sensor_id INTEGER, timestamp TIMESTAMP, value REAL)");

        // 插入测试数据
        executeUpdate("INSERT INTO sensor_data (sensor_id, timestamp, value) VALUES (1, '2023-01-01 00:00:00', 10.0)");
        executeUpdate("INSERT INTO sensor_data (sensor_id, timestamp, value) VALUES (1, '2023-01-01 01:00:00', 15.0)");
        executeUpdate("INSERT INTO sensor_data (sensor_id, timestamp, value) VALUES (1, '2023-01-01 02:00:00', 12.0)");
        executeUpdate("INSERT INTO sensor_data (sensor_id, timestamp, value) VALUES (2, '2023-01-01 00:00:00', 20.0)");
        executeUpdate("INSERT INTO sensor_data (sensor_id, timestamp, value) VALUES (2, '2023-01-01 01:00:00', 25.0)");

        // 测试时序数据的聚合查询
        var resultSet = executeQuery("" +
                "SELECT sensor_id, MIN(value) AS min_value, MAX(value) AS max_value, AVG(value) AS avg_value " +
                "FROM sensor_data " +
                "WHERE timestamp >= '2023-01-01' AND timestamp < '2023-01-02' " +
                "GROUP BY sensor_id " +
                "ORDER BY sensor_id"
        );

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals(1, resultSet.getInt("sensor_id"));
        assertEquals(10.0, resultSet.getDouble("min_value"), 0.001);
        assertEquals(15.0, resultSet.getDouble("max_value"), 0.001);
        assertEquals(12.3333333333, resultSet.getDouble("avg_value"), 0.001);

        assertTrue(resultSet.next());
        assertEquals(2, resultSet.getInt("sensor_id"));
        assertEquals(20.0, resultSet.getDouble("min_value"), 0.001);
        assertEquals(25.0, resultSet.getDouble("max_value"), 0.001);
        assertEquals(22.5, resultSet.getDouble("avg_value"), 0.001);

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE sensor_data");
    }

    @Test
    public void testVectorWithRelationalData() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS items (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, description TEXT)");
        executeUpdate("CREATE TABLE IF NOT EXISTS item_vectors (item_id INTEGER PRIMARY KEY, vec VECTOR(3) WITH DISTANCE=L2)");

        // 插入测试数据
        executeUpdate("INSERT INTO items (id, name, description) VALUES (1, 'Item1', 'This is item 1')");
        executeUpdate("INSERT INTO items (id, name, description) VALUES (2, 'Item2', 'This is item 2')");
        executeUpdate("INSERT INTO items (id, name, description) VALUES (3, 'Item3', 'This is item 3')");
        executeUpdate("INSERT INTO item_vectors (item_id, vec) VALUES (1, VECTOR(1.0, 2.0, 3.0))");
        executeUpdate("INSERT INTO item_vectors (item_id, vec) VALUES (2, VECTOR(1.1, 2.1, 3.1))");
        executeUpdate("INSERT INTO item_vectors (item_id, vec) VALUES (3, VECTOR(4.0, 5.0, 6.0))");

        // 测试向量与关系数据的结合查询
        var resultSet = executeQuery("" +
                "SELECT i.name, iv.vec <-> VECTOR(1.0, 2.0, 3.0) AS distance " +
                "FROM items i " +
                "JOIN item_vectors iv ON i.id = iv.item_id " +
                "ORDER BY distance LIMIT 2"
        );

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals("Item1", resultSet.getString("name"));
        assertTrue(resultSet.getDouble("distance") < 0.001);

        assertTrue(resultSet.next());
        assertEquals("Item2", resultSet.getString("name"));
        assertTrue(resultSet.getDouble("distance") < 0.5);

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE item_vectors");
        executeUpdate("DROP TABLE items");
    }

    @Test
    public void testComplexTransactionWithMultipleOperations() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS accounts (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, balance REAL)");
        executeUpdate("CREATE TABLE IF NOT EXISTS transactions (id INTEGER PRIMARY KEY AUTOINCREMENT, from_account_id INTEGER, to_account_id INTEGER, amount REAL, transaction_date TIMESTAMP)");

        // 插入初始数据
        executeUpdate("INSERT INTO accounts (name, balance) VALUES ('Account1', 1000.0)");
        executeUpdate("INSERT INTO accounts (name, balance) VALUES ('Account2', 500.0)");

        try {
            // 开始事务
            connection.setAutoCommit(false);

            // 执行转账操作
            executeUpdate("UPDATE accounts SET balance = balance - 200.0 WHERE id = 1");
            executeUpdate("UPDATE accounts SET balance = balance + 200.0 WHERE id = 2");
            executeUpdate("INSERT INTO transactions (from_account_id, to_account_id, amount, transaction_date) VALUES (1, 2, 200.0, CURRENT_TIMESTAMP)");

            // 验证余额
            var resultSet = executeQuery("SELECT balance FROM accounts WHERE id = 1");
            resultSet.next();
            assertEquals(800.0, resultSet.getDouble("balance"), 0.001);
            resultSet.close();

            resultSet = executeQuery("SELECT balance FROM accounts WHERE id = 2");
            resultSet.next();
            assertEquals(700.0, resultSet.getDouble("balance"), 0.001);
            resultSet.close();

            // 验证交易记录
            resultSet = executeQuery("SELECT COUNT(*) FROM transactions");
            resultSet.next();
            assertEquals(1, resultSet.getInt(1));
            resultSet.close();

            // 提交事务
            connection.commit();
        } finally {
            // 恢复自动提交
            connection.setAutoCommit(true);
        }

        // 清理测试表
        executeUpdate("DROP TABLE transactions");
        executeUpdate("DROP TABLE accounts");
    }
}
