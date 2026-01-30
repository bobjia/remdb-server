#!/usr/bin/env python3
"""Clean script for remdb-python project"""

import os
import shutil
import glob
import sys


def remove_directory(path):
    """Remove a directory if it exists"""
    if os.path.exists(path):
        try:
            shutil.rmtree(path)
            print(f"✓ Removed directory: {path}")
        except Exception as e:
            print(f"✗ Failed to remove directory {path}: {e}")


def remove_file(path):
    """Remove a file if it exists"""
    if os.path.exists(path):
        try:
            os.remove(path)
            print(f"✓ Removed file: {path}")
        except Exception as e:
            print(f"✗ Failed to remove file {path}: {e}")


def remove_files(pattern):
    """Remove files matching a pattern"""
    for file_path in glob.glob(pattern, recursive=True):
        remove_file(file_path)


def main():
    """Main clean function"""
    print("=== remdb-python Clean Script ===")
    print()
    
    # 1. Remove build artifacts
    print("1. Removing build artifacts...")
    remove_directory("build")
    remove_directory("remdb_python.egg-info")
    
    # 2. Remove compiled Python files
    print("\n2. Removing compiled Python files...")
    remove_files("**/__pycache__")
    remove_files("**/*.pyc")
    remove_files("**/*.pyo")
    remove_files("**/*.pyd")
    
    # 3. Remove extension files
    print("\n3. Removing extension files...")
    remove_file("_remdb.cp313-win_amd64.pyd")
    
    # 4. Remove temporary files
    print("\n4. Removing temporary files...")
    remove_files("*.tmp")
    remove_files("*.temp")
    remove_files("*.log")
    
    # 5. Remove test files created during installation
    print("\n5. Removing test files...")
    # remove_file("test_install.py")
    # remove_file("verify_installation.py")
    # remove_file("clean.py")
    
    print("\n=== Cleanup Complete ===")
    print("The project has been cleaned of build artifacts and temporary files.")
    print()
    print("To rebuild the project:")
    print("  1. python setup.py build_ext --inplace")
    print("  2. python -m pip install -e . --no-build-isolation")


if __name__ == "__main__":
    main()
