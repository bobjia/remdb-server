package cn.totaltrust.remdb;

import org.junit.Test;

import java.sql.SQLException;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class TestTransaction extends TestBase {

    @Test
    public void testTransactionCommit() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_transaction (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, value INTEGER)");

        try {
            // 开始事务
            connection.setAutoCommit(false);

            // 执行多个操作
            executeUpdate("INSERT INTO test_transaction (name, value) VALUES ('test1', 100)");
            executeUpdate("INSERT INTO test_transaction (name, value) VALUES ('test2', 200)");

            // 提交事务
            connection.commit();

            // 验证数据已提交
            var resultSet = executeQuery("SELECT COUNT(*) FROM test_transaction");
            resultSet.next();
            assertEquals(2, resultSet.getInt(1));
            resultSet.close();
        } catch (SQLException e) {
            // 回滚事务
            connection.rollback();
            throw e;
        } finally {
            // 恢复自动提交
            connection.setAutoCommit(true);
        }

        // 清理测试表
        executeUpdate("DROP TABLE test_transaction");
    }

    @Test
    public void testTransactionRollback() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_transaction (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, value INTEGER)");

        try {
            // 开始事务
            connection.setAutoCommit(false);

            // 执行插入操作
            executeUpdate("INSERT INTO test_transaction (name, value) VALUES ('test1', 100)");

            // 验证事务中的数据
            var resultSet = executeQuery("SELECT COUNT(*) FROM test_transaction");
            resultSet.next();
            assertEquals(1, resultSet.getInt(1));
            resultSet.close();

            // 回滚事务
            connection.rollback();

            // 验证数据已回滚
            resultSet = executeQuery("SELECT COUNT(*) FROM test_transaction");
            resultSet.next();
            assertEquals(0, resultSet.getInt(1));
            resultSet.close();
        } finally {
            // 恢复自动提交
            connection.setAutoCommit(true);
        }

        // 清理测试表
        executeUpdate("DROP TABLE test_transaction");
    }

    @Test
    public void testTransactionIsolation() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_transaction (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, value INTEGER)");

        try {
            // 设置事务隔离级别为可重复读
            connection.setTransactionIsolation(java.sql.Connection.TRANSACTION_REPEATABLE_READ);
            connection.setAutoCommit(false);

            // 插入初始数据
            executeUpdate("INSERT INTO test_transaction (name, value) VALUES ('test1', 100)");
            connection.commit();

            // 开始新事务
            connection.setAutoCommit(false);

            // 读取数据
            var resultSet = executeQuery("SELECT value FROM test_transaction WHERE name = 'test1'");
            resultSet.next();
            int initialValue = resultSet.getInt(1);
            assertEquals(100, initialValue);
            resultSet.close();

            // 在同一事务中再次读取
            resultSet = executeQuery("SELECT value FROM test_transaction WHERE name = 'test1'");
            resultSet.next();
            int secondValue = resultSet.getInt(1);
            assertEquals(initialValue, secondValue);
            resultSet.close();

            // 提交事务
            connection.commit();
        } finally {
            // 恢复自动提交
            connection.setAutoCommit(true);
        }

        // 清理测试表
        executeUpdate("DROP TABLE test_transaction");
    }

    @Test
    public void testTransactionBatchOperations() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_transaction (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, value INTEGER)");

        try {
            // 开始事务
            connection.setAutoCommit(false);

            // 创建批处理
            var statement = connection.createStatement();
            statement.addBatch("INSERT INTO test_transaction (name, value) VALUES ('test1', 100)");
            statement.addBatch("INSERT INTO test_transaction (name, value) VALUES ('test2', 200)");
            statement.addBatch("INSERT INTO test_transaction (name, value) VALUES ('test3', 300)");

            // 执行批处理
            int[] results = statement.executeBatch();
            assertEquals(3, results.length);

            // 提交事务
            connection.commit();

            // 验证数据已提交
            var resultSet = executeQuery("SELECT COUNT(*) FROM test_transaction");
            resultSet.next();
            assertEquals(3, resultSet.getInt(1));
            resultSet.close();

            statement.close();
        } catch (SQLException e) {
            // 回滚事务
            connection.rollback();
            throw e;
        } finally {
            // 恢复自动提交
            connection.setAutoCommit(true);
        }

        // 清理测试表
        executeUpdate("DROP TABLE test_transaction");
    }

    @Test
    public void testTransactionExceptionRollback() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_transaction (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, value INTEGER)");

        try {
            // 开始事务
            connection.setAutoCommit(false);

            // 执行正常插入
            executeUpdate("INSERT INTO test_transaction (name, value) VALUES ('test1', 100)");

            // 执行会失败的操作（例如，插入重复的主键）
            try {
                executeUpdate("INSERT INTO test_transaction (id, name, value) VALUES (1, 'test2', 200)");
            } catch (SQLException e) {
                // 预期会失败，继续执行
            }

            // 尝试提交事务
            connection.commit();

            // 验证事务是否回滚（应该没有数据）
            var resultSet = executeQuery("SELECT COUNT(*) FROM test_transaction");
            resultSet.next();
            assertEquals(0, resultSet.getInt(1));
            resultSet.close();
        } finally {
            // 恢复自动提交
            connection.setAutoCommit(true);
        }

        // 清理测试表
        executeUpdate("DROP TABLE test_transaction");
    }
}
