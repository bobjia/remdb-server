package cn.totaltrust.remdb;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import java.sql.*;

public class TestDML extends TestBase {

    @Before
    public void setUp() throws SQLException {
        super.setUp();
        // 创建测试表
        createTestTable();
    }

    @After
    public void tearDown() throws SQLException {
        super.tearDown();
    }

    /**
     * 创建测试表
     * @throws SQLException 如果创建失败
     */
    private void createTestTable() throws SQLException {
        executeSql("CREATE TABLE IF NOT EXISTS test_dml (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, email TEXT UNIQUE, active BOOLEAN DEFAULT true)");
    }

    /**
     * 测试 INSERT 语句
     */
    @Test
    public void testInsert() throws SQLException {
        // 插入单条数据
        executeSql("INSERT INTO test_dml (name, age, email) VALUES ('Alice', 25, 'alice@example.com')");
        executeSql("INSERT INTO test_dml (name, age, email) VALUES ('Bob', 30, 'bob@example.com')");
        executeSql("INSERT INTO test_dml (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')");

        // 验证数据是否插入成功
        verifyRowCount("SELECT * FROM test_dml", 3);

        // 验证数据内容
        ResultSet rs = executeQuery("SELECT name, age, email FROM test_dml WHERE id = 1");
        while (rs.next()) {
            String name = rs.getString("name");
            int age = rs.getInt("age");
            String email = rs.getString("email");
            assert name.equals("Alice");
            assert age == 25;
            assert email.equals("alice@example.com");
        }
        rs.close();
    }

    /**
     * 测试 INSERT 语句（使用默认值）
     */
    @Test
    public void testInsertWithDefaultValues() throws SQLException {
        // 插入数据，不指定默认值字段
        executeSql("INSERT INTO test_dml (name, email) VALUES ('David', 'david@example.com')");

        // 验证数据是否插入成功
        verifyRowCount("SELECT * FROM test_dml", 1);

        // 验证默认值是否生效
        ResultSet rs = executeQuery("SELECT age, active FROM test_dml WHERE name = 'David'");
        while (rs.next()) {
            int age = rs.getInt("age");
            boolean active = rs.getBoolean("active");
            assert age == 0; // INTEGER 类型的默认值
            assert active; // BOOLEAN 类型的默认值
        }
        rs.close();
    }

    /**
     * 测试 UPDATE 语句
     */
    @Test
    public void testUpdate() throws SQLException {
        // 插入测试数据
        executeSql("INSERT INTO test_dml (name, age, email) VALUES ('Eve', 28, 'eve@example.com')");

        // 更新数据
        executeSql("UPDATE test_dml SET age = 29, active = false WHERE name = 'Eve'");

        // 验证数据是否更新成功
        ResultSet rs = executeQuery("SELECT age, active FROM test_dml WHERE name = 'Eve'");
        while (rs.next()) {
            int age = rs.getInt("age");
            boolean active = rs.getBoolean("active");
            assert age == 29;
            assert !active;
        }
        rs.close();
    }

    /**
     * 测试 UPDATE 语句（使用 WHERE 条件）
     */
    @Test
    public void testUpdateWithWhere() throws SQLException {
        // 插入测试数据
        executeBatch(new String[]{
            "INSERT INTO test_dml (name, age, email) VALUES ('Frank', 32, 'frank@example.com')",
            "INSERT INTO test_dml (name, age, email) VALUES ('Grace', 27, 'grace@example.com')",
            "INSERT INTO test_dml (name, age, email) VALUES ('Henry', 31, 'henry@example.com')"
        });

        // 更新年龄大于30的数据
        executeSql("UPDATE test_dml SET active = false WHERE age > 30");

        // 验证数据是否更新成功
        ResultSet rs = executeQuery("SELECT name, age, active FROM test_dml WHERE age > 30");
        while (rs.next()) {
            boolean active = rs.getBoolean("active");
            assert !active;
        }
        rs.close();

        // 验证年龄小于等于30的数据是否未被更新
        rs = executeQuery("SELECT name, age, active FROM test_dml WHERE age <= 30");
        while (rs.next()) {
            boolean active = rs.getBoolean("active");
            assert active;
        }
        rs.close();
    }

