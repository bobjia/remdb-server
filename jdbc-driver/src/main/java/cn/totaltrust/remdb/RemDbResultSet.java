package cn.totaltrust.remdb;

import java.sql.*;
import java.io.InputStream;
import java.io.Reader;
import java.math.BigDecimal;
import java.net.URL;
import java.sql.Array;
import java.sql.Blob;
import java.sql.Clob;
import java.sql.Date;
import java.sql.NClob;
import java.sql.Ref;
import java.sql.RowId;
import java.sql.SQLException;
import java.sql.SQLXML;
import java.sql.Time;
import java.sql.Timestamp;
import java.util.Calendar;
import java.util.List;
import java.util.Map;

public class RemDbResultSet implements ResultSet {
    private List<String> columns;
    private List<List<String>> rows;
    private int currentRowIndex = -1;
    private boolean closed = false;
    private ResultSetMetaData metaData;

    public RemDbResultSet(List<String> columns, List<List<String>> rows) {
        this.columns = columns;
        this.rows = rows;
        this.metaData = new RemDbResultSetMetaData(columns);
    }

    @Override
    public boolean next() throws SQLException {
        checkClosed();
        currentRowIndex++;
        return currentRowIndex < rows.size();
    }

    @Override
    public void close() throws SQLException {
        closed = true;
    }

    @Override
    public boolean isClosed() throws SQLException {
        return closed;
    }

    @Override
    public boolean wasNull() throws SQLException {
        checkClosed();
        // Not implemented, always return false
        return false;
    }

    @Override
    public String getString(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        
        // Fix UTF-8 double encoding issue
        try {
            byte[] bytes = value.getBytes(java.nio.charset.StandardCharsets.ISO_8859_1);
            return new String(bytes, java.nio.charset.StandardCharsets.UTF_8);
        } catch (Exception e) {
            // If conversion fails, return the original value
            return value;
        }
    }

    @Override
    public boolean getBoolean(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Boolean.parseBoolean(value) || value.equals("1");
    }

    @Override
    public byte getByte(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Byte.parseByte(value);
    }

    @Override
    public short getShort(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Short.parseShort(value);
    }

    @Override
    public int getInt(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Integer.parseInt(value);
    }

    @Override
    public long getLong(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Long.parseLong(value);
    }

    @Override
    public float getFloat(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Float.parseFloat(value);
    }

    @Override
    public double getDouble(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Double.parseDouble(value);
    }

    @Override
    public BigDecimal getBigDecimal(int columnIndex, int scale) throws SQLException {
        checkClosed();
        return getBigDecimal(columnIndex);
    }

    @Override
    public byte[] getBytes(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return value.getBytes();
    }

    @Override
    public Date getDate(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Date.valueOf(value);
    }

    @Override
    public Time getTime(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return Time.valueOf(value);
    }

    @Override
    public Timestamp getTimestamp(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        try {
            // 尝试从毫秒时间戳解析
            long timestamp = Long.parseLong(value);
            return new Timestamp(timestamp);
        } catch (NumberFormatException e) {
            // 如果不是数字，尝试从字符串格式解析
            return Timestamp.valueOf(value);
        }
    }

    @Override
    public InputStream getAsciiStream(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getAsciiStream not supported");
    }

    @Override
    public InputStream getUnicodeStream(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getUnicodeStream not supported");
    }

    @Override
    public InputStream getBinaryStream(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getBinaryStream not supported");
    }

    @Override
    public Reader getCharacterStream(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getCharacterStream not supported");
    }

    @Override
    public InputStream getAsciiStream(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getAsciiStream not supported");
    }

    public InputStream getUnicodeStream(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getUnicodeStream not supported");
    }

    @Override
    public Reader getCharacterStream(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getCharacterStream not supported");
    }

    @Override
    public InputStream getBinaryStream(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getBinaryStream not supported");
    }

    @Override
    public String getString(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getString(columnIndex);
    }

    @Override
    public boolean getBoolean(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getBoolean(columnIndex);
    }

