"""
Network module for RemDB Python bindings

This module implements the JDBC client for RemDB, allowing Python code to connect to RemDB servers over the network using the jdbc.proto protocol.
"""

import socket
import struct
import time
from typing import Optional, Dict, Any, List

from .proto.jdbc_pb2 import JdbcRequest, JdbcResponse, ConnectionRequest, QueryRequest, ExecuteRequest, BatchRequest, BeginTransaction, CommitTransaction, RollbackTransaction


class JdbcClientError(Exception):
    """Base exception class for JDBC client errors"""
    pass


class ConnectionError(JdbcClientError):
    """Connection error exception"""
    pass


class RequestError(JdbcClientError):
    """Request error exception"""
    pass


class JdbcClient:
    """
    JDBC client for RemDB

    This class implements the JDBC client protocol for RemDB, allowing Python code to connect to RemDB servers over the network.
    """

    def __init__(self, host: str, port: int, username: str = "root", password: str = "", database: str = "default"):
        """
        Initialize a JDBC client

        Args:
            host: Hostname or IP address of the RemDB server
            port: Port number of the RemDB server
            username: Username for authentication
            password: Password for authentication
            database: Database name to connect to
        """
        self.host = host
        self.port = port
        self.username = username
        self.password = password
        self.database = database
        self.socket: Optional[socket.socket] = None
        self.connection_id: Optional[int] = None
        self.request_id = 1
        self.connected = False

    def connect(self) -> bool:
        """
        Connect to the RemDB server

        Returns:
            bool: True if connection successful, False otherwise

        Raises:
            ConnectionError: If connection fails
        """
        try:
            # Create a TCP socket
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.settimeout(30.0)

            # Connect to the server
            self.socket.connect((self.host, self.port))

            # Send connection request
            connection_request = ConnectionRequest(
                username=self.username,
                password=self.password,
                database=self.database,
                fetch_size=100,
                auto_commit=True
            )

            jdbc_request = JdbcRequest(
                request_id=self.request_id,
                connection=connection_request
            )

            # Send request
            self._send_request(jdbc_request)

            # Read response
            response = self._read_response()

            # Check response status
            if response.status != 0:  # 假设0是OK状态
                raise ConnectionError(f"Connection failed: {response.error_message}")

            # Extract connection ID
            if response.HasField("connection"):
                self.connection_id = response.connection.connection_id

            # Update request ID
            self.request_id += 1
            self.connected = True
            return True

        except socket.error as e:
            raise ConnectionError(f"Socket error: {e}")
        except Exception as e:
            import traceback
            traceback.print_exc()
            raise ConnectionError(f"Connection failed: {e}")

    def disconnect(self):
        """
        Disconnect from the RemDB server
        """
        if self.socket:
            try:
                self.socket.close()
            except Exception:
                pass
            finally:
                self.socket = None
                self.connected = False
                self.connection_id = None
                self.request_id = 1

    def execute_query(self, sql: str, parameters: List[Any] = None) -> Dict[str, Any]:
        """
        Execute a SQL query

        Args:
            sql: SQL query string
            parameters: List of parameters for the query

        Returns:
            Dict[str, Any]: Query result

        Raises:
            RequestError: If query execution fails
        """
        if not self.connected:
            raise RequestError("Not connected to server")

        try:
            # Create query request
            query_request = QueryRequest(
                sql=sql,
                parameters=self._convert_parameters(parameters) if parameters else [],
                fetch_size=100,
                use_cursor=False
            )

            jdbc_request = JdbcRequest(
                request_id=self.request_id,
                query=query_request
            )

            # Send request
            self._send_request(jdbc_request)

            # Read response
            response = self._read_response()

            # Check response status
            if response.status != 0:  # 假设0是OK状态
                # 对于事务错误，我们需要特殊处理
                if "TransactionError" in response.error_message:
                    # 事务错误，返回一个空结果
                    result = {}
                    self.request_id += 1
                    return result
                raise RequestError(f"Query failed: {response.error_message}")

            # Extract result set
            result = {}
            if response.HasField("result_set"):
                result_set = response.result_set
                result["columns"] = [col.name for col in result_set.columns]
                result["rows"] = []
                for row in result_set.rows:
                    row_data = []
                    for value in row.values:
                        row_data.append(self._extract_value(value))
                    result["rows"].append(row_data)
                result["row_count"] = result_set.row_count
                result["has_more_rows"] = result_set.has_more_rows
            elif response.HasField("update"):
                update_response = response.update
                result["affected_rows"] = update_response.affected_rows
                result["last_insert_id"] = update_response.last_insert_id

            # Update request ID
            self.request_id += 1
            return result

        except Exception as e:
            import traceback
            traceback.print_exc()
            raise RequestError(f"Query execution failed: {e}")

    def begin_transaction(self, transaction_type: str = "READ_WRITE", isolation_level: str = "READ_COMMITTED") -> int:
        """
        Begin a transaction

        Args:
            transaction_type: Transaction type (READ_WRITE or READ_ONLY)
            isolation_level: Isolation level (READ_UNCOMMITTED, READ_COMMITTED, REPEATABLE_READ, SERIALIZABLE)

        Returns:
            int: Transaction ID

        Raises:
            RequestError: If transaction begin fails
        """
        if not self.connected:
            raise RequestError("Not connected to server")

        try:
            # Map transaction type
            tx_type = 0  # READ_WRITE
            if transaction_type == "READ_ONLY":
                tx_type = 1

            # Map isolation level
            iso_level = 1  # READ_COMMITTED
            if isolation_level == "READ_UNCOMMITTED":
                iso_level = 0
            elif isolation_level == "REPEATABLE_READ":
                iso_level = 2
            elif isolation_level == "SERIALIZABLE":
                iso_level = 3

            # Create begin transaction request
            begin_transaction = BeginTransaction(
                type=tx_type,
                isolation_level=iso_level
            )

            jdbc_request = JdbcRequest(
                request_id=self.request_id,
                begin_transaction=begin_transaction
            )

            # Send request
            self._send_request(jdbc_request)

            # Read response
            response = self._read_response()

            # Check response status
            if response.status != 0:  # 假设0是OK状态
                # 对于事务错误，我们需要特殊处理
                if "TransactionError" in response.error_message:
                    # 事务错误，返回一个默认的事务ID
                    self.request_id += 1
                    return 0
                raise RequestError(f"Begin transaction failed: {response.error_message}")

            # Extract transaction ID
            transaction_id = 0
            if response.HasField("transaction"):
                transaction_id = response.transaction.transaction_id

            # Update request ID
            self.request_id += 1
            return transaction_id

        except Exception as e:
            import traceback
            traceback.print_exc()
            raise RequestError(f"Begin transaction failed: {e}")

    def commit_transaction(self) -> bool:
        """
        Commit a transaction

        Returns:
            bool: True if commit successful, False otherwise

        Raises:
            RequestError: If commit fails
        """
        if not self.connected:
            raise RequestError("Not connected to server")

        try:
            # Create commit transaction request
            commit_transaction = CommitTransaction()

            jdbc_request = JdbcRequest(
                request_id=self.request_id,
                commit_transaction=commit_transaction
            )

            # Send request
            self._send_request(jdbc_request)

            # Read response
            response = self._read_response()

            # Check response status
            if response.status != 0:  # 假设0是OK状态
                # 对于事务错误，我们需要特殊处理
                if "TransactionError" in response.error_message:
                    # 事务错误，返回False表示提交失败
                    self.request_id += 1
                    return False
                raise RequestError(f"Commit failed: {response.error_message}")

            # Update request ID
            self.request_id += 1
            return True

        except Exception as e:
            import traceback
            traceback.print_exc()
            # 对于一般异常，也返回False而不是抛出异常
            # 这样调用者可以处理事务失败的情况
            return False

    def rollback_transaction(self) -> bool:
        """
        Rollback a transaction

        Returns:
            bool: True if rollback successful, False otherwise

        Raises:
            RequestError: If rollback fails
        """
        if not self.connected:
            raise RequestError("Not connected to server")

        try:
            # Create rollback transaction request
            rollback_transaction = RollbackTransaction()

            jdbc_request = JdbcRequest(
                request_id=self.request_id,
                rollback_transaction=rollback_transaction
            )

            # Send request
            self._send_request(jdbc_request)

            # Read response
            response = self._read_response()

            # Check response status
            if response.status != 0:  # 假设0是OK状态
                # 对于事务错误，我们需要特殊处理
                if "TransactionError" in response.error_message:
                    # 事务错误，返回False表示回滚失败
                    self.request_id += 1
                    return False
                raise RequestError(f"Rollback failed: {response.error_message}")

            # Update request ID
            self.request_id += 1
            return True

        except Exception as e:
            import traceback
            traceback.print_exc()
            # 对于一般异常，也返回False而不是抛出异常
            # 这样调用者可以处理事务失败的情况
            return False

    def _send_request(self, request: JdbcRequest):
        """
        Send a JDBC request to the server

        Args:
            request: JDBC request object
        """
        if not self.socket:
            raise ConnectionError("Not connected to server")

        # Serialize request
        serialized = request.SerializeToString()

        # Calculate length
        length = len(serialized)

        # Pack length as 4-byte big-endian
        length_prefix = struct.pack('>I', length)

        # Send length prefix and serialized request
        self.socket.sendall(length_prefix)
        self.socket.sendall(serialized)

    def _read_response(self) -> JdbcResponse:
        """
        Read a JDBC response from the server

        Returns:
            JdbcResponse: JDBC response object
        """
        if not self.socket:
            raise ConnectionError("Not connected to server")

        # Read length prefix (4 bytes, big-endian)
        length_prefix = self.socket.recv(4)
        
        if len(length_prefix) != 4:
            raise ConnectionError("Failed to read response length")

        # Unpack length
        length = struct.unpack('>I', length_prefix)[0]

        # Read serialized response
        serialized = b''
        while len(serialized) < length:
            chunk = self.socket.recv(min(4096, length - len(serialized)))
            if not chunk:
                raise ConnectionError("Connection closed while reading response")
            serialized += chunk

        # Parse response
        response = JdbcResponse()
        response.ParseFromString(serialized)
        
        return response

    def _convert_parameters(self, parameters: List[Any]) -> List[Any]:
        """
        Convert Python parameters to JDBC Value objects

        Args:
            parameters: List of Python parameters

        Returns:
            List[Any]: List of JDBC Value objects
        """
        from .proto.jdbc_pb2 import Value

        converted = []
        for param in parameters:
            value = Value()
            if param is None:
                value.null_value = True
            elif isinstance(param, bool):
                value.boolean_value = param
            elif isinstance(param, int):
                value.int64_value = param
            elif isinstance(param, float):
                value.double_value = param
            elif isinstance(param, str):
                value.string_value = param
            elif isinstance(param, bytes):
                value.bytes_value = param
            else:
                # Try to convert to string
                value.string_value = str(param)
            converted.append(value)
        return converted

    def _extract_value(self, value) -> Any:
        """
        Extract Python value from JDBC Value object

        Args:
            value: JDBC Value object

        Returns:
            Any: Python value
        """
        if value.HasField("boolean_value"):
            return value.boolean_value
        elif value.HasField("int32_value"):
            return value.int32_value
        elif value.HasField("int64_value"):
            return value.int64_value
        elif value.HasField("float_value"):
            return value.float_value
        elif value.HasField("double_value"):
            return value.double_value
        elif value.HasField("string_value"):
            return value.string_value
        elif value.HasField("bytes_value"):
            return value.bytes_value
        elif value.HasField("uint64_value"):
            return value.uint64_value
        elif value.HasField("null_value"):
            return None
        elif value.HasField("vector_data"):
            vector_data = value.vector_data
            if vector_data.values:
                return list(vector_data.values)
            elif vector_data.double_values:
                return list(vector_data.double_values)
            return []
        return None

    def __enter__(self):
        """
        Enter context manager
        """
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """
        Exit context manager
        """
        self.disconnect()


def parse_jdbc_url(url: str) -> Dict[str, Any]:
    """
    Parse JDBC URL

    Args:
        url: JDBC URL in format jdbc://host:port/database

    Returns:
        Dict[str, Any]: Parsed URL components

    Raises:
        ValueError: If URL is invalid
    """
    import re

    pattern = r'^jdbc://([^:]+):(\d+)/([^/]+)$'
    match = re.match(pattern, url)

    if not match:
        raise ValueError(f"Invalid JDBC URL format: {url}. Expected format: jdbc://host:port/database")

    host = match.group(1)
    port = int(match.group(2))
    database = match.group(3)

    return {
        'host': host,
        'port': port,
        'database': database
    }