    /**
     * 测试 DELETE 语句
     */
    @Test
    public void testDelete() throws SQLException {
        // 插入测试数据
        executeBatch(new String[]{
            "INSERT INTO test_dml (name, age, email) VALUES ('Ivy', 26, 'ivy@example.com')",
            "INSERT INTO test_dml (name, age, email) VALUES ('Jack', 33, 'jack@example.com')",
            "INSERT INTO test_dml (name, age, email) VALUES ('Kate', 29, 'kate@example.com')"
        });

        // 验证初始数据行数
        verifyRowCount("SELECT * FROM test_dml", 3);

        // 删除一条数据
        executeSql("DELETE FROM test_dml WHERE name = 'Jack'");

        // 验证数据是否删除成功
        verifyRowCount("SELECT * FROM test_dml", 2);

        // 验证删除的数据是否不存在
        verifyRowCount("SELECT * FROM test_dml WHERE name = 'Jack'", 0);
    }

    /**
     * 测试 DELETE 语句（使用 WHERE 条件）
     */
    @Test
    public void testDeleteWithWhere() throws SQLException {
        // 插入测试数据
        executeBatch(new String[]{
            "INSERT INTO test_dml (name, age, email) VALUES ('Leo', 24, 'leo@example.com')",
            "INSERT INTO test_dml (name, age, email) VALUES ('Mike', 35, 'mike@example.com')",
            "INSERT INTO test_dml (name, age, email) VALUES ('Nancy', 28, 'nancy@example.com')",
            "INSERT INTO test_dml (name, age, email) VALUES ('Oscar', 31, 'oscar@example.com')"
        });

        // 验证初始数据行数
        verifyRowCount("SELECT * FROM test_dml", 4);

        // 删除年龄大于30的数据
        executeSql("DELETE FROM test_dml WHERE age > 30");

        // 验证数据是否删除成功
        verifyRowCount("SELECT * FROM test_dml", 2);

        // 验证剩余的数据
        ResultSet rs = executeQuery("SELECT name, age FROM test_dml");
        while (rs.next()) {
            String name = rs.getString("name");
            int age = rs.getInt("age");
            assert (name.equals("Leo") && age == 24) || (name.equals("Nancy") && age == 28);
        }
        rs.close();
    }

    /**
     * 测试 INSERT、UPDATE、DELETE 组合操作
     */
    @Test
    public void testCombinedDML() throws SQLException {
        // 1. 插入测试数据
        executeSql("INSERT INTO test_dml (name, age, email) VALUES ('Paul', 32, 'paul@example.com')");
        verifyRowCount("SELECT * FROM test_dml", 1);

        // 2. 更新测试数据
        executeSql("UPDATE test_dml SET age = 33, email = 'paul.new@example.com' WHERE name = 'Paul'");
        ResultSet rs = executeQuery("SELECT age, email FROM test_dml WHERE name = 'Paul'");
        while (rs.next()) {
            int age = rs.getInt("age");
            String email = rs.getString("email");
            assert age == 33;
            assert email.equals("paul.new@example.com");
        }
        rs.close();

        // 3. 删除测试数据
        executeSql("DELETE FROM test_dml WHERE name = 'Paul'");
        verifyRowCount("SELECT * FROM test_dml", 0);
    }

    /**
     * 测试 DML 语句的错误处理
     */
    @Test
    public void testDmlErrorHandling() throws SQLException {
        // 插入测试数据
        executeSql("INSERT INTO test_dml (name, age, email) VALUES ('Quinn', 29, 'quinn@example.com')");

        // 测试唯一约束错误
        try {
            executeSql("INSERT INTO test_dml (name, age, email) VALUES ('Rachel', 30, 'quinn@example.com')");
            assert false : "Unique constraint should have been violated";
        } catch (SQLException e) {
            // 预期会抛出异常，因为邮箱已存在
        }

        // 测试非空约束错误
        try {
            executeSql("INSERT INTO test_dml (age, email) VALUES (31, 'rachel@example.com')");
            assert false : "Not null constraint should have been violated";
        } catch (SQLException e) {
            // 预期会抛出异常，因为姓名不能为空
        }
    }
}
