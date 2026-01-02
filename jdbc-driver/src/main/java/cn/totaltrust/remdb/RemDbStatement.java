package cn.totaltrust.remdb;

import java.sql.*;
import java.util.ArrayList;
import java.util.List;

public class RemDbStatement implements Statement {
    private RemDbConnection connection;
    private boolean closed = false;
    private RemDbResultSet currentResultSet = null;

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
        throw new SQLFeatureNotSupportedException("Batches not supported");
    }

    @Override
    public void clearBatch() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Batches not supported");
    }

    @Override
    public int[] executeBatch() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("Batches not supported");
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