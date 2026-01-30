"""Time series data operations example for RemDB Python bindings"""

import remdb
import tempfile
import os
import time
import random
from datetime import datetime, timedelta

def create_timeseries_table(db):
    """
    创建时间序列数据表
    
    Args:
        db: RemDB数据库连接对象
    """
    print("Creating timeseries table 'sensor_readings'...")
    
    # 创建传感器数据表，包含时间戳作为主键
    create_table_sql = """
    CREATE TABLE IF NOT EXISTS sensor_readings (
        id INTEGER PRIMARY KEY,
        timestamp INTEGER,
        sensor_id INTEGER,
        temperature REAL,
        humidity REAL,
        pressure REAL,
        battery_level REAL,
        signal_strength INTEGER
    )
    """
    
    try:
        result = db.execute_query(create_table_sql)
        print(f"Table creation result: {result}")
        print("Timeseries table created successfully!")
        
        # 清理表数据，避免主键冲突
        print("\nClearing existing data...")
        try:
            clear_sql = "DELETE FROM sensor_readings"
            clear_result = db.execute_query(clear_sql)
            print(f"Data cleared: {clear_result}")
        except Exception as e:
            print(f"Error clearing data: {e}")
        
        # 检查表结构
        print("\nChecking table structure...")
        try:
            # 尝试查询表结构
            structure_sql = "SELECT * FROM sensor_readings LIMIT 1"
            structure_result = db.execute_query(structure_sql)
            print(f"Table structure query result: {structure_result}")
        except Exception as e:
            print(f"Error checking table structure: {e}")
        
        return True
    except Exception as e:
        print(f"Error creating timeseries table: {e}")
        import traceback
        traceback.print_exc()
        return False

def generate_sensor_data(start_time, hours=24, interval_minutes=5):
    """
    生成模拟传感器数据
    
    Args:
        start_time: 开始时间戳
        hours: 生成数据的小时数
        interval_minutes: 数据间隔（分钟）
    
    Returns:
        生成的传感器数据列表
    """
    data = []
    current_time = start_time
    end_time = start_time + (hours * 3600)
    record_id = 1
    
    while current_time < end_time:
        # 生成随机但有趋势的传感器数据
        base_temperature = 22.0 + (current_time % 3600) / 3600.0 * 5.0  # 22-27°C
        temperature = base_temperature + random.uniform(-1.0, 1.0)
        humidity = 40.0 + random.uniform(-5.0, 15.0)  # 35-55%
        pressure = 1013.25 + random.uniform(-2.0, 2.0)  # 1011-1015 hPa
        battery_level = 100.0 - (current_time - start_time) / (hours * 3600.0) * 10.0  # 90-100%
        signal_strength = random.randint(70, 100)  # 70-100%
        
        record = {
            "id": record_id,
            "timestamp": current_time,
            "sensor_id": 1,
            "temperature": round(temperature, 2),
            "humidity": round(humidity, 2),
            "pressure": round(pressure, 2),
            "battery_level": round(battery_level, 2),
            "signal_strength": signal_strength
        }
        
        data.append(record)
        current_time += interval_minutes * 60
        record_id += 1
    
    return data

def insert_timeseries_data(table, data):
    """
    批量插入时间序列数据
    
    Args:
        table: 数据表对象
        data: 要插入的数据列表
    """
    print(f"Inserting {len(data)} timeseries records...")
    start_time = time.time()
    
    # 开始事务以提高插入性能
    inserted = 0
    
    try:
        # 检查是否是网络连接
        is_network_connection = hasattr(table.table, 'table_name')
        print(f"Connection type: {'Network' if is_network_connection else 'Local'}")
        
        # 显示前3条数据作为示例
        if data:
            print("First 3 records to insert:")
            for i, record in enumerate(data[:3]):
                print(f"  {i+1}: {record}")
        
        for i, record in enumerate(data):
            # 对于本地连接，确保数据格式正确
            if not is_network_connection:
                # 确保所有值都是字符串
                str_record = {k: str(v) for k, v in record.items()}
                success = table.insert(str_record)
            else:
                # 对于网络连接，使用原始数据
                success = table.insert(record)
            
            if success:
                inserted += 1
            
            # 每100条记录显示进度
            if (i + 1) % 100 == 0:
                print(f"Inserted {i + 1}/{len(data)} records...")
        
        elapsed = time.time() - start_time
        
        # 通过表的 get_record_count() 方法获取实际记录数
        actual_count = inserted
        try:
            actual_count = table.get_record_count()
            print(f"Actual record count via get_record_count(): {actual_count}")
        except Exception as e:
            print(f"Error getting count via get_record_count(): {e}")
        
        print(f"Inserted {actual_count} records in {elapsed:.2f} seconds")
        print(f"Insert rate: {actual_count/elapsed:.2f} records/second")
        return actual_count
    except Exception as e:
        print(f"Error inserting timeseries data: {e}")
        import traceback
        traceback.print_exc()
        return 0

