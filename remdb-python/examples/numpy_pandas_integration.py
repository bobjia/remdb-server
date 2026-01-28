"""NumPy and Pandas integration example for RemDB Python bindings"""

import remdb
import tempfile
import os

try:
    import numpy as np
    import pandas as pd
    from remdb.extras.numpy import NumPyIntegration
    from remdb.extras.pandas import PandasIntegration
    
    # 创建临时文件作为数据库路径
    temp_file = tempfile.NamedTemporaryFile(delete=False)
    db_path = temp_file.name
    temp_file.close()
    
    try:
        # 连接到数据库
        print("Connecting to database...")
        with remdb.connect(db_path) as db:
            print("Connected successfully!")
            
            # 测试NumPy集成
            print("\nTesting NumPy integration...")
            
            # 创建NumPy数组
            data = np.array([[1, 23.5], [2, 24.0], [3, 22.5], [4, 25.0], [5, 23.0]])
            print(f"Created NumPy array with shape: {data.shape}")
            
            # 转换为列表
            data_list = NumPyIntegration.from_numpy_array(data)
            print(f"Converted to list: {data_list}")
            
            # 转换回NumPy数组
            data_array = NumPyIntegration.to_numpy_array(data_list)
            print(f"Converted back to NumPy array with shape: {data_array.shape}")
            
            # 测试Pandas集成
            print("\nTesting Pandas integration...")
            
            # 创建Pandas DataFrame
            df = pd.DataFrame({
                "id": [1, 2, 3, 4, 5],
                "value": [23.5, 24.0, 22.5, 25.0, 23.0],
                "timestamp": [1620000000, 1620000001, 1620000002, 1620000003, 1620000004]
            })
            print("Created Pandas DataFrame:")
            print(df)
            
            # 尝试获取表并插入数据
            try:
                print("\nGetting table 'sensor_data'...")
                table = db.get_table("sensor_data")
                
                # 从DataFrame插入数据
                print("Inserting data from DataFrame...")
                table.insert_from_dataframe(df)
                print(f"Insert completed. Table now has {table.get_record_count()} records")
                
                # 尝试获取列作为NumPy数组
                print("\nGetting 'value' column as NumPy array...")
                values = table.get_column_as_numpy("value")
                print(f"Column data type: {type(values)}")
                if isinstance(values, np.ndarray):
                    print(f"Array shape: {values.shape}")
                    print(f"First 5 values: {values[:5]}")
                    
                # 尝试转换表为DataFrame
                print("\nConverting table to DataFrame...")
                df_result = table.to_dataframe()
                print(f"DataFrame created with {len(df_result)} rows")
                print(df_result.head())
                
            except remdb.NotFoundError as e:
                print(f"Table not found: {e}")
                print("Skipping DataFrame insertion tests")
                
            # 测试SQL查询转换为DataFrame
            print("\nTesting SQL query to DataFrame...")
            try:
                sql = "SELECT * FROM sensor_data LIMIT 5"
                df_sql = PandasIntegration.read_sql(sql, db)
                print(f"Query returned DataFrame with {len(df_sql)} rows")
                print(df_sql)
            except Exception as e:
                print(f"SQL query failed: {e}")
                
    finally:
        # 清理临时文件
        if os.path.exists(db_path):
            os.unlink(db_path)
            print(f"\nCleaned up temporary file: {db_path}")
            
except ImportError as e:
    print(f"Required module not installed: {e}")
    print("Please install NumPy and Pandas to run this example:")
    print("pip install numpy pandas")

print("\nExample completed!")
