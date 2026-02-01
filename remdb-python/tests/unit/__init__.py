"""Unit tests for RemDB Python bindings"""

from .test_data_types import (
    TestDataTypeINTEGER,
    TestDataTypeREAL,
    TestDataTypeTEXT,
    TestDataTypeBOOLEAN,
    TestDataTypeTIMESTAMP,
    TestDataTypeVECTOR,
    TestDataTypeCombinations,
    TestDataTypeEdgeCases
)

from .test_ddl import (
    TestCreateTable,
    TestAlterTable,
    TestDropTable,
    TestCreateTimeseriesTable,
    TestShowTables,
    TestDatabaseManagement
)

from .test_dml import (
    TestInsertStatement,
    TestSelectStatement,
    TestUpdateStatement,
    TestDeleteStatement
)

from .test_functions import (
    TestAggregateFunctions,
    TestStringFunctions,
    TestMathFunctions,
    TestTimeFunctions,
    TestMovingWindowFunctions,
    TestMiscellaneousFunctions
)

from .test_operators import (
    TestComparisonOperators,
    TestLogicalOperators,
    TestLikeOperator,
    TestVectorDistanceOperators,
    TestArithmeticOperators,
    TestOperatorPrecedence
)

from .test_indexes import (
    TestScalarIndexes,
    TestVectorIndexes,
    TestIndexBuildStatus,
    TestReindex,
    TestIndexUsage
)

from .test_timeseries import (
    TestTimeSeriesOperations,
    TestMovingWindowFunctions,
    TestTimeSeriesCompression,
    TestTimeSeriesEdgeCases
)

from .test_vectors import (
    TestVectorDataTypes,
    TestVectorSearchFunctions,
    TestVectorMixedSearch,
    TestVectorEdgeCases,
    TestVectorPerformance
)

from .test_transactions import (
    TestBasicTransactions,
    TestTransactionIsolation,
    TestTransactionErrorHandling,
    TestTransactionConcurrency,
    TestTransactionEdgeCases
)

__all__ = [
    # Data types
    'TestDataTypeINTEGER',
    'TestDataTypeREAL',
    'TestDataTypeTEXT',
    'TestDataTypeBOOLEAN',
    'TestDataTypeTIMESTAMP',
    'TestDataTypeVECTOR',
    'TestDataTypeCombinations',
    'TestDataTypeEdgeCases',
    
    # DDL
    'TestCreateTable',
    'TestAlterTable',
    'TestDropTable',
    'TestCreateTimeseriesTable',
    'TestShowTables',
    'TestDatabaseManagement',
    
    # DML
    'TestInsertStatement',
    'TestSelectStatement',
    'TestUpdateStatement',
    'TestDeleteStatement',
    
    # Functions
    'TestAggregateFunctions',
    'TestStringFunctions',
    'TestMathFunctions',
    'TestTimeFunctions',
    'TestMovingWindowFunctions',
    'TestMiscellaneousFunctions',
    
    # Operators
    'TestComparisonOperators',
    'TestLogicalOperators',
    'TestLikeOperator',
    'TestVectorDistanceOperators',
    'TestArithmeticOperators',
    'TestOperatorPrecedence',
    
    # Indexes
    'TestScalarIndexes',
    'TestVectorIndexes',
    'TestIndexBuildStatus',
    'TestReindex',
    'TestIndexUsage',
    
    # Time Series
    'TestTimeSeriesOperations',
    'TestMovingWindowFunctions',
    'TestTimeSeriesCompression',
    'TestTimeSeriesEdgeCases',
    
    # Vectors
    'TestVectorDataTypes',
    'TestVectorSearchFunctions',
    'TestVectorMixedSearch',
    'TestVectorEdgeCases',
    'TestVectorPerformance',
    
    # Transactions
    'TestBasicTransactions',
    'TestTransactionIsolation',
    'TestTransactionErrorHandling',
    'TestTransactionConcurrency',
    'TestTransactionEdgeCases'
]