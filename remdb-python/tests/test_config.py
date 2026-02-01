"""
Test configuration for RemDB Python bindings.

This file contains configuration options to optimize test performance.
"""

# Test optimization settings
TEST_OPTIONS = {
    # General test settings
    'timeout_seconds': 30,  # Default test timeout
    'retry_attempts': 3,     # Number of retry attempts for flaky tests
    'sleep_interval': 0.1,   # Sleep interval between retries (seconds)
    
    # Data generation settings
    'default_num_rows': 100,         # Default number of rows for test tables
    'default_num_vectors': 100,      # Default number of vectors for vector tests
    'default_num_timeseries': 1000,  # Default number of timeseries points
    
    # Vector settings
    'default_vector_dimensions': 128,  # Default vector dimensions
    'min_vector_dimensions': 8,        # Minimum vector dimensions for testing
    'max_vector_dimensions': 256,      # Maximum vector dimensions for testing
    
    # Time series settings
    'default_timeseries_interval': 60000,  # Default interval in milliseconds (1 minute)
    'min_timeseries_interval': 1000,       # Minimum interval (1 second)
    'max_timeseries_interval': 3600000,    # Maximum interval (1 hour)
    
    # Index settings
    'index_build_timeout': 60,  # Timeout for index building (seconds)
    'index_test_size': 1000,    # Number of records for index performance tests
    
    # Transaction settings
    'max_transaction_operations': 1000,  # Maximum operations per transaction
    'transaction_test_size': 100,        # Number of records for transaction tests
    
    # Network settings
    'network_timeout': 10,      # Network operation timeout (seconds)
    'network_retries': 3,       # Number of network retry attempts
    'network_retry_delay': 1,   # Delay between network retries (seconds)
    
    # Memory optimization
    'use_memory_efficient_testing': True,  # Use memory-efficient testing strategies
    'max_memory_usage': 512,               # Maximum memory usage per test (MB)
    
    # Parallel testing
    'enable_parallel_testing': False,  # Enable parallel test execution
    'max_parallel_workers': 4,         # Maximum number of parallel workers
    
    # Logging
    'enable_test_logging': False,      # Enable detailed test logging
    'log_level': 'INFO',               # Log level for test logging
    
    # Test isolation
    'isolate_tests': True,             # Isolate tests from each other
    'cleanup_between_tests': True,     # Clean up resources between tests
}

# Test data generation defaults
TEST_DATA_DEFAULTS = {
    'scalar_data': {
        'integer_range': (1, 1000),
        'real_range': (0.0, 1000.0),
        'text_length': (1, 100),
        'boolean_ratio': 0.5,  # 50% True, 50% False
    },
    
    'vector_data': {
        'value_range': (-1.0, 1.0),
        'normalized': True,  # Use normalized vectors by default
        'metadata_fields': ['category', 'label', 'description'],
    },
    
    'timeseries_data': {
        'value_range': (0.0, 100.0),
        'trend': 'random',  # random, increasing, decreasing, seasonal
        'noise_level': 0.1,  # Amount of noise to add (0.0-1.0)
        'seasonality': None,  # Seasonality pattern (None, daily, weekly, monthly)
    },
}

# Test skip conditions
TEST_SKIP_CONDITIONS = {
    'skip_network_tests': False,  # Skip network tests by default
    'skip_vector_tests': False,   # Skip vector tests by default
    'skip_timeseries_tests': False,  # Skip timeseries tests by default
    'skip_index_tests': False,    # Skip index tests by default
    'skip_transaction_tests': False,  # Skip transaction tests by default
}

# Test environment detection
def detect_test_environment():
    """
    Detect test environment and adjust settings accordingly.
    
    Returns:
        dict: Adjusted test options based on environment
    """
    import os
    import sys
    
    adjusted_options = TEST_OPTIONS.copy()
    
    # Detect CI environment
    if os.environ.get('CI') or os.environ.get('CONTINUOUS_INTEGRATION'):
        adjusted_options['enable_test_logging'] = True
        adjusted_options['timeout_seconds'] = 60  # Longer timeouts in CI
        adjusted_options['enable_parallel_testing'] = True
    
    # Detect resource-constrained environments
    if os.environ.get('RESOURCE_CONSTRAINED'):
        adjusted_options['default_num_rows'] = 10
        adjusted_options['default_num_vectors'] = 10
        adjusted_options['default_num_timeseries'] = 100
        adjusted_options['max_parallel_workers'] = 2
    
    # Detect Windows environment
    if sys.platform == 'win32':
        # Windows-specific optimizations
        adjusted_options['sleep_interval'] = 0.2  # Longer sleep intervals on Windows
    
    return adjusted_options

# Get current test options
CURRENT_OPTIONS = detect_test_environment()

# Helper functions
def get_option(key, default=None):
    """
    Get a test option with optional default value.
    
    Args:
        key: Option key
        default: Default value if key not found
        
    Returns:
        Option value or default
    """
    return CURRENT_OPTIONS.get(key, default)

def get_data_default(key, default=None):
    """
    Get a test data default value.
    
    Args:
        key: Data default key (can be nested, e.g., 'scalar_data.integer_range')
        default: Default value if key not found
        
    Returns:
        Data default value or default
    """
    import functools
    import operator
    
    try:
        keys = key.split('.')
        return functools.reduce(operator.getitem, keys, TEST_DATA_DEFAULTS)
    except (KeyError, AttributeError):
        return default

def should_skip_test(test_category):
    """
    Determine if a test category should be skipped.
    
    Args:
        test_category: Test category to check
        
    Returns:
        bool: True if test should be skipped
    """
    skip_key = f'skip_{test_category}_tests'
    return TEST_SKIP_CONDITIONS.get(skip_key, False)