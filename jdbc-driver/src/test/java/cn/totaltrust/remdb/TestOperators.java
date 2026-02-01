package cn.totaltrust.remdb;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import java.sql.*;

public class TestOperators extends TestBase {

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
        executeSql("CREATE TABLE IF NOT EXISTS test_operators_users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, email TEXT)");
        // 创建向量表
        executeSql("CREATE TABLE IF NOT EXISTS test_operators_vectors (id INTEGER PRIMARY KEY AUTOINCREMENT, vec VECTOR(3) WITH DISTANCE=L2, meta TEXT)");
    }

    /**
     * 插入测试数据
     * @throws SQLException 如果插入失败
     */
    private void insertTestData() throws SQLException {
        // 插入用户数据
        executeBatch(new String[]{
            "INSERT INTO test_operators_users (name, age, email) VALUES ('Alice', 25, 'alice@example.com')",
            "INSERT INTO test_operators_users (name, age, email) VALUES ('Bob', 30, 'bob@example.com')",
            "INSERT INTO test_operators_users (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')",
            "INSERT INTO test_operators_users (name, age, email) VALUES ('David', 20, 'david@example.com')",
            "INSERT INTO test_operators_users (name, age, email) VALUES ('Eve', 40, 'eve@example.com')"
        });

        // 插入向量数据
        executeBatch(new String[]{
            "INSERT INTO test_operators_vectors (vec, meta) VALUES ([1.0, 2.0, 3.0], 'Vector 1')",
            "INSERT INTO test_operators_vectors (vec, meta) VALUES ([4.0, 5.0, 6.0], 'Vector 2')",
            "INSERT INTO test_operators_vectors (vec, meta) VALUES ([7.0, 8.0, 9.0], 'Vector 3')",
            "INSERT INTO test_operators_vectors (vec, meta) VALUES ([1.1, 2.1, 3.1], 'Vector 4')" // 与Vector 1接近
        });
    }

    /**
     * 测试比较运算符
     */
    @Test
    public void testComparisonOperators() throws SQLException {
        // 测试 = 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age = 30", 1);

        // 测试 <> 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age <> 30", 4);

        // 测试 != 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age != 30", 4);

        // 测试 > 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age > 30", 2);

        // 测试 >= 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age >= 30", 3);

        // 测试 < 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age < 30", 2);

        // 测试 <= 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age <= 30", 3);
    }

    /**
     * 测试 LIKE 运算符
     */
    @Test
    public void testLikeOperator() throws SQLException {
        // 测试 LIKE 带 % 通配符
        verifyRowCount("SELECT * FROM test_operators_users WHERE name LIKE 'A%'", 1);
        verifyRowCount("SELECT * FROM test_operators_users WHERE name LIKE '%e'", 2); // Alice, Eve
        verifyRowCount("SELECT * FROM test_operators_users WHERE name LIKE '%ob%'", 1); // Bob

        // 测试 LIKE 带 _ 通配符
        verifyRowCount("SELECT * FROM test_operators_users WHERE name LIKE 'A_i_e'", 1); // Alice
        verifyRowCount("SELECT * FROM test_operators_users WHERE name LIKE 'B_b'", 0); // 应该没有匹配

        // 测试 LIKE 不带通配符（相当于 =）
        verifyRowCount("SELECT * FROM test_operators_users WHERE name LIKE 'Bob'", 1);
    }

    /**
     * 测试逻辑运算符
     */
    @Test
    public void testLogicalOperators() throws SQLException {
        // 测试 AND 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age > 25 AND age < 35", 1); // Bob

        // 测试 OR 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age < 25 OR age > 35", 2); // David, Eve

        // 测试 AND 与 OR 组合
        verifyRowCount("SELECT * FROM test_operators_users WHERE (age > 25 AND age < 35) OR name = 'Alice'", 2); // Bob, Alice
    }

    /**
     * 测试向量距离运算符 - L2距离
     */
    @Test
    public void testVectorL2Operator() throws SQLException {
        // 测试 L2 距离运算符
        ResultSet rs = executeQuery("SELECT meta, vec <-> [1.0, 2.0, 3.0] AS distance FROM test_operators_vectors ORDER BY distance ASC");
        int count = 0;
        boolean foundVector1 = false;
        boolean foundVector4 = false;
        while (rs.next()) {
            String meta = rs.getString("meta");
            double distance = rs.getDouble("distance");
            if (meta.equals("Vector 1")) {
                assert distance < 0.1; // 与自身的距离应该接近0
                foundVector1 = true;
            } else if (meta.equals("Vector 4")) {
                assert distance < 0.5; // 与Vector 1接近
                foundVector4 = true;
            }
            count++;
        }
        rs.close();
        assert count == 4;
        assert foundVector1;
        assert foundVector4;
    }

    /**
     * 测试向量距离运算符 - 内积
     */
    @Test
    public void testVectorIPOperator() throws SQLException {
        // 测试内积运算符
        ResultSet rs = executeQuery("SELECT meta, vec <#> [1.0, 1.0, 1.0] AS dot_product FROM test_operators_vectors ORDER BY dot_product DESC");
        int count = 0;
        double previousDotProduct = Double.MIN_VALUE;
        while (rs.next()) {
            String meta = rs.getString("meta");
            double dotProduct = rs.getDouble("dot_product");
            assert dotProduct >= previousDotProduct;
            previousDotProduct = dotProduct;
            count++;
        }
        rs.close();
        assert count == 4;
    }

    /**
     * 测试向量距离运算符 - 余弦相似度
     */
    @Test
    public void testVectorCosineOperator() throws SQLException {
        // 测试余弦相似度运算符
        ResultSet rs = executeQuery("SELECT meta, vec <=> [1.0, 2.0, 3.0] AS cosine FROM test_operators_vectors ORDER BY cosine DESC");
        int count = 0;
        boolean foundVector1 = false;
        boolean foundVector4 = false;
        while (rs.next()) {
            String meta = rs.getString("meta");
            double cosine = rs.getDouble("cosine");
            if (meta.equals("Vector 1")) {
                assert cosine > 0.999; // 与自身的余弦相似度应该接近1
                foundVector1 = true;
            } else if (meta.equals("Vector 4")) {
                assert cosine > 0.99; // 与Vector 1接近
                foundVector4 = true;
            }
            count++;
        }
        rs.close();
        assert count == 4;
        assert foundVector1;
        assert foundVector4;
    }

    /**
     * 测试运算符的组合使用
     */
    @Test
    public void testOperatorCombination() throws SQLException {
        // 测试比较运算符与逻辑运算符的组合
        verifyRowCount("SELECT * FROM test_operators_users WHERE (age > 25 AND age < 35) OR (name = 'David' AND age < 25)", 2); // Bob, David

        // 测试 LIKE 与逻辑运算符的组合
        verifyRowCount("SELECT * FROM test_operators_users WHERE name LIKE 'A%' OR (name LIKE 'B%' AND age > 25)", 2); // Alice, Bob

        // 测试向量运算符与 WHERE 条件的组合
        ResultSet rs = executeQuery("SELECT meta FROM test_operators_vectors WHERE vec <-> [1.0, 2.0, 3.0] < 0.5");
        int count = 0;
        while (rs.next()) {
            String meta = rs.getString("meta");
            assert meta.equals("Vector 1") || meta.equals("Vector 4");
            count++;
        }
        rs.close();
        assert count == 2;
    }

    /**
     * 测试 BETWEEN 运算符
     */
    @Test
    public void testBetweenOperator() throws SQLException {
        // 测试 BETWEEN 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age BETWEEN 25 AND 35", 3); // Alice, Bob, Charlie

        // 测试 NOT BETWEEN 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age NOT BETWEEN 25 AND 35", 2); // David, Eve
    }

    /**
     * 测试 IN 运算符
     */
    @Test
    public void testInOperator() throws SQLException {
        // 测试 IN 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age IN (25, 30, 35)", 3); // Alice, Bob, Charlie

        // 测试 NOT IN 运算符
        verifyRowCount("SELECT * FROM test_operators_users WHERE age NOT IN (25, 30, 35)", 2); // David, Eve

        // 测试 IN 运算符与字符串
        verifyRowCount("SELECT * FROM test_operators_users WHERE name IN ('Alice', 'Bob', 'Charlie')", 3);
    }
}
