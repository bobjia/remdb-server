#!/usr/bin/env python3
"""
Test script to check JDBC connection to RemDB server
"""

import sys
import os

# 添加项目根目录到Python路径
sys.path.insert(0, os.path.abspath(os.path.dirname(os.path.dirname(__file__))))

import socket
import struct
from remdb.proto.jdbc_pb2 import JdbcRequest, JdbcResponse, ConnectionRequest

def test_jdbc_connection():
    print("Testing JDBC connection to port 6666...")
    
    # Create a socket
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(5)
    
    try:
        # Try to connect
        s.connect(('localhost', 6666))
        print("✓ Connected to port 6666")
        
        # Create connection request
        connection_request = ConnectionRequest(
            username="root",
            password="",
            database="default",
            fetch_size=100,
            auto_commit=True
        )
        
        # Create JDBC request
        jdbc_request = JdbcRequest(
            request_id=1,
            connection=connection_request
        )
        
        # Serialize request
        serialized = jdbc_request.SerializeToString()
        print(f"✓ Serialized request length: {len(serialized)}")
        
        # Calculate length
        length = len(serialized)
        
        # Pack length as 4-byte big-endian
        length_prefix = struct.pack('>I', length)
        
        # Send length prefix and serialized request
        s.sendall(length_prefix)
        s.sendall(serialized)
        print("✓ Sent JDBC connection request")
        
        # Try to receive a response
        # First read length prefix (4 bytes)
        length_prefix = s.recv(4)
        if len(length_prefix) != 4:
            print(f"✗ Failed to read response length: {len(length_prefix)} bytes received")
            s.close()
            return False
        
        # Unpack length
        length = struct.unpack('>I', length_prefix)[0]
        print(f"✓ Received response length: {length}")
        
        # Read serialized response
        serialized = b''
        while len(serialized) < length:
            chunk = s.recv(length - len(serialized))
            if not chunk:
                print("✗ Connection closed while reading response")
                s.close()
                return False
            serialized += chunk
        
        print(f"✓ Received serialized response: {len(serialized)} bytes")
        
        # Parse response
        response = JdbcResponse()
        response.ParseFromString(serialized)
        print(f"✓ Parsed response successfully")
        print(f"  Response status: {response.status}")
        print(f"  Error message: {response.error_message}")
        
        if response.status == 0:
            print("✓ Connection successful!")
            if response.HasField("connection"):
                print(f"  Connection ID: {response.connection.connection_id}")
                print(f"  Server version: {response.connection.server_version}")
                print(f"  Protocol version: {response.connection.protocol_version}")
        else:
            print(f"✗ Connection failed: {response.error_message}")
        
        s.close()
        return True
    except socket.error as e:
        print(f"✗ Socket error: {e}")
        s.close()
        return False
    except Exception as e:
        import traceback
        traceback.print_exc()
        s.close()
        return False

if __name__ == "__main__":
    test_jdbc_connection()