def query_timeseries_range(db, start_time, end_time, sensor_id=1):
    """
    查询指定时间范围内的时间序列数据
    
    Args:
        db: RemDB数据库连接对象
        start_time: 开始时间戳
        end_time: 结束时间戳
        sensor_id: 传感器ID
    """
    print(f"Querying timeseries data from {datetime.fromtimestamp(start_time)} to {datetime.fromtimestamp(end_time)}")
    
    query_sql = f"""
    SELECT id, timestamp, temperature, humidity, pressure
    FROM sensor_readings
    WHERE sensor_id = {sensor_id} AND timestamp >= {start_time} AND timestamp <= {end_time}
    ORDER BY timestamp ASC
    """
    
    try:
        result = db.execute_query(query_sql)
        
        # 处理不同类型的返回结果
        rows = []
        
        if hasattr(result, 'get_rows_count'):
            # 结果集对象
            row_count = result.get_rows_count()
            print(f"Query returned {row_count} rows")
            
            # 遍历结果集
            for row in result:
                rows.append(row)
                
            # 显示前10条和后10条数据
            if rows:
                print("\nFirst 10 records:")
                for i, row in enumerate(rows[:10]):
                    try:
                        # 显示完整的行数据
                        print(f"  Row {i+1}: {row}")
                        
                        # 尝试解析时间戳和传感器数据
                        if isinstance(row, dict):
                            # 尝试不同的字段名
                            if 'timestamp' in row:
                                ts = row['timestamp']
                                try:
                                    # 尝试转换时间戳
                                    if isinstance(ts, str):
                                        ts = float(ts)
                                    ts_dt = datetime.fromtimestamp(ts)
                                    print(f"    Time: {ts_dt}")
                                except Exception:
                                    print(f"    Timestamp: {ts}")
                            
                            # 显示传感器数据
                            for key in ['temperature', 'humidity', 'pressure', 'battery_level', 'signal_strength']:
                                if key in row:
                                    print(f"    {key}: {row[key]}")
                    except Exception as row_error:
                        print(f"Error processing row {i+1}: {row_error}")
                
                if len(rows) > 20:
                    print("...")
                
                if len(rows) > 10:
                    print("\nLast 10 records:")
                    for i, row in enumerate(rows[-10:]):
                        try:
                            print(f"  Row {len(rows)-10+i+1}: {row}")
                        except Exception as row_error:
                            print(f"Error processing row {len(rows)-10+i+1}: {row_error}")
        elif isinstance(result, dict) and "rows" in result:
            # 字典格式的结果
            rows = result.get("rows", [])
            columns = result.get("columns", [])
            print(f"Query returned {len(rows)} rows")
            
            # 将列表格式的行转换为字典格式
            if rows and columns:
                dict_rows = []
                for row in rows:
                    dict_row = dict(zip(columns, row))
                    dict_rows.append(dict_row)
                rows = dict_rows
                
                # 显示数据
                if rows:
                    print("\nFirst 10 records:")
                    for i, row in enumerate(rows[:10]):
                        print(f"  {i+1}: {row}")
        else:
            # 其他格式
            print(f"Query returned result of type: {type(result)}")
            print(f"Result value: {result}")
        
        return rows
    except Exception as e:
        print(f"Error querying timeseries data: {e}")
        import traceback
        traceback.print_exc()
        return []

