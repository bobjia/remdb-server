"""RemDB Python bindings"""

__version__ = "0.1.0"

from .core import (
    connect,
    RemDbConnection,
    RemDbTable,
    RemDbTransaction,
    RemDbResultSet,
    RemDbError,
    NotFoundError,
    TransactionError,
    ConfigError
)

__all__ = [
    "connect",
    "RemDbConnection",
    "RemDbTable",
    "RemDbTransaction",
    "RemDbResultSet",
    "RemDbError",
    "NotFoundError",
    "TransactionError",
    "ConfigError"
]
