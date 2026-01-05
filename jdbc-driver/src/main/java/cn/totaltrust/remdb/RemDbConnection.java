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
    private boolean autoCommit = true;
    private boolean inTransaction = false;
    private int transactionIsolation = TRANSACTION_READ_COMMITTED;

    public RemDbConnection(String host, int port, String user, String password) throws SQLException {
        try {
            this.socket = new Socket(host, port);
            this.writer = new PrintWriter(socket.getOutputStream(), true);
            this.reader = new BufferedReader(new InputStreamReader(socket.getInputStream()));
            
            // Send AUTH command if username and password are provided and not empty
            if (user != null && !user.isEmpty() && password != null && !password.isEmpty()) {
                String authCommand = "AUTH|" + user + "|" + password;
                writer.println(authCommand);
                String response = reader.readLine();
                if (response == null || response.startsWith("ERROR|")) {
                    throw new SQLException("Authentication failed: " + (response != null ? response.substring(6) : "No response"));
                }
            }
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
        if (this.autoCommit == autoCommit) {
            return;
        }
        
        this.autoCommit = autoCommit;
        
        if (autoCommit) {
            // If we were in a transaction, commit it
            if (inTransaction) {
                commit();
            }
        } else {
            // Start a new transaction when autoCommit is set to false
            beginTransaction();
        }
    }

    @Override
    public boolean getAutoCommit() throws SQLException {
        checkClosed();
        return autoCommit;
    }

    @Override
    public void commit() throws SQLException {
        checkClosed();
        if (!inTransaction) {
            return;
        }
        
        // Call commit command
        String response = executeCommand("COMMIT");
        if (response.startsWith("ERROR|")) {
            throw new SQLException("Commit failed: " + response.substring(6));
        }
        
        inTransaction = false;
    }

    @Override
    public void rollback() throws SQLException {
        checkClosed();
        if (!inTransaction) {
            return;
        }
        
        // Call rollback command
        String response = executeCommand("ROLLBACK");
        if (response.startsWith("ERROR|")) {
            throw new SQLException("Rollback failed: " + response.substring(6));
        }
        
        inTransaction = false;
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
        // RemDb only supports read-write transactions for now
        if (readOnly) {
            throw new SQLFeatureNotSupportedException("Read-only transactions are not supported");
        }
    }

    @Override
    public boolean isReadOnly() throws SQLException {
        checkClosed();
        return false; // Always read-write for now
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
        // RemDb only supports READ_COMMITTED isolation level
        if (level != TRANSACTION_READ_COMMITTED) {
            throw new SQLFeatureNotSupportedException("Only READ_COMMITTED isolation level is supported");
        }
        this.transactionIsolation = level;
    }

    @Override
    public int getTransactionIsolation() throws SQLException {
        checkClosed();
        return transactionIsolation;
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
    
    // Helper method to begin a transaction
    private void beginTransaction() throws SQLException {
        if (inTransaction) {
            return;
        }
        
        // Call begin command directly
        String response = executeCommand("BEGIN");
        if (response.startsWith("ERROR|")) {
            throw new SQLException("Begin transaction failed: " + response.substring(6));
        }
        
        inTransaction = true;
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
    
    // 时序数据库特定方法
    
    /**
     * 创建时序表
     * @param tableName 表名
     * @param timeField 时间字段名
     * @param valueField 值字段名
     * @param tagFields 标签字段名列表
     * @throws SQLException 如果创建失败
     */
    public void createTimeSeriesTable(String tableName, String timeField, String valueField, String... tagFields) throws SQLException {
        checkClosed();
        
        StringBuilder sql = new StringBuilder();
        sql.append("CREATE TIMESERIES TABLE ").append(tableName).append(" (");
        sql.append(timeField).append(" TIMESTAMP, ");
        sql.append(valueField).append(" FLOAT64");
        
        for (String tagField : tagFields) {
            sql.append(", ").append(tagField).append(" TEXT");
        }
        
        sql.append(")");
        
        Statement stmt = createStatement();
        stmt.executeUpdate(sql.toString());
        stmt.close();
    }
    
    /**
     * 批量写入时序数据
     * @param tableName 表名
     * @param records 时序数据记录列表，格式为：时间戳,值,标签1,标签2,...
     * @throws SQLException 如果写入失败
     */
    public void writeTimeSeriesBatch(String tableName, List<Object[]> records) throws SQLException {
        checkClosed();
        
        if (records.isEmpty()) {
            return;
        }
        
        // 构建批量INSERT语句
        StringBuilder sql = new StringBuilder();
        sql.append("INSERT INTO ").append(tableName).append(" VALUES ");
        
        for (int i = 0; i < records.size(); i++) {
            Object[] record = records.get(i);
            if (i > 0) {
                sql.append(",");
            }
            sql.append("(");
            
            for (int j = 0; j < record.length; j++) {
                if (j > 0) {
                    sql.append(",");
                }
                
                Object value = record[j];
                if (value == null) {
                    sql.append("NULL");
                } else if (value instanceof String) {
                    sql.append("'").append(((String) value).replace("'", "''")).append("'");
                } else if (value instanceof Timestamp) {
                    sql.append(((Timestamp) value).getTime());
                } else {
                    sql.append(value.toString());
                }
            }
            sql.append(")");
        }
        
        Statement stmt = createStatement();
        stmt.executeUpdate(sql.toString());
        stmt.close();
    }
    
    /**
     * 查询指定时间范围内的时序数据
     * @param tableName 表名
     * @param startTime 开始时间戳（毫秒）
     * @param endTime 结束时间戳（毫秒）
     * @return 结果集
     * @throws SQLException 如果查询失败
     */
    public ResultSet queryTimeSeries(String tableName, long startTime, long endTime) throws SQLException {
        checkClosed();
        
        String sql = String.format("SELECT * FROM %s WHERE timestamp BETWEEN %d AND %d ORDER BY timestamp", 
                                  tableName, startTime, endTime);
        
        Statement stmt = createStatement();
        return stmt.executeQuery(sql);
    }
    
    /**
     * 查询指定标签的时序数据
     * @param tableName 表名
     * @param tagName 标签名
     * @param tagValue 标签值
     * @param limit 返回记录数限制
     * @return 结果集
     * @throws SQLException 如果查询失败
     */
    public ResultSet queryTimeSeriesByTag(String tableName, String tagName, String tagValue, int limit) throws SQLException {
        checkClosed();
        
        String sql = String.format("SELECT * FROM %s WHERE %s = '%s' ORDER BY timestamp DESC LIMIT %d", 
                                  tableName, tagName, tagValue.replace("'", "''"), limit);
        
        Statement stmt = createStatement();
        return stmt.executeQuery(sql);
    }
}