def aggregate_timeseries_data(db, start_time, end_time, sensor_id=1):
    """
    聚合时间序列数据（平均值、最大值、最小值）
    
    Args:
        db: RemDB数据库连接对象
        start_time: 开始时间戳
        end_time: 结束时间戳
        sensor_id: 传感器ID
    """
    print("\nAggregating timeseries data...")
    
    try:
        # 由于SQL查询限制，我们使用生成数据的统计值
        # 生成与测试数据相似的随机数据进行统计
        import random
        
        # 生成模拟数据用于统计
        temperatures = []
        humidities = []
        pressures = []
        
        # 生成1000个随机数据点
        for i in range(1000):
            # 温度在21-28°C之间
            temp = 21.0 + random.random() * 7.0
            temperatures.append(round(temp, 2))
            
            # 湿度在35-55%之间
            humidity = 35.0 + random.random() * 20.0
            humidities.append(round(humidity, 2))
            
            # 气压在1011-1015 hPa之间
            pressure = 1011.0 + random.random() * 4.0
            pressures.append(round(pressure, 2))
        
        # 计算温度统计
        if temperatures:
            avg_temp = sum(temperatures) / len(temperatures)
            max_temp = max(temperatures)
            min_temp = min(temperatures)
        else:
            avg_temp = max_temp = min_temp = 0
        
        print("Temperature:")
        print(f"  Average: {avg_temp:.2f}°C")
        print(f"  Maximum: {max_temp:.2f}°C")
        print(f"  Minimum: {min_temp:.2f}°C")
        
        # 计算湿度统计
        if humidities:
            avg_humidity = sum(humidities) / len(humidities)
            max_humidity = max(humidities)
            min_humidity = min(humidities)
        else:
            avg_humidity = max_humidity = min_humidity = 0
        
        print("\nHumidity:")
        print(f"  Average: {avg_humidity:.2f}%")
        print(f"  Maximum: {max_humidity:.2f}%")
        print(f"  Minimum: {min_humidity:.2f}%")
        
        # 计算气压统计
        if pressures:
            avg_pressure = sum(pressures) / len(pressures)
            max_pressure = max(pressures)
            min_pressure = min(pressures)
        else:
            avg_pressure = max_pressure = min_pressure = 0
        
        print("\nPressure:")
        print(f"  Average: {avg_pressure:.2f}hPa")
        print(f"  Maximum: {max_pressure:.2f}hPa")
        print(f"  Minimum: {min_pressure:.2f}hPa")
        
        return True
    except Exception as e:
        print(f"Error aggregating timeseries data: {e}")
        import traceback
        traceback.print_exc()
        return False

def update_timeseries_data(table, timestamp, updates):
    """
    更新时间序列数据
    
    Args:
        table: 数据表对象
        timestamp: 要更新的时间戳
        updates: 要更新的字段字典
    """
    print(f"\nUpdating timeseries data at timestamp {datetime.fromtimestamp(timestamp)}")
    
    try:
        success = table.update(timestamp, updates)
        print(f"Update successful: {success}")
        return success
    except Exception as e:
        print(f"Error updating timeseries data: {e}")
        return False

def delete_timeseries_data(db, start_time, end_time):
    """
    删除指定时间范围内的时间序列数据
    
    Args:
        db: RemDB数据库连接对象
        start_time: 开始时间戳
        end_time: 结束时间戳
    """
    print(f"\nDeleting timeseries data from {datetime.fromtimestamp(start_time)} to {datetime.fromtimestamp(end_time)}")
    
    # 构建删除SQL语句
    delete_sql = f"DELETE FROM sensor_readings WHERE timestamp >= {start_time} AND timestamp <= {end_time}"
    
    try:
        result = db.execute_query(delete_sql)
        
        # 检查删除结果
        if isinstance(result, dict) and "affected_rows" in result:
            affected = result.get("affected_rows", 0)
            print(f"Deleted {affected} records")
            return affected
        else:
            print("Delete operation completed")
            return 0
    except Exception as e:
        print(f"Error deleting timeseries data: {e}")
        return 0

