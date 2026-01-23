package cn.totaltrust.remdb;

import java.sql.*;
import java.util.*;
import java.util.concurrent.Executor;
import java.io.*;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.channels.SocketChannel;
import java.net.InetSocketAddress;
import java.nio.ByteOrder;
import com.google.protobuf.InvalidProtocolBufferException;
import jdbc.Jdbc;

public class RemDbConnection implements Connection {
    private SocketChannel channel;
    private Socket socket;
    private boolean closed = false;
    private boolean autoCommit = true;
    private boolean inTransaction = false;
    private int transactionIsolation = TRANSACTION_READ_COMMITTED;
    private DirectBufferPool bufferPool;
    private boolean zeroCopyEnabled = true;
    private long requestId = 0;
    private final ByteBuffer lenBuffer = ByteBuffer.allocate(4); // 4字节长度前缀
    private final ByteBuffer recvBuffer = ByteBuffer.allocateDirect(65536); // 64KB接收缓冲区

    public RemDbConnection(String host, int port, String user, String password) throws SQLException {
        try {
            // 创建直接内存缓冲池（16个8KB缓冲区）
            this.bufferPool = new DirectBufferPool(16, 8192);
            
            // 创建Socket
            this.socket = new Socket();
            
            // 设置TCP参数
            socket.setTcpNoDelay(true); // 禁用Nagle算法
            socket.setKeepAlive(true);   // 启用TCP keepalive
            socket.setReuseAddress(true); // 启用地址重用
            socket.setSoTimeout(15000); // 设置读取超时为15秒
            socket.setSoLinger(false, 0); // 禁用SO_LINGER，立即关闭连接
            
            // 设置接收和发送缓冲区大小
            socket.setReceiveBufferSize(65536);
            socket.setSendBufferSize(65536);
            
            // 设置连接超时（5秒）
            socket.connect(new InetSocketAddress(host, port), 5000);
            
            // 创建SocketChannel（仅用于零拷贝，不用于常规IO）
            this.channel = SocketChannel.open(socket.getRemoteSocketAddress());
            this.channel.configureBlocking(true);
            
            // 使用Thread.interrupt()实现超时处理
            initializeConnectionWithTimeout(user, password);
        } catch (java.net.SocketTimeoutException e) {
            throw new SQLException("Failed to connect to RemDb server: Connection timed out. Please check if the server is running on port " + port + ".", e);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new SQLException("Connection interrupted: " + e.getMessage(), e);
        } catch (IOException e) {
            throw new SQLException("Failed to connect to RemDb server: " + e.getMessage(), e);
        }
    }
    
    // 初始化连接，发送连接请求
    private void initializeConnection(String user, String password) throws SQLException, IOException {
        // 构建连接请求
        Jdbc.JdbcRequest request = buildConnectionRequestMessage(user, password);
        
        // 发送请求（包含4字节长度前缀）
        sendJdbcRequest(request);
        
        // 接收响应
        Jdbc.JdbcResponse response = receiveJdbcResponse();
        
        // 处理连接响应
        handleConnectionResponse(response);
    }
    
    // 使用单独的线程实现超时处理的初始化连接方法
    private void initializeConnectionWithTimeout(String user, String password) throws SQLException, IOException, InterruptedException {
        // 创建一个线程来执行初始化连接
        final SQLException[] sqlException = {null};
        final IOException[] ioException = {null};
        
        Thread initThread = new Thread(() -> {
            try {
                initializeConnection(user, password);
            } catch (SQLException e) {
                sqlException[0] = e;
            } catch (IOException e) {
                ioException[0] = e;
            }
        });
        
        // 启动线程
        initThread.start();
        
        // 等待线程执行完成，最多等待15秒
        initThread.join(15000);
        
        // 检查线程是否还在运行
        if (initThread.isAlive()) {
            // 超时，中断线程
            initThread.interrupt();
            throw new java.net.SocketTimeoutException("Connection initialization timed out after 15 seconds");
        }
        
        // 检查是否有异常
        if (sqlException[0] != null) {
            throw sqlException[0];
        }
        if (ioException[0] != null) {
            throw ioException[0];
        }
    }
    
    private long nextRequestId() {
        requestId += 1;
        return requestId;
    }
    
    private void sendJdbcRequest(Jdbc.JdbcRequest request) throws IOException {
        sendRequest(request.toByteArray());
    }
    
    private Jdbc.JdbcResponse receiveJdbcResponse() throws IOException, SQLException {
        byte[] responseData = receiveResponse();
        try {
            return Jdbc.JdbcResponse.parseFrom(responseData);
        } catch (InvalidProtocolBufferException e) {
            throw new SQLException("Invalid JDBC response format: " + e.getMessage(), e);
        }
    }
    
