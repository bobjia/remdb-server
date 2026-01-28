"""NumPy integration for RemDB"""

import numpy as np
from typing import Optional, Any

class NumPyIntegration:
    """NumPy integration for RemDB"""

    @staticmethod
    def to_numpy_array(data: list, dtype: Optional[np.dtype] = None) -> np.ndarray:
        """
        Convert list data to NumPy array

        Args:
            data: List of data
            dtype: NumPy data type

        Returns:
            NumPy array
        """
        return np.array(data, dtype=dtype)

    @staticmethod
    def from_numpy_array(array: np.ndarray) -> list:
        """
        Convert NumPy array to list

        Args:
            array: NumPy array

        Returns:
            List of data
        """
        return array.tolist()

    @staticmethod
    def get_column_as_numpy(table, column_name: str, dtype: Optional[np.dtype] = None) -> np.ndarray:
        """
        Get a column as NumPy array

        Args:
            table: RemDbTable instance
            column_name: Column name
            dtype: NumPy data type

        Returns:
            NumPy array
        """
        # 使用表的get_column_as_numpy方法
        return table.get_column_as_numpy(column_name, dtype)

    @staticmethod
    def insert_from_numpy_array(table, column_names: list, array: np.ndarray, batch_size: int = 1000):
        """
        Insert data from NumPy array

        Args:
            table: RemDbTable instance
            column_names: List of column names
            array: NumPy array of data
            batch_size: Batch size for insertion
        """
        # 将NumPy数组转换为列表
        data_list = array.tolist()
        
        # 批量插入
        for i in range(0, len(data_list), batch_size):
            batch = data_list[i:i+batch_size]
            for row in batch:
                # 构建记录字典
                record = dict(zip(column_names, row))
                table.insert(record)
