#!/usr/bin/env python3
"""
Test script to check if RemDB server is running on port 6666
"""

import socket

def test_server_connection():
    print("Testing connection to RemDB server on port 6666...")
    
    # Create a socket
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(5)
    
    try:
        # Try to connect
        s.connect(('localhost', 6666))
        print("✓ Server is running on port 6666")
        s.close()
        return True
    except socket.error as e:
        print(f"✗ Server is not running on port 6666: {e}")
        s.close()
        return False

if __name__ == "__main__":
    test_server_connection()
