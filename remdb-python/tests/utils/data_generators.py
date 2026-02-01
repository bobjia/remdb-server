"""Data generators for RemDB test cases"""

import random
import string
import datetime
from typing import List, Dict, Any, Tuple, Union
import numpy as np

def generate_random_string(length: int = 10) -> str:
    """Generate a random string of specified length"""
    letters = string.ascii_letters + string.digits
    return ''.join(random.choice(letters) for _ in range(length))

def generate_random_int(min_val: int = 0, max_val: int = 1000) -> int:
    """Generate a random integer within range"""
    return random.randint(min_val, max_val)

def generate_random_float(min_val: float = 0.0, max_val: float = 1000.0) -> float:
    """Generate a random float within range"""
    return random.uniform(min_val, max_val)

def generate_random_bool() -> bool:
    """Generate a random boolean value"""
    return random.choice([True, False])

def generate_random_timestamp(start_year: int = 2020, end_year: int = 2024) -> int:
    """Generate a random timestamp in milliseconds"""
    start_date = datetime.datetime(start_year, 1, 1)
    end_date = datetime.datetime(end_year, 12, 31, 23, 59, 59)
    
    delta = end_date - start_date
    random_days = random.randint(0, delta.days)
    random_seconds = random.randint(0, 86400 - 1)
    random_microseconds = random.randint(0, 999999)
    
    random_date = start_date + datetime.timedelta(
        days=random_days,
        seconds=random_seconds,
        microseconds=random_microseconds
    )
    
    # Convert to milliseconds since epoch
    return int(random_date.timestamp() * 1000)

def generate_random_vector(dimensions: int = 128, min_val: float = -1.0, max_val: float = 1.0) -> List[float]:
    """Generate a random vector of specified dimensions"""
    return [random.uniform(min_val, max_val) for _ in range(dimensions)]

def generate_normalized_vector(dimensions: int = 128) -> List[float]:
    """Generate a random normalized vector (unit length)"""
    vector = generate_random_vector(dimensions)
    norm = np.linalg.norm(vector)
    if norm > 0:
        vector = [v / norm for v in vector]
    return vector

def generate_test_table_data(
    num_rows: int = 100,
    schema: Dict[str, str] = None
) -> List[Dict[str, Any]]:
    """
    Generate test data for a table based on schema
    
    Args:
        num_rows: Number of rows to generate
        schema: Dictionary mapping column names to data types
                Supported types: 'INTEGER', 'REAL', 'TEXT', 'BOOLEAN', 'TIMESTAMP', 'VECTOR'
    
    Returns:
        List of dictionaries representing rows
    """
    if schema is None:
        # Default schema for testing
        schema = {
            'id': 'INTEGER',
            'name': 'TEXT',
            'age': 'INTEGER',
            'salary': 'REAL',
            'active': 'BOOLEAN',
            'created_at': 'TIMESTAMP'
        }
    
    data = []
    for i in range(num_rows):
        row = {}
        for col_name, col_type in schema.items():
            if col_name == 'id':
                row[col_name] = i + 1
            elif col_type == 'INTEGER':
                row[col_name] = generate_random_int(1, 100)
            elif col_type == 'REAL':
                row[col_name] = generate_random_float(1000.0, 10000.0)
            elif col_type == 'TEXT':
                row[col_name] = generate_random_string(20)
            elif col_type == 'BOOLEAN':
                row[col_name] = generate_random_bool()
            elif col_type == 'TIMESTAMP':
                row[col_name] = generate_random_timestamp()
            elif col_type == 'VECTOR':
                # Determine dimensions from column name if specified
                # e.g., 'vector_128' -> 128 dimensions
                if '_' in col_name:
                    try:
                        dim = int(col_name.split('_')[1])
                    except (ValueError, IndexError):
                        dim = 128
                else:
                    dim = 128
                row[col_name] = generate_random_vector(dim)
            else:
                # Default to TEXT
                row[col_name] = generate_random_string(10)
        
        data.append(row)
    
    return data

def generate_timeseries_data(
    num_points: int = 1000,
    start_time: int = None,
    interval_ms: int = 60000,  # 1 minute intervals
    value_range: Tuple[float, float] = (0.0, 100.0),
    tags: Dict[str, Union[str, int]] = None
) -> List[Dict[str, Any]]:
    """
    Generate time series test data
    
    Args:
        num_points: Number of data points
        start_time: Starting timestamp in milliseconds (default: current time - num_points * interval)
        interval_ms: Time interval between points in milliseconds
        value_range: Range for value generation
        tags: Dictionary of tag key-value pairs
    
    Returns:
        List of dictionaries with 'timestamp' and 'value' keys
    """
    if start_time is None:
        # Default to current time minus the total span
        current_time = int(datetime.datetime.now().timestamp() * 1000)
        start_time = current_time - (num_points * interval_ms)
    
    if tags is None:
        tags = {'sensor_id': 1, 'location': 'room1'}
    
    data = []
    for i in range(num_points):
        timestamp = start_time + (i * interval_ms)
        value = random.uniform(value_range[0], value_range[1])
        
        row = {
            'timestamp': timestamp,
            'value': value,
            **tags
        }
        
        data.append(row)
    
    return data

def generate_vector_test_data(
    num_vectors: int = 1000,
    dimensions: int = 128,
    metadata_fields: List[str] = None
) -> List[Dict[str, Any]]:
    """
    Generate test data for vector search tests
    
    Args:
        num_vectors: Number of vectors to generate
        dimensions: Vector dimensions
        metadata_fields: List of metadata field names
    
    Returns:
        List of dictionaries with 'id', 'vector', and metadata fields
    """
    if metadata_fields is None:
        metadata_fields = ['category', 'label', 'description']
    
    data = []
    for i in range(num_vectors):
        vector = generate_normalized_vector(dimensions)
        metadata = {}
        
        for field in metadata_fields:
            if field == 'category':
                metadata[field] = random.choice(['A', 'B', 'C', 'D'])
            elif field == 'label':
                metadata[field] = f"label_{random.randint(1, 10)}"
            elif field == 'description':
                metadata[field] = generate_random_string(50)
            else:
                metadata[field] = generate_random_string(10)
        
        row = {
            'id': i + 1,
            'vector': vector,
            **metadata
        }
        
        data.append(row)
    
    return data

def create_sample_datasets() -> Dict[str, Any]:
    """Create a collection of sample datasets for testing"""
    return {
        'users': generate_test_table_data(50, {
            'id': 'INTEGER',
            'username': 'TEXT',
            'email': 'TEXT',
            'age': 'INTEGER',
            'active': 'BOOLEAN',
            'signup_date': 'TIMESTAMP'
        }),
        'products': generate_test_table_data(100, {
            'id': 'INTEGER',
            'name': 'TEXT',
            'category': 'TEXT',
            'price': 'REAL',
            'stock': 'INTEGER',
            'available': 'BOOLEAN'
        }),
        'sensor_readings': generate_timeseries_data(500, tags={
            'sensor_id': 1,
            'location': 'factory_floor',
            'unit': 'celsius'
        }),
        'vectors': generate_vector_test_data(200, dimensions=64)
    }