package cn.totaltrust.remdb;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import java.sql.*;

public class TestDDL extends TestBase {

    @Before
    public void setUp() throws SQLException {
        super.setUp();
    }

    @After
    public void tearDown() throws SQLException {
        super.tearDown();
    }

    /**
     * 测试 CREATE TABLE 语句
     */
    @Test
    public void testCreateTable() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_create_table (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, email TEXT UNIQUE)");

        // 验证表是否存在
        assert tableExists("test_create_table");

        // 插入测试数据
        executeSql("INSERT INTO test_create_table (name, age, email) VALUES ('Alice', 25, 'alice@example.com')");

        // 验证数据是否插入成功
        verifyRowCount("SELECT * FROM test_create_table", 1);
    }

    /**
     * 测试 ALTER TABLE ADD COLUMN 语句
     */
    @Test
    public void testAlterTableAddColumn() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_alter_add (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL)");

        // 添加新列
        executeSql("ALTER TABLE test_alter_add ADD COLUMN age INTEGER");
        executeSql("ALTER TABLE test_alter_add ADD COLUMN email TEXT");

        // 插入测试数据，包括新列
        executeSql("INSERT INTO test_alter_add (name, age, email) VALUES ('Bob', 30, 'bob@example.com')");

        // 验证数据是否插入成功
        verifyRowCount("SELECT * FROM test_alter_add", 1);

        // 验证新列是否可以查询
        ResultSet rs = executeQuery("SELECT age, email FROM test_alter_add WHERE name = 'Bob'");
        while (rs.next()) {
            int age = rs.getInt("age");
            String email = rs.getString("email");
            assert age == 30;
            assert email.equals("bob@example.com");
        }
        rs.close();
    }

    /**
     * 测试 ALTER TABLE MODIFY COLUMN 语句
     */
    @Test
    public void testAlterTableModifyColumn() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_alter_modify (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, value INTEGER)");

        // 修改列类型
        executeSql("ALTER TABLE test_alter_modify MODIFY COLUMN value REAL");

        // 插入测试数据，使用新的列类型
        executeSql("INSERT INTO test_alter_modify (name, value) VALUES ('Test', 100.5)");

        // 验证数据是否插入成功
        verifyRowCount("SELECT * FROM test_alter_modify", 1);

        // 验证修改后的列类型是否正确
        ResultSet rs = executeQuery("SELECT value FROM test_alter_modify WHERE name = 'Test'");
        while (rs.next()) {
            double value = rs.getDouble("value");
            assert value == 100.5;
        }
        rs.close();
    }

    /**
     * 测试 ALTER TABLE DROP COLUMN 语句
     */
    @Test
    public void testAlterTableDropColumn() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_alter_drop (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, email TEXT)");

        // 插入测试数据
        executeSql("INSERT INTO test_alter_drop (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')");

        // 删除列
        executeSql("ALTER TABLE test_alter_drop DROP COLUMN email");

        // 验证数据是否仍然存在
        verifyRowCount("SELECT * FROM test_alter_drop", 1);

        // 验证删除的列是否不再存在
        try {
            executeQuery("SELECT email FROM test_alter_drop");
            assert false : "Email column should have been dropped";
        } catch (SQLException e) {
            // 预期会抛出异常，因为列已被删除
        }
    }

    /**
     * 测试 DROP TABLE 语句
     */
    @Test
    public void testDropTable() throws SQLException {
        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_drop_table (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL)");

        // 验证表是否存在
        assert tableExists("test_drop_table");

        // 删除表
        executeSql("DROP TABLE test_drop_table");

        // 验证表是否不存在
        assert !tableExists("test_drop_table");
    }

    /**
     * 测试 DROP TABLE IF EXISTS 语句
     */
    @Test
    public void testDropTableIfExists() throws SQLException {
        // 尝试删除不存在的表，应该不会报错
        executeSql("DROP TABLE IF EXISTS test_nonexistent_table");

        // 创建测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_drop_if_exists (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL)");

        // 验证表是否存在
        assert tableExists("test_drop_if_exists");

        // 使用 IF EXISTS 删除表
        executeSql("DROP TABLE IF EXISTS test_drop_if_exists");

        // 验证表是否不存在
        assert !tableExists("test_drop_if_exists");
    }

    /**
     * 测试 CREATE TIMESERIES TABLE 语句
     */
    @Test
    public void testCreateTimeSeriesTable() throws SQLException, InterruptedException {
        // 创建时序表
        executeSql("CREATE TIMESERIES TABLE IF NOT EXISTS test_timeseries_table (timestamp TIMESTAMP, value DOUBLE, sensor_id TEXT, location TEXT) WITH COMPRESSION = (algorithm='delta', enabled=true), WITH TTL = '7 days'");
        // Wait for table creation to complete
        Thread.sleep(100);

        // 验证表是否存在
        assert tableExists("test_timeseries_table");

        // 插入测试数据
        long now = System.currentTimeMillis();
        executeSql("INSERT INTO test_timeseries_table (timestamp, value, sensor_id, location) VALUES (" + now + ", 25.5, 'sensor_1', 'room_1')");

        // 验证数据是否插入成功
        verifyRowCount("SELECT * FROM test_timeseries_table", 1);
    }

    /**
     * 测试表约束
     */
    @Test
    public void testTableConstraints() throws SQLException {
        // 创建带有约束的测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_constraints (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, email TEXT UNIQUE, age INTEGER DEFAULT 0, active BOOLEAN DEFAULT true)");

        // 插入测试数据
        executeSql("INSERT INTO test_constraints (name, email) VALUES ('David', 'david@example.com')");
        executeSql("INSERT INTO test_constraints (name, email, age) VALUES ('Eve', 'eve@example.com', 28)");

        // 验证数据是否插入成功
        verifyRowCount("SELECT * FROM test_constraints", 2);

        // 验证默认值是否生效
        ResultSet rs = executeQuery("SELECT age, active FROM test_constraints WHERE name = 'David'");
        while (rs.next()) {
            int age = rs.getInt("age");
            boolean active = rs.getBoolean("active");
            assert age == 0;
            assert active;
        }
        rs.close();

        // 验证唯一约束是否生效
        try {
            executeSql("INSERT INTO test_constraints (name, email) VALUES ('Frank', 'david@example.com')");
            assert false : "Unique constraint should have been violated";
        } catch (SQLException e) {
            // 预期会抛出异常，因为邮箱已存在
        }
    }

    /**
     * 测试复合主键
     */
    @Test
    public void testCompositePrimaryKey() throws SQLException {
        // 创建带有复合主键的测试表
        executeSql("CREATE TABLE IF NOT EXISTS test_composite_pk (id1 INTEGER, id2 INTEGER, name TEXT, PRIMARY KEY (id1, id2))");

        // 插入测试数据
        executeSql("INSERT INTO test_composite_pk (id1, id2, name) VALUES (1, 1, 'Test 1')");
        executeSql("INSERT INTO test_composite_pk (id1, id2, name) VALUES (1, 2, 'Test 2')");
        executeSql("INSERT INTO test_composite_pk (id1, id2, name) VALUES (2, 1, 'Test 3')");

        // 验证数据是否插入成功
        verifyRowCount("SELECT * FROM test_composite_pk", 3);

        // 验证复合主键约束是否生效
        try {
            executeSql("INSERT INTO test_composite_pk (id1, id2, name) VALUES (1, 1, 'Duplicate')");
            assert false : "Composite primary key constraint should have been violated";
        } catch (SQLException e) {
            // 预期会抛出异常，因为复合主键已存在
        }
    }
}
