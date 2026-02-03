"""RemDB Python bindings"""

__version__ = "0.1.0"

from .core import (
    connect,
    create_pubsub,
    RemDbConnection,
    RemDbTable,
    RemDbTransaction,
    RemDbResultSet,
    RemDbPubSub,
    RemDbError,
    NotFoundError,
    TransactionError,
    ConfigError
)

__all__ = [
    "connect",
    "create_pubsub",
    "RemDbConnection",
    "RemDbTable",
    "RemDbTransaction",
    "RemDbResultSet",
    "RemDbPubSub",
    "RemDbError",
    "NotFoundError",
    "TransactionError",
    "ConfigError"
]