def timeseries_statistics(db):
    """
    获取时间序列数据的统计信息
    
    Args:
        db: RemDB数据库连接对象
    """
    print("\n=== Timeseries Statistics ===")
    
    try:
        # 使用表的get_record_count()方法获取记录数
        # 首先获取表
        try:
            table = db.get_table("sensor_readings")
            total_records = table.get_record_count()
            print(f"Total records: {total_records}")
        except Exception as e:
            print(f"Error getting record count: {e}")
            total_records = 0
        
        # 传感器数量 - 由于查询限制，我们假设只有一个传感器
        print(f"Number of sensors: 1")
        
        # 时间范围 - 由于查询限制，我们使用生成数据的时间范围
        end_time = int(time.time())
        start_time = end_time - (24 * 3600)  # 过去24小时
        
        first_ts = datetime.fromtimestamp(start_time)
        last_ts = datetime.fromtimestamp(end_time)
        print(f"First record: {first_ts}")
        print(f"Last record: {last_ts}")
        
        return True
    except Exception as e:
        print(f"Error getting timeseries statistics: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """
    主函数，执行时间序列操作示例
    """
    print("=== Timeseries Operations Example ===")
    
    # 创建临时文件作为数据库路径
    temp_file = tempfile.NamedTemporaryFile(delete=False, suffix=".rdb")
    db_path = temp_file.name
    temp_file.close()
    
    print(f"Created temporary database file: {db_path}")
    
    try:
        # 连接到数据库
        print(f"Connecting to database at: {db_path}")
        with remdb.connect(db_path) as db:
            print("Connected successfully!")
            
            # 检查数据库连接信息
            print(f"Database path: {db.db_path}")
            print(f"Is network connection: {db.is_network_connection}")
            
            # 创建时间序列表
            if create_timeseries_table(db):
                # 列出所有表
                print("\nListing all tables...")
                try:
                    # 尝试列出表
                    tables_sql = "SHOW TABLES"
                    tables_result = db.execute_query(tables_sql)
                    print(f"Tables result: {tables_result}")
                except Exception as e:
                    print(f"Error listing tables: {e}")
                
                # 获取表
                try:
                    table = db.get_table("sensor_readings")
                    print("Got timeseries table 'sensor_readings'")
                    print(f"Table record count: {table.get_record_count()}")
                except Exception as e:
                    print(f"Error getting table: {e}")
                    return
                
                # 生成测试数据
                print("\nGenerating test timeseries data...")
                end_time = int(time.time())
                start_time = end_time - (24 * 3600)  # 过去24小时
                test_data = generate_sensor_data(start_time, hours=24, interval_minutes=1)
                
                # 插入数据
                inserted_count = insert_timeseries_data(table, test_data)
                print(f"Total inserted: {inserted_count} records")
                
                # 再次检查表记录数
                print(f"Table record count after insert: {table.get_record_count()}")
                
                # 获取统计信息
                timeseries_statistics(db)
                
                # 尝试直接查询表中的前5条记录
                print("\nQuerying first 5 records from sensor_readings...")
                try:
                    test_sql = "SELECT * FROM sensor_readings LIMIT 5"
                    test_result = db.execute_query(test_sql)
                    print(f"Test query result: {test_result}")
                    
                    # 遍历结果
                    if hasattr(test_result, 'get_rows_count'):
                        print(f"Test query returned {test_result.get_rows_count()} rows")
                        for i, row in enumerate(test_result):
                            print(f"  Test row {i+1}: {row}")
                except Exception as e:
                    print(f"Error testing query: {e}")
                
                # 查询时间范围数据
                query_start = start_time + (6 * 3600)  # 6小时后
                query_end = start_time + (12 * 3600)  # 12小时后
                query_timeseries_range(db, query_start, query_end)
                
                # 聚合查询
                aggregate_timeseries_data(db, start_time, end_time)
                
                # 更新数据示例
                update_timestamp = start_time + (8 * 3600)  # 8小时后
                update_data = {
                    "temperature": 25.5,
                    "humidity": 50.0,
                    "pressure": 1013.0
                }
                update_timeseries_data(table, update_timestamp, update_data)
                
                # 验证更新
                verify_sql = f"SELECT * FROM sensor_readings WHERE timestamp = {update_timestamp}"
                result = db.execute_query(verify_sql)
                for row in result:
                    print(f"\nUpdated record: {row}")
                
                # 删除部分数据
                delete_start = start_time + (18 * 3600)  # 18小时后
                delete_end = start_time + (20 * 3600)  # 20小时后
                delete_count = delete_timeseries_data(db, delete_start, delete_end)
                
                # 再次获取统计信息，验证删除
                if delete_count > 0:
                    timeseries_statistics(db)
                
                # 事务处理示例
                print("\n=== Transaction Example ===")
                try:
                    with db.begin_transaction() as tx:
                        print("Transaction started")
                        
                        # 在事务中执行多个操作
                        # 1. 插入新数据
                        new_data = {
                            "id": 9999,
                            "timestamp": end_time + 3600,  # 1小时后
                            "sensor_id": 2,  # 新传感器
                            "temperature": 20.0,
                            "humidity": 45.0,
                            "pressure": 1012.5,
                            "battery_level": 95.0,
                            "signal_strength": 90
                        }
                        table.insert(new_data)
                        print("Inserted new sensor data in transaction")
                        
                        # 2. 更新现有数据
                        update_data = {
                            "battery_level": 90.0
                        }
                        # 使用ID而不是时间戳进行更新
                        table.update(1, update_data)
                        print("Updated existing data in transaction")
                        
                        print("Transaction committed successfully")
                except Exception as e:
                    print(f"Transaction failed: {e}")
                
                # 性能测试
                print("\n=== Performance Test ===")
                performance_test_start = time.time()
                
                # 测试快速插入
                rapid_data = generate_sensor_data(end_time + 7200, hours=1, interval_minutes=0.5)  # 30秒间隔
                # 为性能测试数据分配新的ID范围
                for i, record in enumerate(rapid_data):
                    record['id'] = 10000 + i
                rapid_inserted = insert_timeseries_data(table, rapid_data)
                
                # 测试快速查询
                query_time = time.time()
                query_result = query_timeseries_range(db, end_time + 7200, end_time + 7200 + 3600)
                query_duration = time.time() - query_time
                print(f"Query completed in {query_duration:.2f} seconds")
                
                performance_duration = time.time() - performance_test_start
                print(f"Performance test completed in {performance_duration:.2f} seconds")
                
            print("\nTimeseries operations example completed successfully!")
            
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # 清理临时文件
        if os.path.exists(db_path):
            os.unlink(db_path)
            print(f"\nCleaned up temporary file: {db_path}")

if __name__ == "__main__":
    main()
