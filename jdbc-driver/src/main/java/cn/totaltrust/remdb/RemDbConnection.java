package cn.totaltrust.remdb;

import java.sql.*;
import java.util.*;
import java.util.concurrent.Executor;
import java.io.*;
import java.net.Socket;

public class RemDbConnection implements Connection {
    private Socket socket;
    private PrintWriter writer;
    private BufferedReader reader;
    private boolean closed = false;

    public RemDbConnection(String host, int port, String user, String password) throws SQLException {
        try {
            this.socket = new Socket(host, port);
            this.writer = new PrintWriter(socket.getOutputStream(), true);
            this.reader = new BufferedReader(new InputStreamReader(socket.getInputStream()));
        } catch (IOException e) {
            throw new SQLException("Failed to connect to RemDb server: " + e.getMessage(), e);
        }
    }

    @Override
    public Statement createStatement() throws SQLException {
        checkClosed();
        return new RemDbStatement(this);
    }

    @Override
    public PreparedStatement prepareStatement(String sql) throws SQLException {
        checkClosed();
        return new RemDbPreparedStatement(this, sql);
    }

    @Override
    public CallableStatement prepareCall(String sql) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("prepareCall not supported");
    }

    @Override
    public String nativeSQL(String sql) throws SQLException {
        checkClosed();
        return sql; // No translation needed
    }

    @Override
    public void setAutoCommit(boolean autoCommit) throws SQLException {
        checkClosed();
        // Auto-commit is always true in RemDb
    }

    @Override
    public boolean getAutoCommit() throws SQLException {
        checkClosed();
        return true; // Auto-commit is always true in RemDb
    }

    @Override
    public void commit() throws SQLException {
        checkClosed();
        // Commits are not supported in RemDb
    }

    @Override
    public void rollback() throws SQLException {
        checkClosed();
        // Rollbacks are not supported in RemDb
    }

    @Override
    public void close() throws SQLException {
        if (!closed) {
            try {
                writer.println("CLOSE");
                reader.close();
                writer.close();
                socket.close();
                closed = true;
            } catch (IOException e) {
                throw new SQLException("Failed to close connection: " + e.getMessage(), e);
            }
        }
    }

    @Override
    public boolean isClosed() throws SQLException {
        return closed;
    }

    @Override
    public DatabaseMetaData getMetaData() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getMetaData not supported");
    }

    @Override
    public void setReadOnly(boolean readOnly) throws SQLException {
        checkClosed();
        // Read-only mode is not supported in RemDb
    }

    @Override
    public boolean isReadOnly() throws SQLException {
        checkClosed();
        return false; // Always read-write
    }

    @Override
    public void setCatalog(String catalog) throws SQLException {
        checkClosed();
        // Catalogs are not supported in RemDb
    }

    @Override
    public String getCatalog() throws SQLException {
        checkClosed();
        return ""; // No catalogs
    }

    @Override
    public void setTransactionIsolation(int level) throws SQLException {
        checkClosed();
        // Transaction isolation is not supported in RemDb
    }

    @Override
    public int getTransactionIsolation() throws SQLException {
        checkClosed();
        return TRANSACTION_NONE; // No transactions
    }

    @Override
    public SQLWarning getWarnings() throws SQLException {
        checkClosed();
        return null; // No warnings
    }

    @Override
    public void clearWarnings() throws SQLException {
        checkClosed();
        // No warnings to clear
    }

    @Override
    public Statement createStatement(int resultSetType, int resultSetConcurrency) throws SQLException {
        checkClosed();
        return createStatement();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int resultSetType, int resultSetConcurrency) throws SQLException {
        checkClosed();
        return prepareStatement(sql);
    }

    @Override
    public CallableStatement prepareCall(String sql, int resultSetType, int resultSetConcurrency) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("prepareCall not supported");
    }

    @Override
    public Map<String, Class<?>> getTypeMap() throws SQLException {
        checkClosed();
        return new HashMap<>(); // No type map
    }

    @Override
    public void setTypeMap(Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        // Type maps are not supported in RemDb
    }

    @Override
    public void setHoldability(int holdability) throws SQLException {
        checkClosed();
        // Holdability is not supported in RemDb
    }

    @Override
    public int getHoldability() throws SQLException {
        checkClosed();
        return ResultSet.HOLD_CURSORS_OVER_COMMIT; // Default holdability
    }

    @Override
    public Savepoint setSavepoint() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Savepoints not supported");
    }

    @Override
    public Savepoint setSavepoint(String name) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Savepoints not supported");
    }

    @Override
    public void rollback(Savepoint savepoint) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Rollbacks not supported");
    }

    @Override
    public void releaseSavepoint(Savepoint savepoint) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Savepoints not supported");
    }

    @Override
    public Statement createStatement(int resultSetType, int resultSetConcurrency, int resultSetHoldability) throws SQLException {
        checkClosed();
        return createStatement();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int resultSetType, int resultSetConcurrency, int resultSetHoldability) throws SQLException {
        checkClosed();
        return prepareStatement(sql);
    }

    @Override
    public CallableStatement prepareCall(String sql, int resultSetType, int resultSetConcurrency, int resultSetHoldability) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("prepareCall not supported");
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int autoGeneratedKeys) throws SQLException {
        checkClosed();
        return prepareStatement(sql);
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int[] columnIndexes) throws SQLException {
        checkClosed();
        return prepareStatement(sql);
    }

    @Override
    public PreparedStatement prepareStatement(String sql, String[] columnNames) throws SQLException {
        checkClosed();
        return prepareStatement(sql);
    }

    @Override
    public Clob createClob() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Clobs not supported");
    }

    @Override
    public Blob createBlob() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Blobs not supported");
    }

    @Override
    public NClob createNClob() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("NClobs not supported");
    }

    @Override
    public SQLXML createSQLXML() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("SQLXML not supported");
    }

    @Override
    public boolean isValid(int timeout) throws SQLException {
        checkClosed();
        return !closed && socket.isConnected();
    }

    @Override
    public void setClientInfo(String name, String value) throws SQLClientInfoException {
        // Client info not supported
    }

    @Override
    public void setClientInfo(Properties properties) throws SQLClientInfoException {
        // Client info not supported
    }

    @Override
    public String getClientInfo(String name) throws SQLException {
        checkClosed();
        return null; // No client info
    }

    @Override
    public Properties getClientInfo() throws SQLException {
        checkClosed();
        return new Properties(); // No client info
    }

    @Override
    public Array createArrayOf(String typeName, Object[] elements) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Arrays not supported");
    }

    @Override
    public Struct createStruct(String typeName, Object[] attributes) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Structs not supported");
    }

    @Override
    public void setSchema(String schema) throws SQLException {
        checkClosed();
        // Schemas are not supported in RemDb
    }

    @Override
    public String getSchema() throws SQLException {
        checkClosed();
        return ""; // No schema
    }

    @Override
    public void abort(Executor executor) throws SQLException {
        checkClosed();
        try {
            socket.close();
            closed = true;
        } catch (IOException e) {
            throw new SQLException("Failed to abort connection: " + e.getMessage(), e);
        }
    }

    @Override
    public void setNetworkTimeout(Executor executor, int milliseconds) throws SQLException {
        checkClosed();
        // Network timeout not supported
    }

    @Override
    public int getNetworkTimeout() throws SQLException {
        checkClosed();
        return 0; // No network timeout
    }

    @Override
    public <T> T unwrap(Class<T> iface) throws SQLException {
        checkClosed();
        if (iface.isInstance(this)) {
            return iface.cast(this);
        }
        throw new SQLException("Cannot unwrap to " + iface.getName());
    }

    @Override
    public boolean isWrapperFor(Class<?> iface) throws SQLException {
        checkClosed();
        return iface.isInstance(this);
    }

    private void checkClosed() throws SQLException {
        if (closed) {
            throw new SQLException("Connection is closed");
        }
    }

    // Internal method for executing SQL commands
    String executeCommand(String command) throws SQLException {
        checkClosed();
        try {
            // Set socket timeout to 15 seconds to prevent infinite blocking
            socket.setSoTimeout(15000);
            writer.println(command);
            return reader.readLine();
        } catch (java.net.SocketTimeoutException e) {
            throw new SQLException("Command timed out. Is the RemDb server running?", e);
        } catch (IOException e) {
            throw new SQLException("Failed to execute command: " + e.getMessage(), e);
        }
    }
}