    // 构建连接请求
    private Jdbc.JdbcRequest buildConnectionRequestMessage(String user, String password) {
        Jdbc.ConnectionRequest connection = Jdbc.ConnectionRequest.newBuilder()
            .setUsername(user)
            .setPassword(password)
            .setDatabase("default")
            .setFetchSize(100)
            .setAutoCommit(true)
            .build();

        return Jdbc.JdbcRequest.newBuilder()
            .setRequestId(nextRequestId())
            .setConnection(connection)
            .build();
    }
    
    private Jdbc.JdbcRequest buildQueryRequestMessage(String sql) {
        Jdbc.QueryRequest query = Jdbc.QueryRequest.newBuilder()
            .setSql(sql)
            .setFetchSize(100)
            .setUseCursor(false)
            .build();

        return Jdbc.JdbcRequest.newBuilder()
            .setRequestId(nextRequestId())
            .setQuery(query)
            .build();
    }
    
    private Jdbc.JdbcRequest buildBeginRequestMessage() {
        Jdbc.BeginTransaction begin = Jdbc.BeginTransaction.newBuilder()
            .setType(Jdbc.TransactionType.READ_WRITE)
            .setIsolationLevel(Jdbc.IsolationLevel.READ_COMMITTED)
            .build();

        return Jdbc.JdbcRequest.newBuilder()
            .setRequestId(nextRequestId())
            .setBeginTransaction(begin)
            .build();
    }
    
    private Jdbc.JdbcRequest buildCommitRequestMessage() {
        return Jdbc.JdbcRequest.newBuilder()
            .setRequestId(nextRequestId())
            .setCommitTransaction(Jdbc.CommitTransaction.newBuilder().build())
            .build();
    }
    
    private Jdbc.JdbcRequest buildRollbackRequestMessage() {
        return Jdbc.JdbcRequest.newBuilder()
            .setRequestId(nextRequestId())
            .setRollbackTransaction(Jdbc.RollbackTransaction.newBuilder().build())
            .build();
    }
    
    // 发送请求（包含4字节长度前缀）
    private void sendRequest(byte[] data) throws IOException {
        // 确保缓冲区处于写入模式
        lenBuffer.clear();
        
        // 写入4字节长度前缀（大端序）
        lenBuffer.putInt(data.length);
        lenBuffer.flip();
        
        // 使用socket的输出流进行写入
        OutputStream outputStream = socket.getOutputStream();
        
        // 发送长度前缀
        outputStream.write(lenBuffer.array());
        
        // 发送请求数据
        outputStream.write(data);
        outputStream.flush();
    }
    
    // 接收响应
    private byte[] receiveResponse() throws IOException {
        // 使用socket的输入流进行读取，依赖socket.setSoTimeout()设置的超时
        InputStream inputStream = socket.getInputStream();
        
        // 读取4字节长度前缀
        byte[] lenBytes = new byte[4];
        int bytesRead = 0;
        while (bytesRead < 4) {
            int read = inputStream.read(lenBytes, bytesRead, 4 - bytesRead);
            if (read == -1) {
                throw new IOException("Connection closed while reading response length");
            }
            bytesRead += read;
        }
        
        // 获取响应数据长度（大端序）
        int responseLength = ((lenBytes[0] & 0xFF) << 24) | 
                            ((lenBytes[1] & 0xFF) << 16) | 
                            ((lenBytes[2] & 0xFF) << 8) | 
                            (lenBytes[3] & 0xFF);
        
        // 读取响应数据
        byte[] responseData = new byte[responseLength];
        int totalRead = 0;
        while (totalRead < responseLength) {
            int read = inputStream.read(responseData, totalRead, responseLength - totalRead);
            if (read == -1) {
                throw new IOException("Connection closed while reading response data");
            }
            totalRead += read;
        }
        
        return responseData;
    }
    
    // 处理连接响应
    private void handleConnectionResponse(Jdbc.JdbcResponse response) throws SQLException {
        if (response.getStatus() != Jdbc.Status.OK) {
            String message = response.getErrorMessage();
            if (message == null || message.isEmpty()) {
                message = "Connection failed with status: " + response.getStatus().name();
            }
            throw new SQLException(message);
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
                // 关闭资源
                channel.close();
                socket.close();
                
                // 关闭并释放直接内存缓冲池
                bufferPool.close();
                
                closed = true;
            } catch (IOException e) {
                // 忽略关闭过程中的异常，确保资源被释放
                try {
                    channel.close();
                    socket.close();
                    bufferPool.close();
                } catch (IOException ex) {
                    // 再次忽略
                }
                closed = true;
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
            Jdbc.JdbcRequest request;
            String normalized = command.trim();
            
            if (normalized.startsWith("EXECUTE|")) {
                String sql = normalized.substring("EXECUTE|".length());
                request = buildQueryRequestMessage(sql);
            } else if (normalized.equalsIgnoreCase("BEGIN") || normalized.equalsIgnoreCase("BEGIN TRANSACTION")) {
                request = buildBeginRequestMessage();
            } else if (normalized.equalsIgnoreCase("COMMIT")) {
                request = buildCommitRequestMessage();
            } else if (normalized.equalsIgnoreCase("ROLLBACK")) {
                request = buildRollbackRequestMessage();
            } else {
                request = buildQueryRequestMessage(normalized);
            }
            
            sendJdbcRequest(request);
            Jdbc.JdbcResponse response = receiveJdbcResponse();
            return convertResponseToLegacyString(response);
        } catch (java.net.SocketTimeoutException e) {
            throw new SQLException("Command timed out. Is the RemDb server running?", e);
        } catch (IOException e) {
            throw new SQLException("Failed to execute command: " + e.getMessage(), e);
        }
    }

