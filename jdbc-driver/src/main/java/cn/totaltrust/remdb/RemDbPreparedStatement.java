package cn.totaltrust.remdb;

import java.sql.*;
import java.util.ArrayList;
import java.util.List;

public class RemDbPreparedStatement implements PreparedStatement {
    private RemDbConnection connection;
    private String sql;
    private List<Object> parameters;
    private List<List<Object>> batchParameters;
    private boolean closed = false;
    private RemDbResultSet currentResultSet = null;

    public RemDbPreparedStatement(RemDbConnection connection, String sql) {
        this.connection = connection;
        this.sql = sql;
        this.parameters = new ArrayList<>();
        this.batchParameters = new ArrayList<>();
    }

    @Override
    public ResultSet executeQuery() throws SQLException {
        checkClosed();
        String finalSql = replaceParameters();
        String response = connection.executeCommand("EXECUTE|" + finalSql);
        return parseResultSet(response, finalSql);
    }

    @Override
    public int executeUpdate() throws SQLException {
        checkClosed();
        String finalSql = replaceParameters();
        String response = connection.executeCommand("EXECUTE|" + finalSql);
        return parseUpdateCount(response);
    }

    @Override
    public boolean execute() throws SQLException {
        checkClosed();
        String finalSql = replaceParameters();
        String response = connection.executeCommand("EXECUTE|" + finalSql);
        
        if (response.startsWith("ERROR|")) {
            throw new SQLException(response.substring(6));
        }

        // Check if response is a result set
        String[] parts = response.split("\\|", 4);
        if (parts.length >= 4 && parts[0].equals("OK")) {
            int columnCount = Integer.parseInt(parts[2]);
            if (columnCount > 0) {
                currentResultSet = parseResultSet(response, finalSql);
                return true; // Has result set
            }
        }

        return false; // No result set, only update count
    }

