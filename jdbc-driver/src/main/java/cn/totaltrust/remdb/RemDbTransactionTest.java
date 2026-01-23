package cn.totaltrust.remdb;

import java.sql.*;

public class RemDbTransactionTest {
    public static void main(String[] args) {
        String url = "jdbc:remdb://localhost:6666";
        String user = "root";
        String password = "admin";

        Connection conn = null;
        Statement stmt = null;
        ResultSet rs = null;

        try {
            System.out.println("Connecting to RemDb server...");
            // 显式加载RemDb驱动
            Class.forName("cn.totaltrust.remdb.RemDbDriver");
            conn = DriverManager.getConnection(url);
            System.out.println("Connection established successfully!");

            // Test 1: Get initial autoCommit status
            boolean initialAutoCommit = conn.getAutoCommit();
            System.out.println("\nTest 1: Initial autoCommit status: " + initialAutoCommit);
            
            // Test 2: Set autoCommit to false
            System.out.println("\nTest 2: Setting autoCommit to false");
            conn.setAutoCommit(false);
            System.out.println("autoCommit set to false");
            
            // Test 3: Verify autoCommit status
            boolean newAutoCommit = conn.getAutoCommit();
            System.out.println("\nTest 3: Current autoCommit status: " + newAutoCommit);
            
            // Test 4: Commit transaction
            System.out.println("\nTest 4: Committing transaction");
            conn.commit();
            System.out.println("Transaction committed");
            
            // Test 5: Rollback transaction
            System.out.println("\nTest 5: Rolling back transaction");
            conn.rollback();
            System.out.println("Transaction rolled back");
            
            // Test 6: Set autoCommit back to true
            System.out.println("\nTest 6: Setting autoCommit back to true");
            conn.setAutoCommit(true);
            System.out.println("autoCommit set to true");
            
            // Test 7: Verify final autoCommit status
            boolean finalAutoCommit = conn.getAutoCommit();
            System.out.println("\nTest 7: Final autoCommit status: " + finalAutoCommit);
            
            System.out.println("All tests completed successfully!");
            
        } catch (ClassNotFoundException e) {
            System.err.println("Driver class not found: " + e.getMessage());
            e.printStackTrace();
        } catch (SQLException e) {
            System.err.println("SQL Exception: " + e.getMessage());
            e.printStackTrace();
        }
    }
}