    @Override
    public byte getByte(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getByte(columnIndex);
    }

    @Override
    public short getShort(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getShort(columnIndex);
    }

    @Override
    public int getInt(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getInt(columnIndex);
    }

    @Override
    public long getLong(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getLong(columnIndex);
    }

    @Override
    public float getFloat(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getFloat(columnIndex);
    }

    @Override
    public double getDouble(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getDouble(columnIndex);
    }

    @Override
    public BigDecimal getBigDecimal(String columnLabel, int scale) throws SQLException {
        checkClosed();
        return getBigDecimal(columnLabel);
    }

    @Override
    public byte[] getBytes(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getBytes(columnIndex);
    }

    @Override
    public Date getDate(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getDate(columnIndex);
    }

    @Override
    public Time getTime(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getTime(columnIndex);
    }

    @Override
    public Timestamp getTimestamp(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getTimestamp(columnIndex);
    }

    @Override
    public BigDecimal getBigDecimal(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        String value = rows.get(currentRowIndex).get(columnIndex - 1);
        return new BigDecimal(value);
    }

    @Override
    public BigDecimal getBigDecimal(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getBigDecimal(columnIndex);
    }

    @Override
    public void updateNull(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNull not supported");
    }

    @Override
    public void updateBoolean(int columnIndex, boolean x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBoolean not supported");
    }

    @Override
    public void updateByte(int columnIndex, byte x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateByte not supported");
    }

    @Override
    public void updateShort(int columnIndex, short x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateShort not supported");
    }

    @Override
    public void updateInt(int columnIndex, int x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateInt not supported");
    }

    @Override
    public void updateLong(int columnIndex, long x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateLong not supported");
    }

    @Override
    public void updateFloat(int columnIndex, float x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateFloat not supported");
    }

    @Override
    public void updateDouble(int columnIndex, double x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateDouble not supported");
    }

    @Override
    public void updateBigDecimal(int columnIndex, BigDecimal x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBigDecimal not supported");
    }

    @Override
    public void updateString(int columnIndex, String x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateString not supported");
    }

    @Override
    public void updateBytes(int columnIndex, byte[] x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBytes not supported");
    }

    @Override
    public void updateDate(int columnIndex, Date x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateDate not supported");
    }

    @Override
    public void updateTime(int columnIndex, Time x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateTime not supported");
    }

    @Override
    public void updateTimestamp(int columnIndex, Timestamp x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateTimestamp not supported");
    }

    @Override
    public void updateAsciiStream(int columnIndex, InputStream x, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateAsciiStream not supported");
    }

    @Override
    public void updateBinaryStream(int columnIndex, InputStream x, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBinaryStream not supported");
    }

    @Override
    public void updateCharacterStream(int columnIndex, Reader x, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateCharacterStream not supported");
    }

    @Override
    public void updateNull(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNull not supported");
    }

    @Override
    public void updateBoolean(String columnLabel, boolean x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBoolean not supported");
    }

    @Override
    public void updateByte(String columnLabel, byte x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateByte not supported");
    }

    @Override
    public void updateShort(String columnLabel, short x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateShort not supported");
    }

    @Override
    public void updateInt(String columnLabel, int x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateInt not supported");
    }

    @Override
    public void updateLong(String columnLabel, long x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateLong not supported");
    }

    @Override
    public void updateFloat(String columnLabel, float x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateFloat not supported");
    }

    @Override
    public void updateDouble(String columnLabel, double x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateDouble not supported");
    }

    @Override
    public void updateBigDecimal(String columnLabel, BigDecimal x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBigDecimal not supported");
    }

    @Override
    public void updateString(String columnLabel, String x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateString not supported");
    }

    @Override
    public void updateBytes(String columnLabel, byte[] x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBytes not supported");
    }

    @Override
    public void updateDate(String columnLabel, Date x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateDate not supported");
    }

    @Override
    public void updateTime(String columnLabel, Time x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateTime not supported");
    }

