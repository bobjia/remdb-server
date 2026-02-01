"""Validators for RemDB test results"""

import re
from typing import Any, List, Dict, Union, Optional
import numpy as np

def validate_sql_result_structure(result_set, expected_columns: List[str] = None):
    """Validate that a SQL result set has the expected structure"""
    assert hasattr(result_set, 'get_columns'), "Result set must have get_columns method"
    assert hasattr(result_set, 'get_rows_count'), "Result set must have get_rows_count method"
    assert hasattr(result_set, '__iter__'), "Result set must be iterable"
    
    columns = result_set.get_columns()
    if expected_columns:
        assert len(columns) == len(expected_columns), \
            f"Expected {len(expected_columns)} columns, got {len(columns)}"
        for i, (actual, expected) in enumerate(zip(columns, expected_columns)):
            assert actual == expected, f"Column {i} mismatch: expected '{expected}', got '{actual}'"
    
    return True

def validate_row_data(row: Dict[str, Any], expected_schema: Dict[str, type] = None):
    """Validate that a row matches expected schema and data types"""
    assert isinstance(row, dict), "Row must be a dictionary"
    
    if expected_schema:
        for col_name, expected_type in expected_schema.items():
            assert col_name in row, f"Missing column '{col_name}' in row"
            
            value = row[col_name]
            if expected_type is not None and value is not None:
                # Handle optional types
                if hasattr(expected_type, '__origin__') and expected_type.__origin__ is Union:
                    # Union type, check if any of the types match
                    type_args = expected_type.__args__
                    if type(None) in type_args:
                        # Optional type, remove None from check
                        type_args = [t for t in type_args if t is not type(None)]
                    
                    matches = any(isinstance(value, t) for t in type_args)
                    assert matches, f"Column '{col_name}' value {value} (type {type(value)}) does not match any of {type_args}"
                else:
                    # Simple type check
                    assert isinstance(value, expected_type), \
                        f"Column '{col_name}' value {value} is type {type(value)}, expected {expected_type}"
    
    return True

def validate_vector_distance(
    vector1: List[float], 
    vector2: List[float], 
    distance_type: str = 'L2',
    tolerance: float = 1e-6
) -> float:
    """Calculate and validate vector distance"""
    assert len(vector1) == len(vector2), "Vectors must have same dimensions"
    
    if distance_type == 'L2':
        # Euclidean distance
        distance = np.linalg.norm(np.array(vector1) - np.array(vector2))
    elif distance_type == 'IP':
        # Inner product (dot product)
        distance = np.dot(vector1, vector2)
    elif distance_type == 'COSINE':
        # Cosine similarity
        dot = np.dot(vector1, vector2)
        norm1 = np.linalg.norm(vector1)
        norm2 = np.linalg.norm(vector2)
        if norm1 > 0 and norm2 > 0:
            distance = dot / (norm1 * norm2)
        else:
            distance = 0.0
    else:
        raise ValueError(f"Unsupported distance type: {distance_type}")
    
    return distance

def validate_timeseries_data(
    data: List[Dict[str, Any]],
    time_field: str = 'timestamp',
    value_field: str = 'value',
    check_monotonic: bool = True
) -> bool:
    """Validate time series data properties"""
    assert len(data) > 0, "Time series data cannot be empty"
    
    timestamps = [row[time_field] for row in data]
    values = [row[value_field] for row in data]
    
    # Check that timestamps are integers (milliseconds)
    for ts in timestamps:
        assert isinstance(ts, int), f"Timestamp {ts} must be integer"
        assert ts > 0, f"Timestamp {ts} must be positive"
    
    # Check monotonic increasing timestamps
    if check_monotonic:
        for i in range(1, len(timestamps)):
            assert timestamps[i] >= timestamps[i-1], \
                f"Timestamps not monotonic: {timestamps[i-1]} -> {timestamps[i]}"
    
    # Check values are numeric
    for val in values:
        assert isinstance(val, (int, float)), f"Value {val} must be numeric"
    
    return True

def validate_index_properties(
    index_info: Dict[str, Any],
    expected_type: str = None,
    expected_column: str = None,
    expected_params: Dict[str, Any] = None
) -> bool:
    """Validate index properties"""
    assert 'name' in index_info, "Index info must have 'name' field"
    assert 'type' in index_info, "Index info must have 'type' field"
    assert 'column' in index_info, "Index info must have 'column' field"
    
    if expected_type:
        assert index_info['type'] == expected_type, \
            f"Index type mismatch: expected '{expected_type}', got '{index_info['type']}'"
    
    if expected_column:
        assert index_info['column'] == expected_column, \
            f"Index column mismatch: expected '{expected_column}', got '{index_info['column']}'"
    
    if expected_params:
        for param_name, expected_value in expected_params.items():
            assert param_name in index_info, f"Missing index parameter '{param_name}'"
            actual_value = index_info[param_name]
            
            # Allow tolerance for floating point comparisons
            if isinstance(expected_value, float) and isinstance(actual_value, (int, float)):
                assert abs(actual_value - expected_value) < 1e-6, \
                    f"Index parameter '{param_name}' mismatch: expected {expected_value}, got {actual_value}"
            else:
                assert actual_value == expected_value, \
                    f"Index parameter '{param_name}' mismatch: expected {expected_value}, got {actual_value}"
    
    return True