    @Override
    public void setNull(int parameterIndex, int sqlType) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, null);
    }

    @Override
    public void setBoolean(int parameterIndex, boolean x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setByte(int parameterIndex, byte x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setShort(int parameterIndex, short x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setInt(int parameterIndex, int x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setLong(int parameterIndex, long x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setFloat(int parameterIndex, float x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setDouble(int parameterIndex, double x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setBigDecimal(int parameterIndex, java.math.BigDecimal x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setString(int parameterIndex, String x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setBytes(int parameterIndex, byte[] x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setDate(int parameterIndex, Date x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setTime(int parameterIndex, Time x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setTimestamp(int parameterIndex, Timestamp x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setAsciiStream(int parameterIndex, java.io.InputStream x, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setAsciiStream not supported");
    }

    @Override
    public void setUnicodeStream(int parameterIndex, java.io.InputStream x, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setUnicodeStream not supported");
    }

    @Override
    public void setBinaryStream(int parameterIndex, java.io.InputStream x, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setBinaryStream not supported");
    }

    @Override
    public void clearParameters() throws SQLException {
        checkClosed();
        parameters.clear();
    }

    @Override
    public void setObject(int parameterIndex, Object x, int targetSqlType) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setObject(int parameterIndex, Object x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void addBatch() throws SQLException {
        checkClosed();
        // 保存当前参数到批处理列表
        List<Object> batchParam = new ArrayList<>(parameters);
        batchParameters.add(batchParam);
        // 清空当前参数，准备下一批
        parameters.clear();
    }

    @Override
    public void clearBatch() throws SQLException {
        checkClosed();
        batchParameters.clear();
    }

    @Override
    public int[] executeBatch() throws SQLException {
        checkClosed();
        
        if (batchParameters.isEmpty()) {
            return new int[0];
        }
        
        // 处理批量参数
        int[] result = new int[batchParameters.size()];
        
        // 如果是INSERT语句，尝试合并为批量INSERT
        String lowerSql = sql.trim().toLowerCase();
        boolean isInsert = lowerSql.startsWith("insert into ");
        
        if (isInsert) {
            // 构建批量INSERT语句
            String batchInsertSql = buildBatchInsertSql();
            String response = connection.executeCommand("EXECUTE|" + batchInsertSql);
            int affectedRows = parseUpdateCount(response);
            
            // 对于批量INSERT，返回每个操作的影响行数
            for (int i = 0; i < result.length; i++) {
                result[i] = affectedRows / result.length;
            }
        } else {
            // 逐个执行SQL
            for (int i = 0; i < batchParameters.size(); i++) {
                // 设置当前参数
                parameters = new ArrayList<>(batchParameters.get(i));
                // 替换参数并执行
                String finalSql = replaceParameters();
                String response = connection.executeCommand("EXECUTE|" + finalSql);
                result[i] = parseUpdateCount(response);
            }
        }
        
        // 清空批处理列表
        clearBatch();
        
        return result;
    }
    
    private String buildBatchInsertSql() {
        // 解析原始INSERT语句
        String[] parts = sql.split("\\?");
        StringBuilder columnPart = new StringBuilder();
        StringBuilder valuePart = new StringBuilder();
        
        // 提取列名部分
        int openParen = sql.indexOf('(');
        int closeParen = sql.indexOf(')', openParen);
        if (openParen != -1 && closeParen != -1) {
            columnPart.append(sql.substring(0, closeParen + 1));
        } else {
            // 没有列名，直接使用原始语句的前半部分
            int valuesPos = sql.indexOf("VALUES");
            if (valuesPos == -1) {
                valuesPos = sql.indexOf("values");
            }
            if (valuesPos != -1) {
                columnPart.append(sql.substring(0, valuesPos));
            } else {
                columnPart.append(sql);
            }
        }
        
        columnPart.append(" VALUES ");
        
        // 构建所有值组
        for (int i = 0; i < batchParameters.size(); i++) {
            if (i > 0) {
                valuePart.append(", ");
            }
            
            valuePart.append("(");
            List<Object> params = batchParameters.get(i);
            for (int j = 0; j < params.size(); j++) {
                if (j > 0) {
                    valuePart.append(", ");
                }
                
                Object param = params.get(j);
                if (param == null) {
                    valuePart.append("NULL");
                } else if (param instanceof String) {
                    // 转义字符串中的单引号
                    String strVal = (String) param;
                    strVal = strVal.replace("'", "''");
                    valuePart.append("'").append(strVal).append("'");
                } else if (param instanceof Boolean) {
                    valuePart.append(((Boolean) param) ? 1 : 0);
                } else {
                    valuePart.append(param.toString());
                }
            }
            valuePart.append(")");
        }
        
        return columnPart.toString() + valuePart.toString();
    }

    @Override
    public void setCharacterStream(int parameterIndex, java.io.Reader reader, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setCharacterStream not supported");
    }

    @Override
    public void setRef(int parameterIndex, Ref x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setRef not supported");
    }

    @Override
    public void setBlob(int parameterIndex, Blob x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setBlob not supported");
    }

    @Override
    public void setClob(int parameterIndex, Clob x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setClob not supported");
    }

    @Override
    public void setArray(int parameterIndex, Array x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setArray not supported");
    }

    @Override
    public ResultSetMetaData getMetaData() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getMetaData not supported");
    }

    @Override
    public void setDate(int parameterIndex, Date x, java.util.Calendar cal) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setTime(int parameterIndex, Time x, java.util.Calendar cal) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setTimestamp(int parameterIndex, Timestamp x, java.util.Calendar cal) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setNull(int parameterIndex, int sqlType, String typeName) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, null);
    }

    @Override
    public void setURL(int parameterIndex, java.net.URL x) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public ParameterMetaData getParameterMetaData() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getParameterMetaData not supported");
    }

    @Override
    public void setRowId(int parameterIndex, RowId x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setRowId not supported");
    }

    @Override
    public void setNString(int parameterIndex, String value) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, value);
    }

    @Override
    public void setNCharacterStream(int parameterIndex, java.io.Reader value, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setNCharacterStream not supported");
    }

    @Override
    public void setNClob(int parameterIndex, NClob value) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setNClob not supported");
    }

    @Override
    public void setClob(int parameterIndex, java.io.Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setClob not supported");
    }

    @Override
    public void setBlob(int parameterIndex, java.io.InputStream inputStream, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setBlob not supported");
    }

    @Override
    public void setNClob(int parameterIndex, java.io.Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setNClob not supported");
    }

    @Override
    public void setSQLXML(int parameterIndex, SQLXML xmlObject) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setSQLXML not supported");
    }

    @Override
    public void setObject(int parameterIndex, Object x, int targetSqlType, int scaleOrLength) throws SQLException {
        checkClosed();
        setParameter(parameterIndex, x);
    }

    @Override
    public void setAsciiStream(int parameterIndex, java.io.InputStream x, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setAsciiStream not supported");
    }

    @Override
    public void setBinaryStream(int parameterIndex, java.io.InputStream x, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setBinaryStream not supported");
    }

    @Override
    public void setCharacterStream(int parameterIndex, java.io.Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setCharacterStream not supported");
    }

    @Override
    public void setAsciiStream(int parameterIndex, java.io.InputStream x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setAsciiStream not supported");
    }

    @Override
    public void setBinaryStream(int parameterIndex, java.io.InputStream x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setBinaryStream not supported");
    }

    @Override
    public void setCharacterStream(int parameterIndex, java.io.Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setCharacterStream not supported");
    }

    @Override
    public void setNCharacterStream(int parameterIndex, java.io.Reader value) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setNCharacterStream not supported");
    }

    @Override
    public void setClob(int parameterIndex, java.io.Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setClob not supported");
    }

    @Override
    public void setBlob(int parameterIndex, java.io.InputStream inputStream) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setBlob not supported");
    }

    @Override
    public void setNClob(int parameterIndex, java.io.Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("setNClob not supported");
    }

    @Override
    public ResultSet getResultSet() throws SQLException {
        checkClosed();
        return currentResultSet;
    }

    @Override
    public int getUpdateCount() throws SQLException {
        checkClosed();
        if (currentResultSet != null) {
            return -1; // Has result set, no update count
        }
        return 0; // Default update count
    }

    @Override
    public boolean getMoreResults() throws SQLException {
        checkClosed();
        return false; // Only one result set per execution
    }

    @Override
    public void setCursorName(String name) throws SQLException {
        checkClosed();
        // Cursor names not supported
    }

    @Override
    public int getMaxFieldSize() throws SQLException {
        checkClosed();
        return 0; // No maximum field size
    }

    @Override
    public void setMaxFieldSize(int max) throws SQLException {
        checkClosed();
        // Maximum field size not supported
    }

    @Override
    public int getMaxRows() throws SQLException {
        checkClosed();
        return 0; // No maximum rows
    }

    @Override
    public void setMaxRows(int max) throws SQLException {
        checkClosed();
        // Maximum rows not supported
    }

    @Override
    public void setEscapeProcessing(boolean enable) throws SQLException {
        checkClosed();
        // Escape processing not supported
    }

    @Override
    public int getQueryTimeout() throws SQLException {
        checkClosed();
        return 0; // No query timeout
    }

    @Override
    public void setQueryTimeout(int seconds) throws SQLException {
        checkClosed();
        // Query timeout not supported
    }

    @Override
    public void cancel() throws SQLException {
        checkClosed();
        // Cancellation not supported
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
    public void setFetchDirection(int direction) throws SQLException {
        checkClosed();
        // Fetch direction not supported
    }

    @Override
    public int getFetchDirection() throws SQLException {
        checkClosed();
        return ResultSet.FETCH_FORWARD; // Default fetch direction
    }

    @Override
    public void setFetchSize(int rows) throws SQLException {
        checkClosed();
        // Fetch size not supported
    }

    @Override
    public int getFetchSize() throws SQLException {
        checkClosed();
        return 0; // No fetch size
    }

    @Override
    public int getResultSetConcurrency() throws SQLException {
        checkClosed();
        return ResultSet.CONCUR_READ_ONLY; // Result sets are read-only
    }

    @Override
    public int getResultSetType() throws SQLException {
        checkClosed();
        return ResultSet.TYPE_FORWARD_ONLY; // Result sets are forward-only
    }

    @Override
    public void addBatch(String sql) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("addBatch not supported");
    }

    public void clearBatch(String sql) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("clearBatch not supported");
    }

    public int[] executeBatch(String sql) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("executeBatch not supported");
    }

    @Override
    public Connection getConnection() throws SQLException {
        checkClosed();
        return connection;
    }

    @Override
    public boolean getMoreResults(int current) throws SQLException {
        checkClosed();
        return false; // Only one result set per execution
    }

    @Override
    public ResultSet getGeneratedKeys() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Generated keys not supported");
    }

    @Override
    public int executeUpdate(String sql) throws SQLException {
        checkClosed();
        return executeUpdate();
    }

    @Override
    public ResultSet executeQuery(String sql) throws SQLException {
        checkClosed();
        return executeQuery();
    }

    @Override
    public boolean execute(String sql) throws SQLException {
        checkClosed();
        return execute();
    }

    @Override
    public int executeUpdate(String sql, int autoGeneratedKeys) throws SQLException {
        checkClosed();
        return executeUpdate();
    }

    @Override
    public int executeUpdate(String sql, int[] columnIndexes) throws SQLException {
        checkClosed();
        return executeUpdate();
    }

    @Override
    public int executeUpdate(String sql, String[] columnNames) throws SQLException {
        checkClosed();
        return executeUpdate();
    }

    @Override
    public boolean execute(String sql, int autoGeneratedKeys) throws SQLException {
        checkClosed();
        return execute();
    }

    @Override
    public boolean execute(String sql, int[] columnIndexes) throws SQLException {
        checkClosed();
        return execute();
    }

    @Override
    public boolean execute(String sql, String[] columnNames) throws SQLException {
        checkClosed();
        return execute();
    }

    public Statement getStatement() throws SQLException {
        checkClosed();
        return new RemDbStatement(connection);
    }

    @Override
    public int getResultSetHoldability() throws SQLException {
        checkClosed();
        return ResultSet.HOLD_CURSORS_OVER_COMMIT; // Default holdability
    }

    @Override
    public void close() throws SQLException {
        if (!closed) {
            closed = true;
            if (currentResultSet != null) {
                currentResultSet.close();
            }
        }
    }

    @Override
    public boolean isClosed() throws SQLException {
        return closed;
    }

    @Override
    public void setPoolable(boolean poolable) throws SQLException {
        checkClosed();
        // Poolable not supported
    }

    @Override
    public boolean isPoolable() throws SQLException {
        checkClosed();
        return false; // Not poolable
    }

    @Override
    public void closeOnCompletion() throws SQLException {
        checkClosed();
        // Close on completion not supported
    }

    @Override
    public boolean isCloseOnCompletion() throws SQLException {
        checkClosed();
        return false; // Not closed on completion
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
            throw new SQLException("PreparedStatement is closed");
        }
    }

    private void setParameter(int parameterIndex, Object value) {
        // Adjust index to 0-based
        int index = parameterIndex - 1;
        // Ensure parameters list is large enough
        while (parameters.size() <= index) {
            parameters.add(null);
        }
        parameters.set(index, value);
    }

    private String replaceParameters() {
        // Simple parameter replacement for ? placeholders
        String[] parts = sql.split("\\?");
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < parts.length; i++) {
            sb.append(parts[i]);
            if (i < parameters.size()) {
                Object param = parameters.get(i);
                if (param == null) {
                    sb.append("NULL");
                } else if (param instanceof String) {
                    // Escape single quotes in strings
                    String strVal = (String) param;
                    strVal = strVal.replace("'", "''");
                    sb.append("'").append(strVal).append("'");
                } else if (param instanceof Boolean) {
                    sb.append(((Boolean) param) ? 1 : 0);
                } else {
                    sb.append(param.toString());
                }
            }
        }
        return sb.toString();
    }

    private RemDbResultSet parseResultSet(String response, String sql) throws SQLException {
        if (response.startsWith("ERROR|")) {
            throw new SQLException(response.substring(6));
        }

        // Parse response: OK|affected_rows|column_count|columns|rows
        String[] parts = response.split("\\|", 5);
        if (parts.length < 5 || !parts[0].equals("OK")) {
            throw new SQLException("Invalid response format: " + response);
        }

        List<String> columns = new ArrayList<>();
        List<List<String>> rows = new ArrayList<>();

        int columnCount = Integer.parseInt(parts[2]);
        if (columnCount > 0) {
            // Parse columns
            String[] columnNames = parts[3].split(",");
            for (String columnName : columnNames) {
                columns.add(columnName);
            }

            // Parse rows
            if (parts.length > 4) {
                String rowsPart = parts[4];
                if (!rowsPart.isEmpty()) {
                    String[] rowStrings = rowsPart.split(";");
                    for (String rowString : rowStrings) {
                        List<String> row = new ArrayList<>();
                        String[] values = rowString.split(",");
                        for (String value : values) {
                            row.add(value);
                        }
                        rows.add(row);
                    }
                }
            }
        }

        return new RemDbResultSet(columns, rows);
    }

    private int parseUpdateCount(String response) throws SQLException {
        if (response.startsWith("ERROR|")) {
            throw new SQLException(response.substring(6));
        }

        // Parse response: OK|affected_rows|0|
        String[] parts = response.split("\\|", 3);
        if (parts.length >= 2 && parts[0].equals("OK")) {
            return Integer.parseInt(parts[1]);
        }

        return 0; // Default update count
    }
}