    @Override
    public void updateTimestamp(String columnLabel, Timestamp x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateTimestamp not supported");
    }

    @Override
    public void updateAsciiStream(String columnLabel, InputStream x, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateAsciiStream not supported");
    }

    @Override
    public void updateBinaryStream(String columnLabel, InputStream x, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBinaryStream not supported");
    }

    @Override
    public void updateCharacterStream(String columnLabel, Reader reader, int length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateCharacterStream not supported");
    }

    @Override
    public void insertRow() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("insertRow not supported");
    }

    @Override
    public void updateRow() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateRow not supported");
    }

    @Override
    public void deleteRow() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("deleteRow not supported");
    }

    @Override
    public void refreshRow() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("refreshRow not supported");
    }

    @Override
    public void cancelRowUpdates() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("cancelRowUpdates not supported");
    }

    @Override
    public void moveToInsertRow() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("moveToInsertRow not supported");
    }

    @Override
    public void moveToCurrentRow() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("moveToCurrentRow not supported");
    }

    @Override
    public Statement getStatement() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getStatement not supported");
    }

    @Override
    public Object getObject(int columnIndex) throws SQLException {
        checkClosed();
        checkRowIndex();
        checkColumnIndex(columnIndex);
        return rows.get(currentRowIndex).get(columnIndex - 1);
    }

    @Override
    public Object getObject(String columnLabel) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getObject(columnIndex);
    }

    @Override
    public int findColumn(String columnLabel) throws SQLException {
        checkClosed();
        for (int i = 0; i < columns.size(); i++) {
            if (columns.get(i).equals(columnLabel)) {
                return i + 1;
            }
        }
        throw new SQLException("Column not found: " + columnLabel);
    }

    @Override
    public boolean rowDeleted() throws SQLException {
        checkClosed();
        return false;
    }

    @Override
    public boolean rowInserted() throws SQLException {
        checkClosed();
        return false;
    }

    @Override
    public boolean rowUpdated() throws SQLException {
        checkClosed();
        return false;
    }

    @Override
    public String getCursorName() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getCursorName not supported");
    }

    @Override
    public SQLWarning getWarnings() throws SQLException {
        checkClosed();
        return null;
    }

    @Override
    public void clearWarnings() throws SQLException {
        checkClosed();
        // No warnings to clear
    }

    @Override
    public Object getObject(int columnIndex, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        return getObject(columnIndex);
    }

    @Override
    public Ref getRef(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getRef not supported");
    }

    @Override
    public Ref getRef(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getRef not supported");
    }

    @Override
    public Blob getBlob(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getBlob not supported");
    }

    @Override
    public Blob getBlob(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getBlob not supported");
    }

    @Override
    public Clob getClob(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getClob not supported");
    }

    @Override
    public Clob getClob(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getClob not supported");
    }

    @Override
    public Array getArray(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getArray not supported");
    }

    @Override
    public Array getArray(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getArray not supported");
    }

    @Override
    public Date getDate(int columnIndex, Calendar cal) throws SQLException {
        checkClosed();
        return getDate(columnIndex);
    }

    @Override
    public Date getDate(String columnLabel, Calendar cal) throws SQLException {
        checkClosed();
        return getDate(columnLabel);
    }

    @Override
    public Time getTime(int columnIndex, Calendar cal) throws SQLException {
        checkClosed();
        return getTime(columnIndex);
    }

    @Override
    public Time getTime(String columnLabel, Calendar cal) throws SQLException {
        checkClosed();
        return getTime(columnLabel);
    }

    @Override
    public Timestamp getTimestamp(int columnIndex, Calendar cal) throws SQLException {
        checkClosed();
        return getTimestamp(columnIndex);
    }

    @Override
    public Timestamp getTimestamp(String columnLabel, Calendar cal) throws SQLException {
        checkClosed();
        return getTimestamp(columnLabel);
    }

    @Override
    public Object getObject(String columnLabel, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        return getObject(columnLabel);
    }

    @Override
    public URL getURL(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getURL not supported");
    }

    @Override
    public URL getURL(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getURL not supported");
    }

    @Override
    public void updateRef(int columnIndex, Ref x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateRef not supported");
    }

    @Override
    public void updateRef(String columnLabel, Ref x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateRef not supported");
    }

    @Override
    public void updateBlob(int columnIndex, Blob x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBlob not supported");
    }

    @Override
    public void updateBlob(String columnLabel, Blob x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBlob not supported");
    }

    @Override
    public void updateBlob(int columnIndex, InputStream inputStream, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBlob not supported");
    }

    @Override
    public void updateBlob(String columnLabel, InputStream inputStream, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBlob not supported");
    }

    @Override
    public void updateClob(int columnIndex, Clob x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateClob not supported");
    }

    @Override
    public void updateClob(String columnLabel, Clob x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateClob not supported");
    }

    @Override
    public void updateClob(int columnIndex, Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateClob not supported");
    }

    @Override
    public void updateClob(String columnLabel, Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateClob not supported");
    }

    @Override
    public void updateArray(int columnIndex, Array x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateArray not supported");
    }

    @Override
    public void updateArray(String columnLabel, Array x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateArray not supported");
    }

    @Override
    public RowId getRowId(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getRowId not supported");
    }

    @Override
    public RowId getRowId(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getRowId not supported");
    }

    @Override
    public void updateRowId(int columnIndex, RowId x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateRowId not supported");
    }

    @Override
    public void updateRowId(String columnLabel, RowId x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateRowId not supported");
    }

    @Override
    public int getHoldability() throws SQLException {
        checkClosed();
        return ResultSet.HOLD_CURSORS_OVER_COMMIT;
    }

    @Override
    public boolean isAfterLast() throws SQLException {
        checkClosed();
        return currentRowIndex >= rows.size();
    }

    @Override
    public boolean isBeforeFirst() throws SQLException {
        checkClosed();
        return currentRowIndex < 0;
    }

    @Override
    public boolean isFirst() throws SQLException {
        checkClosed();
        return currentRowIndex == 0;
    }

    @Override
    public boolean isLast() throws SQLException {
        checkClosed();
        return currentRowIndex == rows.size() - 1;
    }

    @Override
    public void beforeFirst() throws SQLException {
        checkClosed();
        currentRowIndex = -1;
    }

    @Override
    public void afterLast() throws SQLException {
        checkClosed();
        currentRowIndex = rows.size();
    }

    @Override
    public boolean first() throws SQLException {
        checkClosed();
        if (rows.isEmpty()) {
            return false;
        }
        currentRowIndex = 0;
        return true;
    }

    @Override
    public boolean last() throws SQLException {
        checkClosed();
        if (rows.isEmpty()) {
            return false;
        }
        currentRowIndex = rows.size() - 1;
        return true;
    }

    @Override
    public int getRow() throws SQLException {
        checkClosed();
        return currentRowIndex + 1;
    }

    @Override
    public boolean absolute(int row) throws SQLException {
        checkClosed();
        if (row > 0) {
            currentRowIndex = row - 1;
        } else if (row < 0) {
            currentRowIndex = rows.size() + row;
        } else {
            currentRowIndex = -1;
        }
        return currentRowIndex >= 0 && currentRowIndex < rows.size();
    }

    @Override
    public boolean relative(int rowsToMove) throws SQLException {
        checkClosed();
        currentRowIndex += rowsToMove;
        return currentRowIndex >= 0 && currentRowIndex < this.rows.size();
    }

    @Override
    public boolean previous() throws SQLException {
        checkClosed();
        if (currentRowIndex > 0) {
            currentRowIndex--;
            return true;
        }
        return false;
    }

    @Override
    public void setFetchDirection(int direction) throws SQLException {
        checkClosed();
        // Not implemented
    }

    @Override
    public int getFetchDirection() throws SQLException {
        checkClosed();
        return ResultSet.FETCH_FORWARD;
    }

    @Override
    public void setFetchSize(int rows) throws SQLException {
        checkClosed();
        // Not implemented
    }

    @Override
    public int getFetchSize() throws SQLException {
        checkClosed();
        return rows.size();
    }

    @Override
    public int getType() throws SQLException {
        checkClosed();
        return ResultSet.TYPE_FORWARD_ONLY;
    }

    @Override
    public int getConcurrency() throws SQLException {
        checkClosed();
        return ResultSet.CONCUR_READ_ONLY;
    }

    @Override
    public ResultSetMetaData getMetaData() throws SQLException {
        checkClosed();
        return metaData;
    }

    @Override
    public <T> T getObject(int columnIndex, Class<T> type) throws SQLException {
        checkClosed();
        String value = getString(columnIndex);
        if (type == String.class) {
            return type.cast(value);
        } else if (type == Integer.class) {
            return type.cast(Integer.parseInt(value));
        } else if (type == Long.class) {
            return type.cast(Long.parseLong(value));
        } else if (type == Double.class) {
            return type.cast(Double.parseDouble(value));
        } else if (type == Float.class) {
            return type.cast(Float.parseFloat(value));
        } else if (type == Boolean.class) {
            return type.cast(Boolean.parseBoolean(value) || value.equals("1"));
        } else if (type == Byte.class) {
            return type.cast(Byte.parseByte(value));
        } else if (type == Short.class) {
            return type.cast(Short.parseShort(value));
        } else if (type == BigDecimal.class) {
            return type.cast(new BigDecimal(value));
        } else if (type == Date.class) {
            return type.cast(Date.valueOf(value));
        } else if (type == Time.class) {
            return type.cast(Time.valueOf(value));
        } else if (type == Timestamp.class) {
            try {
                // 尝试从毫秒时间戳解析
                long timestamp = Long.parseLong(value);
                return type.cast(new Timestamp(timestamp));
            } catch (NumberFormatException e) {
                // 如果不是数字，尝试从字符串格式解析
                return type.cast(Timestamp.valueOf(value));
            }
        } else if (type == float[].class || type == Float[].class) {
            // 处理向量类型，从字符串解析为float数组
            return type.cast(parseVectorToFloatArray(value));
        } else if (type == double[].class || type == Double[].class) {
            // 处理向量类型，从字符串解析为double数组
            return type.cast(parseVectorToDoubleArray(value));
        } else {
            throw new SQLException("Unsupported type: " + type.getName());
        }
    }
    
    /**
     * 将向量字符串解析为float数组
     */
    private float[] parseVectorToFloatArray(String vectorStr) throws SQLException {
        if (vectorStr == null || vectorStr.isEmpty() || vectorStr.equals("NULL")) {
            return null;
        }
        
        // 处理可能的特殊情况
        String trimmedStr = vectorStr.trim();
        
        // 检查是否是向量格式
        if (!trimmedStr.startsWith("[") || !trimmedStr.endsWith("]")) {
            // 不是向量格式，可能是其他表示形式，直接返回null
            return null;
        }
        
        // 移除首尾括号
        String content = trimmedStr.substring(1, trimmedStr.length() - 1).trim();
        
        if (content.isEmpty()) {
            return new float[0];
        }
        
        // 分割向量元素
        String[] elements = content.split(",");
        float[] result = new float[elements.length];
        
        for (int i = 0; i < elements.length; i++) {
            try {
                String element = elements[i].trim();
                // 处理可能的格式化问题
                element = element.replaceAll("[\"'\\[\\]]", ""); // 移除所有引号和括号
                element = element.replaceAll("\\s+", ""); // 移除所有空格
                
                if (element.isEmpty()) {
                    result[i] = 0.0f;
                } else {
                    result[i] = Float.parseFloat(element);
                }
            } catch (NumberFormatException e) {
                // 详细的错误信息，帮助调试
                throw new SQLException(
                    String.format("Failed to parse vector element at index %d. " +
                                "Vector string: %s, Content: %s, Element: %s", 
                                i, vectorStr, content, elements[i]), 
                    e);
            }
        }
        
        return result;
    }
    
    /**
     * 将向量字符串解析为double数组
     */
    private double[] parseVectorToDoubleArray(String vectorStr) throws SQLException {
        if (vectorStr == null || vectorStr.isEmpty() || vectorStr.equals("NULL")) {
            return null;
        }
        
        // 处理可能的特殊情况
        String trimmedStr = vectorStr.trim();
        
        // 检查是否是向量格式
        if (!trimmedStr.startsWith("[") || !trimmedStr.endsWith("]")) {
            // 不是向量格式，可能是其他表示形式，直接返回null
            return null;
        }
        
        // 移除首尾括号
        String content = trimmedStr.substring(1, trimmedStr.length() - 1).trim();
        
        if (content.isEmpty()) {
            return new double[0];
        }
        
        // 分割向量元素
        String[] elements = content.split(",");
        double[] result = new double[elements.length];
        
        for (int i = 0; i < elements.length; i++) {
            try {
                String element = elements[i].trim();
                // 处理可能的格式化问题
                element = element.replaceAll("[\"'\\[\\]]", ""); // 移除所有引号和括号
                element = element.replaceAll("\\s+", ""); // 移除所有空格
                
                if (element.isEmpty()) {
                    result[i] = 0.0;
                } else {
                    result[i] = Double.parseDouble(element);
                }
            } catch (NumberFormatException e) {
                // 详细的错误信息，帮助调试
                throw new SQLException(
                    String.format("Failed to parse vector element at index %d. " +
                                "Vector string: %s, Content: %s, Element: %s", 
                                i, vectorStr, content, elements[i]), 
                    e);
            }
        }
        
        return result;
    }

    @Override
    public <T> T getObject(String columnLabel, Class<T> type) throws SQLException {
        checkClosed();
        int columnIndex = findColumn(columnLabel);
        return getObject(columnIndex, type);
    }

    public Ref getRef(int columnIndex, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getRef not supported");
    }

    public Ref getRef(String columnLabel, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getRef not supported");
    }

    public Blob getBlob(int columnIndex, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getBlob not supported");
    }

    public Blob getBlob(String columnLabel, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getBlob not supported");
    }

    public Clob getClob(int columnIndex, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getClob not supported");
    }

    public Clob getClob(String columnLabel, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getClob not supported");
    }

    public Array getArray(int columnIndex, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getArray not supported");
    }

    public Array getArray(String columnLabel, Map<String, Class<?>> map) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getArray not supported");
    }

    @Override
    public SQLXML getSQLXML(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getSQLXML not supported");
    }

    @Override
    public SQLXML getSQLXML(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getSQLXML not supported");
    }

    @Override
    public void updateSQLXML(int columnIndex, SQLXML xmlObject) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateSQLXML not supported");
    }

    @Override
    public void updateSQLXML(String columnLabel, SQLXML xmlObject) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateSQLXML not supported");
    }

    @Override
    public NClob getNClob(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getNClob not supported");
    }

    @Override
    public NClob getNClob(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getNClob not supported");
    }

    @Override
    public void updateNClob(int columnIndex, NClob nClob) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNClob not supported");
    }

    @Override
    public void updateNClob(String columnLabel, NClob nClob) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNClob not supported");
    }

    @Override
    public String getNString(int columnIndex) throws SQLException {
        checkClosed();
        return getString(columnIndex);
    }

    @Override
    public String getNString(String columnLabel) throws SQLException {
        checkClosed();
        return getString(columnLabel);
    }

    @Override
    public void updateNString(int columnIndex, String nString) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNString not supported");
    }

    @Override
    public void updateNString(String columnLabel, String nString) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNString not supported");
    }

    @Override
    public Reader getNCharacterStream(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getNCharacterStream not supported");
    }

    @Override
    public Reader getNCharacterStream(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getNCharacterStream not supported");
    }

    @Override
    public void updateNCharacterStream(int columnIndex, Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNCharacterStream not supported");
    }

    @Override
    public void updateNCharacterStream(String columnLabel, Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNCharacterStream not supported");
    }

    @Override
    public void updateAsciiStream(int columnIndex, InputStream inputStream, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateAsciiStream not supported");
    }

    @Override
    public void updateBinaryStream(int columnIndex, InputStream inputStream, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBinaryStream not supported");
    }

    @Override
    public void updateCharacterStream(int columnIndex, Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateCharacterStream not supported");
    }

    @Override
    public void updateNClob(int columnIndex, Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNClob not supported");
    }

    @Override
    public void updateNClob(String columnLabel, Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNClob not supported");
    }

    @Override
    public void updateAsciiStream(String columnLabel, InputStream inputStream, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateAsciiStream not supported");
    }

    @Override
    public void updateBinaryStream(String columnLabel, InputStream inputStream, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBinaryStream not supported");
    }

    @Override
    public void updateCharacterStream(String columnLabel, Reader reader, long length) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateCharacterStream not supported");
    }

    @Override
    public void updateNCharacterStream(int columnIndex, Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNCharacterStream not supported");
    }

    @Override
    public void updateNCharacterStream(String columnLabel, Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNCharacterStream not supported");
    }

    @Override
    public void updateAsciiStream(int columnIndex, InputStream inputStream) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateAsciiStream not supported");
    }

    @Override
    public void updateBinaryStream(int columnIndex, InputStream inputStream) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBinaryStream not supported");
    }

    @Override
    public void updateCharacterStream(int columnIndex, Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateCharacterStream not supported");
    }

    @Override
    public void updateBlob(int columnIndex, InputStream inputStream) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBlob not supported");
    }

    @Override
    public void updateClob(int columnIndex, Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateClob not supported");
    }

    @Override
    public void updateNClob(int columnIndex, Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNClob not supported");
    }

    @Override
    public void updateNClob(String columnLabel, Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateNClob not supported");
    }

    @Override
    public void updateAsciiStream(String columnLabel, InputStream inputStream) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateAsciiStream not supported");
    }

    @Override
    public void updateBinaryStream(String columnLabel, InputStream inputStream) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBinaryStream not supported");
    }

    @Override
    public void updateBlob(String columnLabel, InputStream inputStream) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateBlob not supported");
    }

    @Override
    public void updateCharacterStream(String columnLabel, Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateCharacterStream not supported");
    }

    @Override
    public void updateClob(String columnLabel, Reader reader) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateClob not supported");
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

    public void updateObject(int columnIndex, Object x, int targetSqlType, int scaleOrLength) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateObject not supported");
    }

    public void updateObject(String columnLabel, Object x, int targetSqlType, int scaleOrLength) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateObject not supported");
    }

    public void updateObject(int columnIndex, Object x, int targetSqlType) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateObject not supported");
    }

    public void updateObject(String columnLabel, Object x, int targetSqlType) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateObject not supported");
    }

    @Override
    public void updateObject(int columnIndex, Object x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateObject not supported");
    }

    @Override
    public void updateObject(String columnLabel, Object x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateObject not supported");
    }

    public long getLargeSerialVersionUID() throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getLargeSerialVersionUID not supported");
    }

    public long getLargeInt(int columnIndex) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getLargeInt not supported");
    }

    public long getLargeInt(String columnLabel) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("getLargeInt not supported");
    }

    public void updateLargeInt(int columnIndex, long x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateLargeInt not supported");
    }

    public void updateLargeInt(String columnLabel, long x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateLargeInt not supported");
    }

    public long getLargeLong(int columnIndex) throws SQLException {
        checkClosed();
        return getLong(columnIndex);
    }

    public long getLargeLong(String columnLabel) throws SQLException {
        checkClosed();
        return getLong(columnLabel);
    }

    public void updateLargeLong(int columnIndex, long x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateLargeLong not supported");
    }

    public void updateLargeLong(String columnLabel, long x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateLargeLong not supported");
    }

    public BigDecimal getLargeDecimal(int columnIndex) throws SQLException {
        checkClosed();
        return getBigDecimal(columnIndex);
    }

    public BigDecimal getLargeDecimal(String columnLabel) throws SQLException {
        checkClosed();
        return getBigDecimal(columnLabel);
    }

    public void updateLargeDecimal(int columnIndex, BigDecimal x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateLargeDecimal not supported");
    }

    public void updateLargeDecimal(String columnLabel, BigDecimal x) throws SQLException {
        checkClosed();
        throw new SQLFeatureNotSupportedException("updateLargeDecimal not supported");
    }

    private void checkClosed() throws SQLException {
        if (closed) {
            throw new SQLException("ResultSet is closed");
        }
    }

    private void checkRowIndex() throws SQLException {
        if (currentRowIndex < 0 || currentRowIndex >= rows.size()) {
            throw new SQLException("Invalid row index");
        }
    }

    private void checkColumnIndex(int columnIndex) throws SQLException {
        if (columnIndex < 1 || columnIndex > columns.size()) {
            throw new SQLException("Invalid column index: " + columnIndex);
        }
    }

    // ResultSetMetaData implementation
    private static class RemDbResultSetMetaData implements ResultSetMetaData {
        private List<String> columns;

        public RemDbResultSetMetaData(List<String> columns) {
            this.columns = columns;
        }

        @Override
        public int getColumnCount() throws SQLException {
            return columns.size();
        }

        @Override
        public boolean isAutoIncrement(int column) throws SQLException {
            return false;
        }

        @Override
        public boolean isCaseSensitive(int column) throws SQLException {
            return false;
        }

        @Override
        public boolean isSearchable(int column) throws SQLException {
            return true;
        }

        @Override
        public boolean isCurrency(int column) throws SQLException {
            return false;
        }

        @Override
        public int isNullable(int column) throws SQLException {
            return ResultSetMetaData.columnNullableUnknown;
        }

        @Override
        public boolean isSigned(int column) throws SQLException {
            return false;
        }

        @Override
        public int getColumnDisplaySize(int column) throws SQLException {
            return 255; // Default display size
        }

        @Override
        public String getColumnLabel(int column) throws SQLException {
            return getColumnName(column);
        }

        @Override
        public String getColumnName(int column) throws SQLException {
            if (column < 1 || column > columns.size()) {
                throw new SQLException("Invalid column index: " + column);
            }
            return columns.get(column - 1);
        }

        @Override
        public String getSchemaName(int column) throws SQLException {
            return "";
        }

        @Override
        public int getPrecision(int column) throws SQLException {
            return 0;
        }

        @Override
        public int getScale(int column) throws SQLException {
            return 0;
        }

        @Override
        public String getTableName(int column) throws SQLException {
            return "";
        }

        @Override
        public String getCatalogName(int column) throws SQLException {
            return "";
        }

        @Override
        public int getColumnType(int column) throws SQLException {
            return Types.VARCHAR; // Default to VARCHAR
        }

        @Override
        public String getColumnTypeName(int column) throws SQLException {
            return "VARCHAR";
        }

        @Override
        public boolean isReadOnly(int column) throws SQLException {
            return true;
        }

        @Override
        public boolean isWritable(int column) throws SQLException {
            return false;
        }

        @Override
        public boolean isDefinitelyWritable(int column) throws SQLException {
            return false;
        }

        @Override
        public String getColumnClassName(int column) throws SQLException {
            return String.class.getName();
        }

        @Override
        public <T> T unwrap(Class<T> iface) throws SQLException {
            if (iface.isInstance(this)) {
                return iface.cast(this);
            }
            throw new SQLException("Cannot unwrap to " + iface.getName());
        }

        @Override
        public boolean isWrapperFor(Class<?> iface) throws SQLException {
            return iface.isInstance(this);
        }
    }
}