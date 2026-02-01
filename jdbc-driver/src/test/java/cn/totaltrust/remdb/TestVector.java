package cn.totaltrust.remdb;

import org.junit.Test;

import java.sql.SQLException;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class TestVector extends TestBase {

    @Test
    public void testVectorDataType() throws SQLException {
        // 创建包含向量类型的表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_vectors (id INTEGER PRIMARY KEY AUTOINCREMENT, vec VECTOR(3) WITH DISTANCE=L2, meta TEXT)");

        // 插入向量数据
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(1.0, 2.0, 3.0), 'vector1')");
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(4.0, 5.0, 6.0), 'vector2')");

        // 验证数据插入成功
        var resultSet = executeQuery("SELECT COUNT(*) FROM test_vectors");
        resultSet.next();
        assertEquals(2, resultSet.getInt(1));
        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE test_vectors");
    }

    @Test
    public void testVectorL2Operator() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_vectors (id INTEGER PRIMARY KEY AUTOINCREMENT, vec VECTOR(3) WITH DISTANCE=L2, meta TEXT)");

        // 插入测试数据
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(1.0, 2.0, 3.0), 'vector1')");
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(4.0, 5.0, 6.0), 'vector2')");

        // 测试L2距离运算符
        var resultSet = executeQuery("SELECT meta, vec <-> VECTOR(1.0, 2.0, 3.0) AS distance FROM test_vectors ORDER BY distance");

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals("vector1", resultSet.getString("meta"));
        assertTrue(resultSet.getDouble("distance") < 0.001); // 应该接近0

        assertTrue(resultSet.next());
        assertEquals("vector2", resultSet.getString("meta"));
        double distance = resultSet.getDouble("distance");
        assertTrue(distance > 5.0 && distance < 6.0); // 预期距离约为5.196

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE test_vectors");
    }

    @Test
    public void testVectorInnerProductOperator() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_vectors (id INTEGER PRIMARY KEY AUTOINCREMENT, vec VECTOR(3) WITH DISTANCE=IP, meta TEXT)");

        // 插入测试数据
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(1.0, 2.0, 3.0), 'vector1')");
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(4.0, 5.0, 6.0), 'vector2')");

        // 测试内积运算符
        var resultSet = executeQuery("SELECT meta, vec <#> VECTOR(1.0, 2.0, 3.0) AS ip FROM test_vectors ORDER BY ip DESC");

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals("vector1", resultSet.getString("meta"));
        assertEquals(14.0, resultSet.getDouble("ip"), 0.001); // 1*1 + 2*2 + 3*3 = 14

        assertTrue(resultSet.next());
        assertEquals("vector2", resultSet.getString("meta"));
        assertEquals(32.0, resultSet.getDouble("ip"), 0.001); // 4*1 + 5*2 + 6*3 = 32

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE test_vectors");
    }

    @Test
    public void testVectorCosineOperator() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_vectors (id INTEGER PRIMARY KEY AUTOINCREMENT, vec VECTOR(3) WITH DISTANCE=COSINE, meta TEXT)");

        // 插入测试数据
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(1.0, 0.0, 0.0), 'vector1')");
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(0.0, 1.0, 0.0), 'vector2')");

        // 测试余弦相似度运算符
        var resultSet = executeQuery("SELECT meta, vec <=> VECTOR(1.0, 0.0, 0.0) AS cosine FROM test_vectors ORDER BY cosine DESC");

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals("vector1", resultSet.getString("meta"));
        assertEquals(1.0, resultSet.getDouble("cosine"), 0.001); // 完全相同

        assertTrue(resultSet.next());
        assertEquals("vector2", resultSet.getString("meta"));
        assertEquals(0.0, resultSet.getDouble("cosine"), 0.001); // 正交

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE test_vectors");
    }

    @Test
    public void testVectorSearch() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_vectors (id INTEGER PRIMARY KEY AUTOINCREMENT, vec VECTOR(3) WITH DISTANCE=L2, meta TEXT)");

        // 插入测试数据
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(1.0, 2.0, 3.0), 'vector1')");
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(1.1, 2.1, 3.1), 'vector2')");
        executeUpdate("INSERT INTO test_vectors (vec, meta) VALUES (VECTOR(4.0, 5.0, 6.0), 'vector3')");

        // 测试向量搜索 - 查找最近的2个向量
        var resultSet = executeQuery("SELECT meta, vec <-> VECTOR(1.0, 2.0, 3.0) AS distance FROM test_vectors ORDER BY distance LIMIT 2");

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals("vector1", resultSet.getString("meta"));
        assertTrue(resultSet.getDouble("distance") < 0.001);

        assertTrue(resultSet.next());
        assertEquals("vector2", resultSet.getString("meta"));
        assertTrue(resultSet.getDouble("distance") < 0.5); // 应该很近

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE test_vectors");
    }

    @Test
    public void testVectorHybridSearch() throws SQLException {
        // 创建测试表
        executeUpdate("CREATE TABLE IF NOT EXISTS test_vectors (id INTEGER PRIMARY KEY AUTOINCREMENT, vec VECTOR(3) WITH DISTANCE=L2, category TEXT, meta TEXT)");

        // 插入测试数据
        executeUpdate("INSERT INTO test_vectors (vec, category, meta) VALUES (VECTOR(1.0, 2.0, 3.0), 'A', 'vector1')");
        executeUpdate("INSERT INTO test_vectors (vec, category, meta) VALUES (VECTOR(1.1, 2.1, 3.1), 'A', 'vector2')");
        executeUpdate("INSERT INTO test_vectors (vec, category, meta) VALUES (VECTOR(4.0, 5.0, 6.0), 'B', 'vector3')");

        // 测试混合搜索 - 查找category为'A'且向量接近的向量
        var resultSet = executeQuery("SELECT meta, vec <-> VECTOR(1.0, 2.0, 3.0) AS distance FROM test_vectors WHERE category = 'A' ORDER BY distance");

        // 验证结果
        assertTrue(resultSet.next());
        assertEquals("vector1", resultSet.getString("meta"));

        assertTrue(resultSet.next());
        assertEquals("vector2", resultSet.getString("meta"));

        // 应该没有更多结果
        assertTrue(!resultSet.next());

        resultSet.close();

        // 清理测试表
        executeUpdate("DROP TABLE test_vectors");
    }
}