    private String convertResponseToLegacyString(Jdbc.JdbcResponse response) {
        if (response.getStatus() != Jdbc.Status.OK) {
            String message = response.getErrorMessage();
            if (message == null || message.isEmpty()) {
                message = response.getStatus().name();
            }
            return "ERROR|" + message;
        }

        if (response.hasResultSet()) {
            Jdbc.ResultSetResponse resultSet = response.getResultSet();
            StringBuilder columns = new StringBuilder();
            for (int i = 0; i < resultSet.getColumnsCount(); i++) {
                if (i > 0) {
                    columns.append(",");
                }
                columns.append(resultSet.getColumns(i).getName());
            }

            StringBuilder rows = new StringBuilder();
            for (int i = 0; i < resultSet.getRowsCount(); i++) {
                if (i > 0) {
                    rows.append(";");
                }
                Jdbc.RowData row = resultSet.getRows(i);
                for (int j = 0; j < row.getValuesCount(); j++) {
                    if (j > 0) {
                        rows.append(",");
                    }
                    rows.append(valueToString(row.getValues(j)));
                }
            }

            return "OK|" + resultSet.getRowCount() + "|" + resultSet.getColumnsCount() + "|" + columns + "|" + rows;
        }

        if (response.hasUpdate()) {
            Jdbc.UpdateResponse update = response.getUpdate();
            return "OK|" + update.getAffectedRows() + "|0|";
        }

        return "OK|0|0|";
    }

    private String valueToString(Jdbc.Value value) {
        switch (value.getValueCase()) {
            case BOOLEAN_VALUE:
                return String.valueOf(value.getBooleanValue());
            case INT32_VALUE:
                return String.valueOf(value.getInt32Value());
            case INT64_VALUE:
                return String.valueOf(value.getInt64Value());
            case FLOAT_VALUE:
                return String.valueOf(value.getFloatValue());
            case DOUBLE_VALUE:
                return String.valueOf(value.getDoubleValue());
            case STRING_VALUE:
                return value.getStringValue();
            case BYTES_VALUE:
                return value.getBytesValue().toStringUtf8();
            case UINT64_VALUE:
                return String.valueOf(value.getUint64Value());
            case SINT64_VALUE:
                return String.valueOf(value.getSint64Value());
            case FIXED32_VALUE:
                return String.valueOf(value.getFixed32Value());
            case FIXED64_VALUE:
                return String.valueOf(value.getFixed64Value());
            case SFIXED32_VALUE:
                return String.valueOf(value.getSfixed32Value());
            case SFIXED64_VALUE:
                return String.valueOf(value.getSfixed64Value());
            case DATE_VALUE:
                return value.getDateValue().toStringUtf8();
            case TIME_VALUE:
                return value.getTimeValue().toStringUtf8();
            case TIMESTAMP_VALUE:
                return value.getTimestampValue().toStringUtf8();
            case VECTOR_DATA:
                Jdbc.VectorData vectorData = value.getVectorData();
                StringBuilder sb = new StringBuilder();
                sb.append('[');
                
                if (vectorData.getValuesCount() > 0) {
                    // 处理float向量
                    for (int i = 0; i < vectorData.getValuesCount(); i++) {
                        if (i > 0) {
                            sb.append(", ");
                        }
                        sb.append(String.format("%.4f", vectorData.getValues(i)));
                    }
                } else if (vectorData.getDoubleValuesCount() > 0) {
                    // 处理double向量
                    for (int i = 0; i < vectorData.getDoubleValuesCount(); i++) {
                        if (i > 0) {
                            sb.append(", ");
                        }
                        sb.append(String.format("%.4f", vectorData.getDoubleValues(i)));
                    }
                }
                
                sb.append(']');
                return sb.toString();
            case NULL_VALUE:
            case VALUE_NOT_SET:
            default:
                return "NULL";
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