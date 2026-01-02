package cn.totaltrust.remdb;

import java.sql.*;

public class RemDbJdbcExample {
    public static void main(String[] args) {
        String url = "jdbc:remdb://localhost:5432";
        String user = "";
        String password = "";

        Connection conn = null;
        Statement stmt = null;
        ResultSet rs = null;

        try {
            System.out.println("Connecting to RemDb server...");
            conn = DriverManager.getConnection(url, user, password);
            System.out.println("Connection established successfully!");

            stmt = conn.createStatement();

            String createTableSQL = "CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY, name VARCHAR(50), age INT)";
            stmt.executeUpdate(createTableSQL);
            System.out.println("\n1. Table 'users' created or already exists");

            String insertSQL = "INSERT INTO users (id, name, age) VALUES (1, 'Alice', 25)";
            int rowsInserted = stmt.executeUpdate(insertSQL);
            System.out.println("\n2. Inserted " + rowsInserted + " row(s)");

            String selectSQL = "SELECT id, name, age FROM users";
            rs = stmt.executeQuery(selectSQL);

            System.out.println("\n3. Query results:");
            System.out.println("ID | Name | Age");
            System.out.println("---|------|----");
            while (rs.next()) {
                int id = rs.getInt("id");
                String name = rs.getString("name");
                int age = rs.getInt("age");
                System.out.printf("%d  | %s    | %d\n", id, name, age);
            }

            String updateSQL = "UPDATE users SET age = 26 WHERE id = 1";
            int rowsUpdated = stmt.executeUpdate(updateSQL);
            System.out.println("\n4. Updated " + rowsUpdated + " row(s)");

            rs = stmt.executeQuery(selectSQL);
            System.out.println("\n5. Updated query results:");
            System.out.println("ID | Name | Age");
            System.out.println("---|------|----");
            while (rs.next()) {
                int id = rs.getInt("id");
                String name = rs.getString("name");
                int age = rs.getInt("age");
                System.out.printf("%d  | %s    | %d\n", id, name, age);
            }

            String deleteSQL = "DELETE FROM users WHERE id = 1";
            int rowsDeleted = stmt.executeUpdate(deleteSQL);
            System.out.println("\n6. Deleted " + rowsDeleted + " row(s)");

        } catch (SQLException e) {
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) rs.close();
                if (stmt != null) stmt.close();
                if (conn != null) conn.close();
                System.out.println("\nResources closed successfully!");
            } catch (SQLException e) {
                e.printStackTrace();
            }
        }
    }
}