def validate_sql_error_message(
    error_message: str,
    expected_keywords: List[str] = None,
    expected_pattern: str = None
) -> bool:
    """Validate SQL error message contains expected information"""
    assert error_message, "Error message cannot be empty"
    
    if expected_keywords:
        for keyword in expected_keywords:
            assert keyword.lower() in error_message.lower(), \
                f"Error message missing keyword '{keyword}': {error_message}"
    
    if expected_pattern:
        pattern = re.compile(expected_pattern, re.IGNORECASE)
        assert pattern.search(error_message) is not None, \
            f"Error message does not match pattern '{expected_pattern}': {error_message}"
    
    return True

def validate_transaction_properties(
    transaction_info: Dict[str, Any],
    expected_active: bool = None,
    expected_isolation_level: str = None
) -> bool:
    """Validate transaction properties"""
    assert 'id' in transaction_info, "Transaction info must have 'id' field"
    assert isinstance(transaction_info['id'], int), "Transaction ID must be integer"
    
    if expected_active is not None:
        assert 'active' in transaction_info, "Transaction info must have 'active' field"
        assert transaction_info['active'] == expected_active, \
            f"Transaction active state mismatch: expected {expected_active}, got {transaction_info['active']}"
    
    if expected_isolation_level is not None:
        assert 'isolation_level' in transaction_info, "Transaction info must have 'isolation_level' field"
        assert transaction_info['isolation_level'] == expected_isolation_level, \
            f"Transaction isolation level mismatch: expected '{expected_isolation_level}', " \
            f"got '{transaction_info['isolation_level']}'"
    
    return True

def compare_result_sets(
    actual_result_set,
    expected_result_set,
    tolerance: float = 1e-6,
    ignore_column_order: bool = False
) -> bool:
    """Compare two result sets for equality"""
    # Compare row counts
    actual_rows = list(actual_result_set)
    expected_rows = list(expected_result_set)
    
    assert len(actual_rows) == len(expected_rows), \
        f"Row count mismatch: expected {len(expected_rows)}, got {len(actual_rows)}"
    
    # Compare columns
    actual_columns = actual_result_set.get_columns()
    expected_columns = expected_result_set.get_columns()
    
    if ignore_column_order:
        assert set(actual_columns) == set(expected_columns), \
            f"Column set mismatch: expected {set(expected_columns)}, got {set(actual_columns)}"
        # Reorder actual columns to match expected order for row comparison
        column_mapping = {col: actual_columns.index(col) for col in expected_columns}
        actual_rows = [
            {col: row[actual_columns[column_mapping[col]]] for col in expected_columns}
            for row in actual_rows
        ]
    else:
        assert actual_columns == expected_columns, \
            f"Column order mismatch: expected {expected_columns}, got {actual_columns}"
    
    # Compare each row
    for i, (actual_row, expected_row) in enumerate(zip(actual_rows, expected_rows)):
        for col in expected_columns:
            actual_value = actual_row[col]
            expected_value = expected_row[col]
            
            # Handle floating point comparisons with tolerance
            if isinstance(actual_value, float) and isinstance(expected_value, (int, float)):
                assert abs(actual_value - expected_value) < tolerance, \
                    f"Row {i}, column '{col}' value mismatch: expected {expected_value}, got {actual_value}"
            elif isinstance(actual_value, list) and isinstance(expected_value, list):
                # Compare lists (e.g., vectors)
                assert len(actual_value) == len(expected_value), \
                    f"Row {i}, column '{col}' list length mismatch: expected {len(expected_value)}, got {len(actual_value)}"
                for j, (a, b) in enumerate(zip(actual_value, expected_value)):
                    if isinstance(a, float) and isinstance(b, (int, float)):
                        assert abs(a - b) < tolerance, \
                            f"Row {i}, column '{col}'[{j}] value mismatch: expected {b}, got {a}"
                    else:
                        assert a == b, \
                            f"Row {i}, column '{col}'[{j}] value mismatch: expected {b}, got {a}"
            else:
                assert actual_value == expected_value, \
                    f"Row {i}, column '{col}' value mismatch: expected {expected_value}, got {actual_value}"
    
    return True