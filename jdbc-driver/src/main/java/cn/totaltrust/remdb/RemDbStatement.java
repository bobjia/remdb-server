package cn.totaltrust.remdb;

import java.sql.*;
import java.util.ArrayList;
import java.util.List;

public class RemDbStatement implements Statement {
    private RemDbConnection connection;
    private boolean closed = false;
    private RemDbResultSet currentResultSet = null;
    private List<String> batchSqls = new ArrayList<>();

    public RemDbStatement(RemDbConnection connection) {
        this.connection = connection;
    }

    @Override
    public ResultSet executeQuery(String sql) throws SQLException {
        checkClosed();
        String response = connection.executeCommand("EXECUTE|" + sql);
        return parseResultSet(response, sql);
    }

    @Override
    public int executeUpdate(String sql) throws SQLException {
        checkClosed();
        String response = connection.executeCommand("EXECUTE|" + sql);
        return parseUpdateCount(response);
    }

    @Override
    public boolean execute(String sql) throws SQLException {
        checkClosed();
        String response = connection.executeCommand("EXECUTE|" + sql);
        
        if (response.startsWith("ERROR|")) {
            throw new SQLException(response.substring(6));
        }

        // Check if response is a result set
        String[] parts = response.split("\\|", 4);
        if (parts.length >= 4 && parts[0].equals("OK")) {
            int columnCount = Integer.parseInt(parts[2]);
            if (columnCount > 0) {
                currentResultSet = parseResultSet(response, sql);
                return true; // Has result set
            }
        }

        return false; // No result set, only update count
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
    public void setCursorName(String name) throws SQLException {
        checkClosed();
        // Cursor names not supported
    }

    @Override
    public boolean getMoreResults() throws SQLException {
        checkClosed();
        return false; // Only one result set per execution
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
        batchSqls.add(sql);
    }

    @Override
    public void clearBatch() throws SQLException {
        checkClosed();
        batchSqls.clear();
    }

    @Override
    public int[] executeBatch() throws SQLException {
        checkClosed();
        
        if (batchSqls.isEmpty()) {
            return new int[0];
        }
        
        // 处理批量SQL
        int[] result = new int[batchSqls.size()];
        
        // 如果所有SQL都是INSERT语句，尝试合并为批量INSERT
        boolean allInserts = true;
        String firstSql = batchSqls.get(0);
        String tableName = null;
        List<String> columns = null;
        List<List<String>> allValues = new ArrayList<>();
        
        // 检查是否所有SQL都是INSERT语句，并且结构相同
        for (int i = 0; i < batchSqls.size(); i++) {
            String sql = batchSqls.get(i);
            String lowerSql = sql.trim().toLowerCase();
            
            if (!lowerSql.startsWith("insert into ")) {
                allInserts = false;
                break;
            }
            
            // 解析表名和列名
            if (i == 0) {
                // 解析第一个INSERT语句
                tableName = extractTableName(lowerSql);
                columns = extractColumns(sql);
            } else {
                // 检查后续INSERT语句的表名和列名是否与第一个相同
                String currentTableName = extractTableName(lowerSql);
                List<String> currentColumns = extractColumns(sql);
                
                if (!currentTableName.equals(tableName) || !currentColumns.equals(columns)) {
                    allInserts = false;
                    break;
                }
            }
            
            // 提取值
            List<String> values = extractValues(sql);
            allValues.add(values);
        }
        
        if (allInserts && !batchSqls.isEmpty()) {
            // 构建批量INSERT语句
            String batchInsertSql = buildBatchInsertSql(firstSql, columns, allValues);
            String response = connection.executeCommand("EXECUTE|" + batchInsertSql);
            int affectedRows = parseUpdateCount(response);
            
            // 对于批量INSERT，返回每个操作的影响行数
            for (int i = 0; i < result.length; i++) {
                result[i] = affectedRows / result.length;
            }
        } else {
            // 逐个执行SQL
            for (int i = 0; i < batchSqls.size(); i++) {
                String sql = batchSqls.get(i);
                String response = connection.executeCommand("EXECUTE|" + sql);
                result[i] = parseUpdateCount(response);
            }
        }
        
        // 清空批处理列表
        clearBatch();
        
        return result;
    }
    
    private String extractTableName(String lowerSql) {
        // 从 INSERT INTO table_name 中提取表名
        int start = "insert into ".length();
        int end = lowerSql.indexOf('(', start);
        if (end == -1) {
            end = lowerSql.indexOf(' ', start + 1);
            if (end == -1) {
                end = lowerSql.length();
            }
        }
        return lowerSql.substring(start, end).trim();
    }
    
    private List<String> extractColumns(String sql) {
        List<String> columns = new ArrayList<>();
        int openParen = sql.indexOf('(');
        if (openParen == -1) {
            return columns;
        }
        
        int closeParen = sql.indexOf(')', openParen);
        if (closeParen == -1) {
            return columns;
        }
        
        String columnsPart = sql.substring(openParen + 1, closeParen).trim();
        if (columnsPart.isEmpty()) {
            return columns;
        }
        
        // 检查是否是VALUES前的列名列表
        int valuesPos = sql.indexOf("VALUES", closeParen);
        if (valuesPos == -1) {
            valuesPos = sql.indexOf("values", closeParen);
        }
        
        if (valuesPos == -1) {
            return columns;
        }
        
        String[] columnArray = columnsPart.split(",");
        for (String col : columnArray) {
            columns.add(col.trim());
        }
        
        return columns;
    }
    
    private List<String> extractValues(String sql) {
        List<String> values = new ArrayList<>();
        
        // 查找VALUES关键字
        int valuesPos = sql.indexOf("VALUES");
        if (valuesPos == -1) {
            valuesPos = sql.indexOf("values");
        }
        
        if (valuesPos == -1) {
            return values;
        }
        
        valuesPos += 6; // 跳过VALUES关键字
        
        // 查找值列表的开始和结束位置
        int openParen = sql.indexOf('(', valuesPos);
        if (openParen == -1) {
            return values;
        }
        
        int closeParen = sql.indexOf(')', openParen);
        if (closeParen == -1) {
            return values;
        }
        
        String valuesPart = sql.substring(openParen + 1, closeParen).trim();
        if (valuesPart.isEmpty()) {
            return values;
        }
        
        // 解析值列表，处理引号内的逗号
        boolean inQuotes = false;
        char quoteChar = '\0';
        StringBuilder currentValue = new StringBuilder();
        
        for (char c : valuesPart.toCharArray()) {
            if (c == '"' || c == '\'') {
                if (!inQuotes) {
                    inQuotes = true;
                    quoteChar = c;
                    currentValue.append(c);
                } else if (c == quoteChar) {
                    inQuotes = false;
                    quoteChar = '\0';
                    currentValue.append(c);
                } else {
                    currentValue.append(c);
                }
            } else if (c == ',' && !inQuotes) {
                values.add(currentValue.toString().trim());
                currentValue.setLength(0);
            } else {
                currentValue.append(c);
            }
        }
        
        if (currentValue.length() > 0) {
            values.add(currentValue.toString().trim());
        }
        
        return values;
    }
    
    private String buildBatchInsertSql(String firstSql, List<String> columns, List<List<String>> allValues) {
        StringBuilder sql = new StringBuilder();
        
        // 提取INSERT INTO table_name部分
        int openParen = firstSql.indexOf('(');
        if (openParen == -1) {
            return firstSql; // 无法解析，返回原SQL
        }
        
        sql.append(firstSql.substring(0, openParen + 1));
        
        // 添加列名
        if (!columns.isEmpty()) {
            for (int i = 0; i < columns.size(); i++) {
                if (i > 0) {
                    sql.append(", ");
                }
                sql.append(columns.get(i));
            }
        }
        
        sql.append(") VALUES ");
        
        // 添加所有值组
        for (int i = 0; i < allValues.size(); i++) {
            if (i > 0) {
                sql.append(", ");
            }
            
            sql.append("(");
            List<String> values = allValues.get(i);
            for (int j = 0; j < values.size(); j++) {
                if (j > 0) {
                    sql.append(", ");
                }
                sql.append(values.get(j));
            }
            sql.append(")");
        }
        
        return sql.toString();
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
    public int executeUpdate(String sql, int autoGeneratedKeys) throws SQLException {
        checkClosed();
        return executeUpdate(sql);
    }

    @Override
    public int executeUpdate(String sql, int[] columnIndexes) throws SQLException {
        checkClosed();
        return executeUpdate(sql);
    }

    @Override
    public int executeUpdate(String sql, String[] columnNames) throws SQLException {
        checkClosed();
        return executeUpdate(sql);
    }

    @Override
    public boolean execute(String sql, int autoGeneratedKeys) throws SQLException {
        checkClosed();
        return execute(sql);
    }

    @Override
    public boolean execute(String sql, int[] columnIndexes) throws SQLException {
        checkClosed();
        return execute(sql);
    }

    @Override
    public boolean execute(String sql, String[] columnNames) throws SQLException {
        checkClosed();
        return execute(sql);
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
            throw new SQLException("Statement is closed");
        }
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
                        if (!rowString.isEmpty()) {
                            List<String> row = new ArrayList<>();
                            String[] values = rowString.split(",");
                            // 确保只添加与列名数量匹配的行数据
                            for (int i = 0; i < Math.min(values.length, columnNames.length); i++) {
                                row.add(values[i]);
                            }
                            // 只有当行数据的列数与列名的数量匹配时，才添加到 rows 列表中
                            if (row.size() == columnNames.length) {
                                rows.add(row);
                            }
                        }
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