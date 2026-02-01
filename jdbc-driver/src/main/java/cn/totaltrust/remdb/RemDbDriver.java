package cn.totaltrust.remdb;

import java.sql.*;
import java.util.Properties;
import java.util.logging.Logger;

public class RemDbDriver implements Driver {
    static {
        try {
            DriverManager.registerDriver(new RemDbDriver());
        } catch (SQLException e) {
            throw new RuntimeException("Failed to register RemDbDriver", e);
        }
    }

    @Override
    public Connection connect(String url, Properties info) throws SQLException {
        if (!acceptsURL(url)) {
            return null;
        }

        // Parse URL: jdbc:remdb://host:port/database
        String host = "localhost";
        int port = 6666; // Default JDBC port
        String database = "default"; // Default database name

        // Simple URL parsing without regex
        // "jdbc:remdb://" is 13 characters long
        String urlPart = url.substring(13);
        
        // Get host, port, and database
        int colonIndex = urlPart.indexOf(":");
        if (colonIndex != -1) {
            host = urlPart.substring(0, colonIndex);
            
            // Get port and possibly database
            String portAndDatabase = urlPart.substring(colonIndex + 1);
            int slashIndex = portAndDatabase.indexOf("/");
            
            if (slashIndex != -1) {
                // Extract port
                String portStr = portAndDatabase.substring(0, slashIndex);
                port = Integer.parseInt(portStr);
                
                // Extract database name
                database = portAndDatabase.substring(slashIndex + 1);
            } else {
                // No database specified
                String portStr = portAndDatabase;
                port = Integer.parseInt(portStr);
            }
        } else {
            host = urlPart;
        }

        String user = info.getProperty("user", "");
        String password = info.getProperty("password", "");

        return new RemDbConnection(host, port, user, password, database);
    }

    @Override
    public boolean acceptsURL(String url) throws SQLException {
        return url != null && url.startsWith("jdbc:remdb://");
    }

    @Override
    public DriverPropertyInfo[] getPropertyInfo(String url, Properties info) throws SQLException {
        return new DriverPropertyInfo[0];
    }

    @Override
    public int getMajorVersion() {
        return 1;
    }

    @Override
    public int getMinorVersion() {
        return 0;
    }

    @Override
    public boolean jdbcCompliant() {
        return false;
    }

    @Override
    public Logger getParentLogger() throws SQLFeatureNotSupportedException {
        throw new SQLFeatureNotSupportedException("getParentLogger not supported");
    }
}