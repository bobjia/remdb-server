package cn.totaltrust.remdb;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import java.sql.*;

public class TestIndex extends TestBase {

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
        executeSql("CREATE TABLE IF NOT EXISTS test_index_users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, email TEXT)");
        // 创建向量表
        executeSql("CREATE TABLE IF NOT EXISTS test_index_vectors (id INTEGER PRIMARY KEY AUTOINCREMENT, vec VECTOR(3) WITH DISTANCE=L2, meta TEXT)");
    }

    /**
     * 插入测试数据
     * @throws SQLException 如果插入失败
     */
    private void insertTestData() throws SQLException {
        // 插入用户数据
        executeBatch(new String[]{
            "INSERT INTO test_index_users (name, age, email) VALUES ('Alice', 25, 'alice@example.com')",
            "INSERT INTO test_index_users (name, age, email) VALUES ('Bob', 30, 'bob@example.com')",
            "INSERT INTO test_index_users (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')",
            "INSERT INTO test_index_users (name, age, email) VALUES ('David', 20, 'david@example.com')",
            "INSERT INTO test_index_users (name, age, email) VALUES ('Eve', 40, 'eve@example.com')",
            "INSERT INTO test_index_users (name, age, email) VALUES ('Frank', 28, 'frank@example.com')",
            "INSERT INTO test_index_users (name, age, email) VALUES ('Grace', 32, 'grace@example.com')",
            "INSERT INTO test_index_users (name, age, email) VALUES ('Henry', 38, 'henry@example.com')"
        });

        // 插入向量数据
        executeBatch(new String[]{
            "INSERT INTO test_index_vectors (vec, meta) VALUES ([1.0, 2.0, 3.0], 'Vector 1')",
            "INSERT INTO test_index_vectors (vec, meta) VALUES ([4.0, 5.0, 6.0], 'Vector 2')",
            "INSERT INTO test_index_vectors (vec, meta) VALUES ([7.0, 8.0, 9.0], 'Vector 3')",
            "INSERT INTO test_index_vectors (vec, meta) VALUES ([1.1, 2.1, 3.1], 'Vector 4')",
            "INSERT INTO test_index_vectors (vec, meta) VALUES ([4.1, 5.1, 6.1], 'Vector 5')",
            "INSERT INTO test_index_vectors (vec, meta) VALUES ([7.1, 8.1, 9.1], 'Vector 6')"
        });
    }

    /**
     * 测试创建标量索引
     */
    @Test
    public void testCreateScalarIndex() throws SQLException {
        // 创建年龄列的B-Tree索引
        executeSql("CREATE INDEX idx_test_index_users_age ON test_index_users (age) USING BTREE");

        // 创建姓名列的B-Tree索引
        executeSql("CREATE INDEX idx_test_index_users_name ON test_index_users (name) USING BTREE");

        // 测试索引是否提高查询性能（通过验证查询是否成功执行）
        ResultSet rs = executeQuery("SELECT * FROM test_index_users WHERE age > 30");
        int count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 4; // Charlie, Eve, Grace, Henry

        // 测试使用姓名索引的查询
        rs = executeQuery("SELECT * FROM test_index_users WHERE name LIKE 'A%'");
        count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 1; // Alice
    }

    /**
     * 测试创建向量索引 - HNSW
     */
    @Test
    public void testCreateVectorIndexHNSW() throws SQLException {
        // 创建HNSW向量索引
        executeSql("CREATE INDEX idx_test_index_vectors_vec_hnsw ON test_index_vectors (vec) USING HNSW WITH (M=16, ef_construction=200, ef_search=100, DISTANCE=L2) ONLINE");

        // 测试向量索引查询
        ResultSet rs = executeQuery("SELECT meta, vec <-> [1.0, 2.0, 3.0] AS distance FROM test_index_vectors ORDER BY distance ASC LIMIT 3");
        int count = 0;
        while (rs.next()) {
            String meta = rs.getString("meta");
            double distance = rs.getDouble("distance");
            assert meta != null;
            assert distance >= 0;
            count++;
        }
        rs.close();
        assert count == 3;
    }

    /**
     * 测试创建向量索引 - IVF_FLAT
     */
    @Test
    public void testCreateVectorIndexIVFFlat() throws SQLException {
        // 创建IVF_FLAT向量索引
        executeSql("CREATE INDEX idx_test_index_vectors_vec_ivf_flat ON test_index_vectors (vec) USING IVF_FLAT WITH (nlist=128, nprobe=16, DISTANCE=L2) ONLINE");

        // 测试向量索引查询
        ResultSet rs = executeQuery("SELECT meta, vec <-> [4.0, 5.0, 6.0] AS distance FROM test_index_vectors ORDER BY distance ASC LIMIT 3");
        int count = 0;
        while (rs.next()) {
            String meta = rs.getString("meta");
            double distance = rs.getDouble("distance");
            assert meta != null;
            assert distance >= 0;
            count++;
        }
        rs.close();
        assert count == 3;
    }

    /**
     * 测试创建向量索引 - IVF_PQ
     */
    @Test
    public void testCreateVectorIndexIVFPQ() throws SQLException {
        // 创建IVF_PQ向量索引
        executeSql("CREATE INDEX idx_test_index_vectors_vec_ivf_pq ON test_index_vectors (vec) USING IVF_PQ WITH (nlist=128, nprobe=8, M=8, nbits=8) ONLINE");

        // 测试向量索引查询
        ResultSet rs = executeQuery("SELECT meta, vec <-> [7.0, 8.0, 9.0] AS distance FROM test_index_vectors ORDER BY distance ASC LIMIT 3");
        int count = 0;
        while (rs.next()) {
            String meta = rs.getString("meta");
            double distance = rs.getDouble("distance");
            assert meta != null;
            assert distance >= 0;
            count++;
        }
        rs.close();
        assert count == 3;
    }

    /**
     * 测试索引构建状态监控
     */
    @Test
    public void testIndexBuildStatus() throws SQLException {
        // 创建索引
        executeSql("CREATE INDEX idx_test_index_users_email ON test_index_users (email) USING BTREE");

        // 查看索引构建状态
        ResultSet rs = executeQuery("SHOW INDEX BUILD STATUS");
        int count = 0;
        while (rs.next()) {
            String indexName = rs.getString("index_name");
            String tableName = rs.getString("table_name");
            String status = rs.getString("status");
            assert indexName != null;
            assert tableName != null;
            assert status != null;
            count++;
        }
        rs.close();
        assert count >= 1; // 至少有一个索引
    }

    /**
     * 测试索引重建
     */
    @Test
    public void testIndexReindex() throws SQLException {
        // 创建索引
        executeSql("CREATE INDEX idx_test_index_users_age_reindex ON test_index_users (age) USING BTREE");

        // 重建索引
        executeSql("REINDEX idx_test_index_users_age_reindex ONLINE");

        // 测试重建后的索引是否可用
        ResultSet rs = executeQuery("SELECT * FROM test_index_users WHERE age BETWEEN 25 AND 35");
        int count = 0;
        while (rs.next()) {
            count++;
        }
        rs.close();
        assert count == 4; // Alice, Bob, Frank, Grace
    }

    /**
     * 测试索引持久化
     */
    @Test
    public void testIndexPersistence() throws SQLException {
        // 创建带有持久化的索引
        executeSql("CREATE INDEX idx_test_index_users_name_persistent ON test_index_users (name) USING BTREE WITH (STORAGE=DISK)");

        // 测试持久化索引是否可用
        ResultSet rs = executeQuery("SELECT * FROM test_index_users WHERE name = 'Bob'");
        int count = 0;
        while (rs.next()) {
            String name = rs.getString("name");
            assert name.equals("Bob");
            count++;
        }
        rs.close();
        assert count == 1;
    }

    /**
     * 测试复合索引
     */
    @Test
    public void testCompositeIndex() throws SQLException {
        // 创建复合索引
        executeSql("CREATE INDEX idx_test_index_users_name_age ON test_index_users (name, age) USING BTREE");

        // 测试复合索引查询
        ResultSet rs = executeQuery("SELECT * FROM test_index_users WHERE name = 'Alice' AND age = 25");
        int count = 0;
        while (rs.next()) {
            String name = rs.getString("name");
            int age = rs.getInt("age");
            assert name.equals("Alice");
            assert age == 25;
            count++;
        }
        rs.close();
        assert count == 1;
    }

    /**
     * 测试索引与WHERE条件组合
     */
    @Test
    public void testIndexWithWhere() throws SQLException {
        // 创建索引
        executeSql("CREATE INDEX idx_test_index_users_age_where ON test_index_users (age) USING BTREE");

        // 测试索引与复杂WHERE条件
        ResultSet rs = executeQuery("SELECT * FROM test_index_users WHERE age > 25 AND age < 35 AND name LIKE 'G%'");
        int count = 0;
        while (rs.next()) {
            String name = rs.getString("name");
            int age = rs.getInt("age");
            assert name.equals("Grace");
            assert age == 32;
            count++;
        }
        rs.close();
        assert count == 1;
    }
}
