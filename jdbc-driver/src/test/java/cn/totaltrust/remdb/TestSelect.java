package cn.totaltrust.remdb;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import java.sql.*;

public class TestSelect extends TestBase {

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
        executeSql("CREATE TABLE IF NOT EXISTS test_users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, email TEXT UNIQUE)");
        // 创建订单表
        executeSql("CREATE TABLE IF NOT EXISTS test_orders (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER, product TEXT NOT NULL, amount REAL, order_date TIMESTAMP)");
    }

    /**
     * 插入测试数据
     * @throws SQLException 如果插入失败
     */
    private void insertTestData() throws SQLException {
        // 插入用户数据
        executeBatch(new String[]{
            "INSERT INTO test_users (name, age, email) VALUES ('Alice', 25, 'alice@example.com')",
            "INSERT INTO test_users (name, age, email) VALUES ('Bob', 30, 'bob@example.com')",
            "INSERT INTO test_users (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')",
            "INSERT INTO test_users (name, age, email) VALUES ('Alice', 28, 'alice2@example.com')" // 测试DISTINCT
        });

        // 插入订单数据
        long now = System.currentTimeMillis();
        executeBatch(new String[]{
            "INSERT INTO test_orders (user_id, product, amount, order_date) VALUES (1, 'Product A', 100.0, " + now + ")",
            "INSERT INTO test_orders (user_id, product, amount, order_date) VALUES (1, 'Product B', 200.0, " + (now + 3600000) + ")",
            "INSERT INTO test_orders (user_id, product, amount, order_date) VALUES (2, 'Product C', 150.0, " + now + ")",
            "INSERT INTO test_orders (user_id, product, amount, order_date) VALUES (3, 'Product A', 100.0, " + (now + 7200000) + ")"
        });
    }

    /**
     * 测试基本 SELECT 语句
     */
    @Test
    public void testBasicSelect() throws SQLException {
        // 测试 SELECT *
        ResultSet rs = executeQuery("SELECT * FROM test_users");
        int count = 0;
        while (rs.next()) {
            count++;
            int id = rs.getInt("id");
            String name = rs.getString("name");
            int age = rs.getInt("age");
            String email = rs.getString("email");
            assert id > 0;
            assert name != null;
            assert email != null;
        }
        rs.close();
        assert count == 4;

        // 测试 SELECT 特定列
        rs = executeQuery("SELECT name, age FROM test_users WHERE id = 1");
        while (rs.next()) {
            String name = rs.getString("name");
            int age = rs.getInt("age");
            assert name.equals("Alice");
            assert age == 25;
        }
        rs.close();
    }

    /**
     * 测试 SELECT DISTINCT 语句
     */
    @Test
    public void testSelectDistinct() throws SQLException {
        // 测试 DISTINCT 单列
        ResultSet rs = executeQuery("SELECT DISTINCT name FROM test_users");
        int count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 3; // 应该只有 Alice, Bob, Charlie

        // 测试 DISTINCT 多列
        rs = executeQuery("SELECT DISTINCT name, age FROM test_users");
        count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 4; // 所有组合都是唯一的
    }

    /**
     * 测试 SELECT 语句的别名
     */
    @Test
    public void testSelectWithAlias() throws SQLException {
        // 测试列别名
        ResultSet rs = executeQuery("SELECT name AS user_name, age AS user_age FROM test_users WHERE id = 2");
        while (rs.next()) {
            String userName = rs.getString("user_name");
            int userAge = rs.getInt("user_age");
            assert userName.equals("Bob");
            assert userAge == 30;
        }
        rs.close();

        // 测试表别名
        rs = executeQuery("SELECT u.name, u.age FROM test_users AS u WHERE u.id = 3");
        while (rs.next()) {
            String name = rs.getString("name");
            int age = rs.getInt("age");
            assert name.equals("Charlie");
            assert age == 35;
        }
        rs.close();
    }

    /**
     * 测试 SELECT 语句的 WHERE 子句
     */
    @Test
    public void testSelectWithWhere() throws SQLException {
        // 测试 WHERE 条件
        ResultSet rs = executeQuery("SELECT name, age FROM test_users WHERE age > 25");
        int count = 0;
        while (rs.next()) {
            int age = rs.getInt("age");
            assert age > 25;
            count++;
        }
        rs.close();
        assert count == 3; // Bob (30), Charlie (35), Alice2 (28)

        // 测试 WHERE 条件组合
        rs = executeQuery("SELECT name, age FROM test_users WHERE age > 25 AND name = 'Alice'");
        count = 0;
        while (rs.next()) {
            String name = rs.getString("name");
            int age = rs.getInt("age");
            assert name.equals("Alice");
            assert age == 28;
            count++;
        }
        rs.close();
        assert count == 1;
    }

    /**
     * 测试 SELECT 语句的 ORDER BY 子句
     */
    @Test
    public void testSelectWithOrderBy() throws SQLException {
        // 测试 ORDER BY ASC
        ResultSet rs = executeQuery("SELECT name, age FROM test_users ORDER BY age ASC");
        int previousAge = 0;
        while (rs.next()) {
            int age = rs.getInt("age");
            assert age >= previousAge;
            previousAge = age;
        }
        rs.close();

        // 测试 ORDER BY DESC
        rs = executeQuery("SELECT name, age FROM test_users ORDER BY age DESC");
        previousAge = Integer.MAX_VALUE;
        while (rs.next()) {
            int age = rs.getInt("age");
            assert age <= previousAge;
            previousAge = age;
        }
        rs.close();

        // 测试 ORDER BY 多列
        rs = executeQuery("SELECT name, age FROM test_users ORDER BY name ASC, age ASC");
        String previousName = "";
        previousAge = 0;
        while (rs.next()) {
            String name = rs.getString("name");
            int age = rs.getInt("age");
            if (name.equals(previousName)) {
                assert age >= previousAge;
            }
            previousName = name;
            previousAge = age;
        }
        rs.close();
    }

    /**
     * 测试 SELECT 语句的 LIMIT 子句
     */
    @Test
    public void testSelectWithLimit() throws SQLException {
        // 测试 LIMIT
        ResultSet rs = executeQuery("SELECT * FROM test_users LIMIT 2");
        int count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 2;

        // 测试 ORDER BY 与 LIMIT 组合
        rs = executeQuery("SELECT * FROM test_users ORDER BY age DESC LIMIT 2");
        count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 2;
    }

    /**
     * 测试 SELECT 语句的 GROUP BY 子句
     */
    @Test
    public void testSelectWithGroupBy() throws SQLException {
        // 测试 GROUP BY 与 COUNT
        ResultSet rs = executeQuery("SELECT name, COUNT(*) AS user_count FROM test_users GROUP BY name");
        while (rs.next()) {
            String name = rs.getString("name");
            int count = rs.getInt("user_count");
            if (name.equals("Alice")) {
                assert count == 2;
            } else if (name.equals("Bob") || name.equals("Charlie")) {
                assert count == 1;
            }
        }
        rs.close();

        // 测试 GROUP BY 与 SUM
        rs = executeQuery("SELECT user_id, SUM(amount) AS total_amount FROM test_orders GROUP BY user_id");
        while (rs.next()) {
            int userId = rs.getInt("user_id");
            double totalAmount = rs.getDouble("total_amount");
            if (userId == 1) {
                assert totalAmount == 300.0; // 100 + 200
            } else if (userId == 2) {
                assert totalAmount == 150.0; // 150
            } else if (userId == 3) {
                assert totalAmount == 100.0; // 100
            }
        }
        rs.close();
    }

    /**
     * 测试 INNER JOIN
     */
    @Test
    public void testInnerJoin() throws SQLException {
        ResultSet rs = executeQuery(
            "SELECT u.name, o.product, o.amount " +
            "FROM test_users u " +
            "INNER JOIN test_orders o ON u.id = o.user_id " +
            "WHERE u.name = 'Alice'"
        );
        int count = 0;
        while (rs.next()) {
            String name = rs.getString("name");
            String product = rs.getString("product");
            double amount = rs.getDouble("amount");
            assert name.equals("Alice");
            assert (product.equals("Product A") && amount == 100.0) || (product.equals("Product B") && amount == 200.0);
            count++;
        }
        rs.close();
        assert count == 2;
    }

    /**
     * 测试 LEFT JOIN
     */
    @Test
    public void testLeftJoin() throws SQLException {
        // 创建一个没有订单的用户
        executeSql("INSERT INTO test_users (name, age, email) VALUES ('David', 40, 'david@example.com')");

        ResultSet rs = executeQuery(
            "SELECT u.name, o.product " +
            "FROM test_users u " +
            "LEFT JOIN test_orders o ON u.id = o.user_id"
        );
        int count = 0;
        boolean foundDavid = false;
        while (rs.next()) {
            String name = rs.getString("name");
            String product = rs.getString("product");
            if (name.equals("David")) {
                assert product == null;
                foundDavid = true;
            }
            count++;
        }
        rs.close();
        assert count == 5; // 4个原有用户 + 1个新用户
        assert foundDavid;
    }

    /**
     * 测试 SELECT 语句的复杂组合
     */
    @Test
    public void testComplexSelect() throws SQLException {
        ResultSet rs = executeQuery(
            "SELECT u.name AS user_name, o.product, o.amount " +
            "FROM test_users u " +
            "INNER JOIN test_orders o ON u.id = o.user_id " +
            "WHERE o.amount > 100 " +
            "ORDER BY o.amount DESC " +
            "LIMIT 2"
        );
        int count = 0;
        double previousAmount = Double.MAX_VALUE;
        while (rs.next()) {
            String userName = rs.getString("user_name");
            String product = rs.getString("product");
            double amount = rs.getDouble("amount");
            assert amount > 100;
            assert amount <= previousAmount;
            previousAmount = amount;
            count++;
        }
        rs.close();
        assert count == 2;
    }
}
