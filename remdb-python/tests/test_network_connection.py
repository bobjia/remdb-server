#!/usr/bin/env python3
"""
Test script to verify network connection issue
"""

import sys
import os
import time

# 添加项目根目录到Python路径
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

import socket
import struct
import traceback
from remdb.proto.jdbc_pb2 import JdbcRequest, JdbcResponse, ConnectionRequest, QueryRequest

def test_network_connection():
    print("=== Testing Network Connection ===")
    
    # 服务器地址和端口
    host = "localhost"
    port = 6666
    
    # 创建socket连接
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(5)
    
    try:
        print(f"Connecting to {host}:{port}...")
        sock.connect((host, port))
        print("✓ Connected successfully!")
        
        # 创建连接请求
        print("\nCreating connection request...")
        conn_request = ConnectionRequest(
            username="root",
            password="",
            database="default",
            fetch_size=100,
            auto_commit=True
        )
        
        # 创建JDBC请求
        jdbc_request = JdbcRequest(
            request_id=1,
            connection=conn_request
        )
        
        # 序列化请求
        print("Serializing request...")
        serialized = jdbc_request.SerializeToString()
        print(f"✓ Serialized request length: {len(serialized)}")
        
        # 发送长度前缀
        length = len(serialized)
        length_prefix = struct.pack('>I', length)
        sock.sendall(length_prefix)
        
        # 发送序列化数据
        sock.sendall(serialized)
        print("✓ Request sent successfully!")
        
        # 接收响应
        print("\nReceiving connection response...")
        length_prefix = sock.recv(4)
        if len(length_prefix) == 4:
            length = struct.unpack('>I', length_prefix)[0]
            response_data = sock.recv(length)
            if len(response_data) == length:
                response = JdbcResponse()
                response.ParseFromString(response_data)
                print(f"✓ Connection response: status={response.status}, error={response.error_message}")
                
                # 测试SQL查询
                print("\n=== Testing SQL Query ===")
                
                # 创建查询请求
                query_request = QueryRequest(
                    sql="SELECT * FROM products LIMIT 5",
                    fetch_size=100,
                    use_cursor=False
                )
                
                # 创建JDBC请求
                jdbc_request = JdbcRequest(
                    request_id=2,
                    query=query_request
                )
                
                # 序列化请求
                serialized = jdbc_request.SerializeToString()
                print(f"✓ Serialized query request length: {len(serialized)}")
                
                # 发送请求
                length_prefix = struct.pack('>I', len(serialized))
                sock.sendall(length_prefix)
                sock.sendall(serialized)
                print("✓ Query request sent successfully!")
                
                # 接收响应
                print("\nReceiving query response...")
                length_prefix = sock.recv(4)
                if len(length_prefix) == 4:
                    length = struct.unpack('>I', length_prefix)[0]
                    response_data = sock.recv(length)
                    if len(response_data) == length:
                        response = JdbcResponse()
                        response.ParseFromString(response_data)
                        print(f"✓ Query response: status={response.status}, error={response.error_message}")
                        
                        if response.status == 0 and response.HasField("result_set"):
                            result_set = response.result_set
                            print(f"✓ Found {result_set.row_count} rows")
                            print(f"✓ Columns: {[col.name for col in result_set.columns]}")
                            for i, row in enumerate(result_set.rows):
                                values = []
                                for value in row.values:
                                    if value.HasField("string_value"):
                                        values.append(value.string_value)
                                    elif value.HasField("int64_value"):
                                        values.append(str(value.int64_value))
                                    elif value.HasField("double_value"):
                                        values.append(str(value.double_value))
                                    elif value.HasField("boolean_value"):
                                        values.append(str(value.boolean_value))
                                    else:
                                        values.append("NULL")
                                print(f"  Row {i}: {values}")
                    else:
                        print(f"✗ Incomplete query response: {len(response_data)} of {length} bytes")
                else:
                    print(f"✗ Failed to read query response length: {len(length_prefix)} bytes")
            else:
                print(f"✗ Incomplete connection response: {len(response_data)} of {length} bytes")
        else:
            print(f"✗ Failed to read connection response length: {len(length_prefix)} bytes")
        
        # 关闭连接
        sock.close()
        print("\n✓ Connection closed")
        
    except socket.error as e:
        print(f"✗ Socket error: {e}")
        traceback.print_exc()
    except Exception as e:
        print(f"✗ Unexpected error: {e}")
        traceback.print_exc()
    finally:
        try:
            sock.close()
        except:
            pass
        print("\n=== Test Complete ===")

if __name__ == "__main__":
    test_network_connection()