package cn.totaltrust.remdb;

import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.Statement;

/**
 * JDBC驱动向量功能测试示例
 */
public class RemDbVectorExample {
    public static void main(String[] args) {
        try {
            // 加载JDBC驱动
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            
            // 建立数据库连接
            String url = "jdbc:remdb://localhost:6666";
            String user = "root";
            String password = "admin";
            
            System.out.println("正在连接到RemDB数据库...");
            System.out.println("注意：请确保RemDB服务器已在端口6666上启动！");
            
            Connection conn = DriverManager.getConnection(url, user, password);
            System.out.println("连接成功！");
            
            Statement stmt = conn.createStatement();
            
            // 1. 创建包含向量字段的表
            System.out.println("\n1. 创建包含向量字段的表...");
            String createTableSql = "CREATE TABLE products (" +
                    "id INT32 PRIMARY KEY, " +
                    "name TEXT, " +
                    "embedding VECTOR(4) WITH DISTANCE=IP" +
                    ")";
            stmt.executeUpdate(createTableSql);
            System.out.println("表创建成功！");
            
            // 2. 插入向量数据
            System.out.println("\n2. 插入向量数据...");
            String insertSql = "INSERT INTO products (id, name, embedding) VALUES " +
                    "(1, 'product1', '[0.1, 0.2, 0.3, 0.4]'), " +
                    "(2, 'product2', '[1.0, 0.9, 0.8, 0.7]'), " +
                    "(3, 'product3', '[0.5, 0.5, 0.5, 0.5]')";
            int rowsAffected = stmt.executeUpdate(insertSql);
            System.out.println("成功插入 " + rowsAffected + " 行数据！");
            
            // 3. 查询所有数据
            System.out.println("\n3. 查询所有数据...");
            String selectSql = "SELECT id, name, embedding FROM products";
            ResultSet rs = stmt.executeQuery(selectSql);
            
            while (rs.next()) {
                int id = rs.getInt("id");
                String name = rs.getString("name");
                String embeddingStr = rs.getString("embedding");
                
                // 测试向量类型转换
                float[] embeddingFloat = rs.getObject("embedding", float[].class);
                double[] embeddingDouble = rs.getObject("embedding", double[].class);
                
                System.out.printf("ID: %d, Name: %s, Embedding: %s%n", id, name, embeddingStr);
                System.out.printf("  -> float[]: %s%n", arrayToString(embeddingFloat));
                System.out.printf("  -> double[]: %s%n", arrayToString(embeddingDouble));
            }
            rs.close();
            
            // 4. 向量相似性查询
            System.out.println("\n4. 向量相似性查询（内积距离）...");
            String vectorQuerySql = "SELECT id, name, embedding <#> '[0.2, 0.3, 0.4, 0.5]' AS similarity FROM products ORDER BY similarity DESC LIMIT 2";
            ResultSet vectorRs = stmt.executeQuery(vectorQuerySql);
            
            while (vectorRs.next()) {
                int id = vectorRs.getInt("id");
                String name = vectorRs.getString("name");
                double similarity = vectorRs.getDouble("similarity");
                
                System.out.printf("ID: %d, Name: %s, Similarity: %.4f%n", id, name, similarity);
            }
            vectorRs.close();
            
            // 5. 创建向量索引
            System.out.println("\n5. 创建向量索引...");
            String createIndexSql = "CREATE INDEX idx_products_embedding ON products (embedding) USING HNSW WITH (M=16, ef_construction=200)";
            stmt.executeUpdate(createIndexSql);
            System.out.println("向量索引创建成功！");
            
            // 6. 使用向量索引进行查询
            System.out.println("\n6. 使用向量索引进行相似性查询...");
            ResultSet indexedQueryRs = stmt.executeQuery(vectorQuerySql);
            
            while (indexedQueryRs.next()) {
                int id = indexedQueryRs.getInt("id");
                String name = indexedQueryRs.getString("name");
                double similarity = indexedQueryRs.getDouble("similarity");
                
                System.out.printf("ID: %d, Name: %s, Similarity: %.4f%n", id, name, similarity);
            }
            indexedQueryRs.close();
            
            // 7. 清理测试数据
            System.out.println("\n7. 清理测试数据...");
            stmt.executeUpdate("DROP TABLE products");
            System.out.println("测试数据清理完成！");
            
            // 关闭资源
            stmt.close();
            conn.close();
            
            System.out.println("\n所有测试完成！JDBC驱动向量功能正常工作。");
            
        } catch (ClassNotFoundException e) {
            System.err.println("错误：未找到JDBC驱动类！");
            e.printStackTrace();
        } catch (java.sql.SQLException e) {
            if (e.getMessage().contains("Connection refused") || e.getMessage().contains("getsockopt")) {
                System.err.println("错误：无法连接到RemDB服务器！");
                System.err.println("请确保：");
                System.err.println("1. RemDB服务器已启动");
                System.err.println("2. 服务器正在端口6666上运行");
                System.err.println("3. 服务器允许来自本地的连接");
            } else {
                System.err.println("SQL错误：" + e.getMessage());
            }
            e.printStackTrace();
        } catch (Exception e) {
            System.err.println("其他错误：" + e.getMessage());
            e.printStackTrace();
        }
    }
    
    /**
     * 将float数组转换为字符串
     */
    private static String arrayToString(float[] array) {
        if (array == null) {
            return "null";
        }
        StringBuilder sb = new StringBuilder();
        sb.append("[");
        for (int i = 0; i < array.length; i++) {
            if (i > 0) {
                sb.append(", ");
            }
            sb.append(String.format("%.4f", array[i]));
        }
        sb.append("]");
        return sb.toString();
    }
    
    /**
     * 将double数组转换为字符串
     */
    private static String arrayToString(double[] array) {
        if (array == null) {
            return "null";
        }
        StringBuilder sb = new StringBuilder();
        sb.append("[");
        for (int i = 0; i < array.length; i++) {
            if (i > 0) {
                sb.append(", ");
            }
            sb.append(String.format("%.4f", array[i]));
        }
        sb.append("]");
        return sb.toString();
    }
}