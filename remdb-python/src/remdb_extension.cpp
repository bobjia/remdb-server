#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <pybind11/numpy.h>
#include "remdb.h"
#include <string>
#include <vector>
#include <memory>

// 声明向量搜索相关的C API函数
enum RemDbError remdb_vector_search(RemDbHandle handle, const char* table_name, const char* field_name, const float* query_vector, uint16_t vector_dim, uint32_t k, uint32_t** results, float** distances, uint32_t* result_count);
enum RemDbError remdb_free_vector_search_results(uint32_t* results, float* distances, uint32_t count);
enum RemDbDistanceType {
    REMDB_DISTANCE_L2 = 0,
    REMDB_DISTANCE_INNER_PRODUCT = 1,
    REMDB_DISTANCE_COSINE = 2
};

namespace py = pybind11;

// 数据库连接类
class RemDbConnection {
public:
    RemDbConnection() : handle(nullptr) {
    }

    ~RemDbConnection() {
        // 清理资源
    }

    bool init(const std::string& config_path) {
        // 初始化数据库
        RemDbConfig config;
        // TODO: 从配置文件加载配置
        return remdb_init_global(&config, &handle) == REMDB_SUCCESS;
    }

    bool get_global() {
        return remdb_get_global(&handle) == REMDB_SUCCESS;
    }

    RemDbHandle get_handle() const {
        return handle;
    }

private:
    RemDbHandle handle;
};

// 表操作类
class RemDbTable {
public:
    RemDbTable(RemDbHandle db_handle, size_t table_id, const std::string& table_name) 
        : db_handle(db_handle), table_id(table_id), table_name(table_name) {
    }

    bool insert(const std::vector<uint8_t>& record) {
        return remdb_table_insert(db_handle, table_id, record.data()) == REMDB_SUCCESS;
    }

    bool get(const RemDbValue& key, std::vector<uint8_t>& record) {
        // 假设record已经分配了足够的空间
        return remdb_table_get(db_handle, table_id, &key, record.data()) == REMDB_SUCCESS;
    }

    py::array_t<uint8_t> get_zero_copy(const RemDbValue& key) {
        // TODO: 实现零拷贝获取
        // 这里简化处理，返回空数组
        return py::array_t<uint8_t>(0);
    }

    bool update(const RemDbValue& key, const std::vector<uint8_t>& record) {
        return remdb_table_update(db_handle, table_id, &key, record.data()) == REMDB_SUCCESS;
    }

    bool delete_record(const RemDbValue& key) {
        return remdb_table_delete(db_handle, table_id, &key) == REMDB_SUCCESS;
    }

    size_t get_record_count() {
        size_t count = 0;
        remdb_table_get_record_count(db_handle, table_id, &count);
        return count;
    }

    std::vector<std::pair<uint32_t, float>> vector_search(const std::string& field_name, const std::vector<float>& query_vector, uint32_t k) {
        std::vector<std::pair<uint32_t, float>> results;
        
        // 执行向量搜索
        uint32_t* result_ids = nullptr;
        float* distances = nullptr;
        uint32_t result_count = 0;
        
        enum RemDbError error = remdb_vector_search(
            db_handle,
            this->table_name.c_str(),
            field_name.c_str(),
            query_vector.data(),
            static_cast<uint16_t>(query_vector.size()),
            k,
            &result_ids,
            &distances,
            &result_count
        );
        
        if (error == REMDB_SUCCESS && result_count > 0) {
            // 3. 处理搜索结果
            for (uint32_t i = 0; i < result_count; ++i) {
                results.emplace_back(result_ids[i], distances[i]);
            }
            
            // 4. 释放结果内存
            remdb_free_vector_search_results(result_ids, distances, result_count);
        }
        
        return results;
    }

private:
    RemDbHandle db_handle;
    size_t table_id;
    std::string table_name; // 添加表名字段
};

// 事务管理类
class RemDbTransaction {
public:
    RemDbTransaction(RemDbHandle db_handle) : db_handle(db_handle), active(false) {
    }

    bool begin() {
        if (!active) {
            active = remdb_begin_transaction(db_handle, REMDB_TX_READ_WRITE, REMDB_ISO_READ_COMMITTED) == REMDB_SUCCESS;
        }
        return active;
    }

    bool commit() {
        if (active) {
            bool success = remdb_commit_transaction(db_handle) == REMDB_SUCCESS;
            active = false;
            return success;
        }
        return false;
    }

    bool rollback() {
        if (active) {
            bool success = remdb_rollback_transaction(db_handle) == REMDB_SUCCESS;
            active = false;
            return success;
        }
        return false;
    }

    bool is_active() const {
        return active;
    }

private:
    RemDbHandle db_handle;
    bool active;
};

// 结果集类
class RemDbResultSet {
public:
    RemDbResultSet(RemDbResultSet* result_set) : result_set(result_set) {
    }

    ~RemDbResultSet() {
        if (result_set) {
            remdb_free_result_set(result_set);
        }
    }

    size_t get_columns_count() const {
        return result_set ? result_set->columns_count : 0;
    }

    size_t get_rows_count() const {
        return result_set ? result_set->rows_count : 0;
    }

    std::vector<std::string> get_columns() const {
        std::vector<std::string> columns;
        if (result_set) {
            for (size_t i = 0; i < result_set->columns_count; ++i) {
                const char* column_name = reinterpret_cast<const char*>(result_set->columns[i]);
                columns.push_back(column_name);
            }
        }
        return columns;
    }

    // 获取行数据
    std::vector<std::string> get_row(size_t row_index) const {
        std::vector<std::string> row;
        if (result_set && row_index < result_set->rows_count) {
            const RemDbResultRow* result_row = &result_set->rows[row_index];
            for (size_t i = 0; i < result_row->values_count; ++i) {
                const RemDbTypedValue* value = &result_row->values[i];
                // 转换值为字符串
                std::string value_str;
                switch (value->data_type) {
                    case REMDB_TYPE_UINT8:
                        value_str = std::to_string(value->value.u8);
                        break;
                    case REMDB_TYPE_UINT16:
                        value_str = std::to_string(value->value.u16);
                        break;
                    case REMDB_TYPE_UINT32:
                        value_str = std::to_string(value->value.u32);
                        break;
                    case REMDB_TYPE_UINT64:
                        value_str = std::to_string(value->value.u64);
                        break;
                    case REMDB_TYPE_FLOAT32:
                        value_str = std::to_string(value->value.float32);
                        break;
                    case REMDB_TYPE_FLOAT64:
                        value_str = std::to_string(value->value.float64);
                        break;
                    case REMDB_TYPE_BOOL:
                        value_str = value->value.bool ? "true" : "false";
                        break;
                    case REMDB_TYPE_TIMESTAMP:
                        value_str = std::to_string(value->value.timestamp);
                        break;
                    case REMDB_TYPE_STRING:
                        value_str = reinterpret_cast<const char*>(value->value.string);
                        break;
                    default:
                        value_str = "<unknown>";
                        break;
                }
                row.push_back(value_str);
            }
        }
        return row;
    }

private:
    RemDbResultSet* result_set;
};

// 数据库操作类
class RemDb {
public:
    RemDb() : handle(nullptr) {
    }

    ~RemDb() {
        // 清理资源
    }

    bool connect(const std::string& path) {
        // 对于嵌入式模式，path可以是文件路径或内存数据库标识符
        // 这里简化处理，直接获取全局实例
        return remdb_get_global(&handle) == REMDB_SUCCESS;
    }

    std::shared_ptr<RemDbTable> get_table(const std::string& table_name) {
        if (!handle) return nullptr;

        size_t table_id;
        if (remdb_table_get_by_name(handle, table_name.c_str(), &table_id) == REMDB_SUCCESS) {
            return std::make_shared<RemDbTable>(handle, table_id, table_name);
        }
        return nullptr;
    }

    std::shared_ptr<RemDbTransaction> begin_transaction() {
        auto tx = std::make_shared<RemDbTransaction>(handle);
        tx->begin();
        return tx;
    }

    std::shared_ptr<RemDbResultSet> execute_query(const std::string& sql) {
        if (!handle) return nullptr;

        RemDbResultSet* result_set = nullptr;
        if (remdb_sql_query(handle, sql.c_str(), &result_set) == REMDB_SUCCESS) {
            return std::make_shared<RemDbResultSet>(result_set);
        }
        return nullptr;
    }

    bool save_snapshot(const std::string& path) {
        if (!handle) return false;
        return remdb_save_snapshot(handle, path.c_str()) == REMDB_SUCCESS;
    }

    bool restore_snapshot(const std::string& path) {
        if (!handle) return false;
        return remdb_restore_snapshot(handle, path.c_str()) == REMDB_SUCCESS;
    }

    RemDbHandle get_handle() const {
        return handle;
    }

private:
    RemDbHandle handle;
};

// 异常类
class RemDbError : public std::exception {
public:
    RemDbError(const std::string& message) : message(message) {
    }

    const char* what() const noexcept override {
        return message.c_str();
    }

private:
    std::string message;
};

// 绑定到Python
PYBIND11_MODULE(_remdb, m) {
    m.doc() = "RemDB Python bindings";

    // 绑定异常类
    py::register_exception<RemDbError>(m, "RemDbError");

    // 绑定数据库类
    py::class_<RemDb>(m, "RemDb")
        .def(py::init<>())
        .def("connect", &RemDb::connect, "Connect to database")
        .def("get_table", &RemDb::get_table, "Get table by name")
        .def("begin_transaction", &RemDb::begin_transaction, "Begin a transaction")
        .def("execute_query", &RemDb::execute_query, "Execute SQL query")
        .def("save_snapshot", &RemDb::save_snapshot, "Save database snapshot")
        .def("restore_snapshot", &RemDb::restore_snapshot, "Restore database from snapshot");

    // 绑定表类
    py::class_<RemDbTable>(m, "RemDbTable")
        .def("insert", &RemDbTable::insert, "Insert record")
        .def("get", &RemDbTable::get, "Get record by key")
        .def("get_zero_copy", &RemDbTable::get_zero_copy, "Get record with zero copy")
        .def("update", &RemDbTable::update, "Update record")
        .def("delete_record", &RemDbTable::delete_record, "Delete record")
        .def("get_record_count", &RemDbTable::get_record_count, "Get record count")
        .def("get_column_as_numpy", [](RemDbTable& self, const std::string& column_name) {
            // TODO: 实现NumPy集成
            return py::array_t<double>(0);
        }, "Get column as NumPy array")
        .def("vector_search", &RemDbTable::vector_search, "Perform vector similarity search",
            py::arg("field_name"), py::arg("query_vector"), py::arg("k"));

    // 绑定距离类型枚举
    py::enum_<RemDbDistanceType>(m, "RemDbDistanceType")
        .value("L2", REMDB_DISTANCE_L2)
        .value("INNER_PRODUCT", REMDB_DISTANCE_INNER_PRODUCT)
        .value("COSINE", REMDB_DISTANCE_COSINE)
        .export_values();


    // 绑定事务类
    py::class_<RemDbTransaction>(m, "RemDbTransaction")
        .def("begin", &RemDbTransaction::begin, "Begin transaction")
        .def("commit", &RemDbTransaction::commit, "Commit transaction")
        .def("rollback", &RemDbTransaction::rollback, "Rollback transaction")
        .def("is_active", &RemDbTransaction::is_active, "Check if transaction is active");

    // 绑定结果集类
    py::class_<RemDbResultSet>(m, "RemDbResultSet")
        .def("get_columns_count", &RemDbResultSet::get_columns_count, "Get number of columns")
        .def("get_rows_count", &RemDbResultSet::get_rows_count, "Get number of rows")
        .def("get_columns", &RemDbResultSet::get_columns, "Get column names")
        .def("get_row", &RemDbResultSet::get_row, "Get row data by index");

    // 绑定数据类型
    py::enum_<RemDbDataType>(m, "RemDbDataType")
        .value("UINT8", REMDB_TYPE_UINT8)
        .value("UINT16", REMDB_TYPE_UINT16)
        .value("UINT32", REMDB_TYPE_UINT32)
        .value("UINT64", REMDB_TYPE_UINT64)
        .value("FLOAT32", REMDB_TYPE_FLOAT32)
        .value("FLOAT64", REMDB_TYPE_FLOAT64)
        .value("BOOL", REMDB_TYPE_BOOL)
        .value("TIMESTAMP", REMDB_TYPE_TIMESTAMP)
        .value("STRING", REMDB_TYPE_STRING)
        .export_values();

    // 绑定值类型
    py::class_<RemDbValue>(m, "RemDbValue")
        .def(py::init<>())
        .def_property("u8", [](const RemDbValue& v) { return v.u8; }, [](RemDbValue& v, uint8_t val) { v.u8 = val; })
        .def_property("u16", [](const RemDbValue& v) { return v.u16; }, [](RemDbValue& v, uint16_t val) { v.u16 = val; })
        .def_property("u32", [](const RemDbValue& v) { return v.u32; }, [](RemDbValue& v, uint32_t val) { v.u32 = val; })
        .def_property("u64", [](const RemDbValue& v) { return v.u64; }, [](RemDbValue& v, uint64_t val) { v.u64 = val; })
        .def_property("float32", [](const RemDbValue& v) { return v.float32; }, [](RemDbValue& v, float val) { v.float32 = val; })
        .def_property("float64", [](const RemDbValue& v) { return v.float64; }, [](RemDbValue& v, double val) { v.float64 = val; })
        .def_property("bool", [](const RemDbValue& v) { return v.bool; }, [](RemDbValue& v, bool val) { v.bool = val; })
        .def_property("timestamp", [](const RemDbValue& v) { return v.timestamp; }, [](RemDbValue& v, uint64_t val) { v.timestamp = val; })
        .def_property("string", 
            [](const RemDbValue& v) { return std::string(reinterpret_cast<const char*>(v.string)); }, 
            [](RemDbValue& v, const std::string& val) {
                std::strncpy(reinterpret_cast<char*>(v.string), val.c_str(), REMDB_MAX_STRING_LEN - 1);
                v.string[REMDB_MAX_STRING_LEN - 1] = '\0';
            }
        );

    // 绑定错误码
    py::enum_<RemDbError>(m, "RemDbErrorCode")
        .value("SUCCESS", REMDB_SUCCESS)
        .value("OUT_OF_MEMORY", REMDB_ERROR_OUT_OF_MEMORY)
        .value("RECORD_NOT_FOUND", REMDB_ERROR_RECORD_NOT_FOUND)
        .value("DUPLICATE_KEY", REMDB_ERROR_DUPLICATE_KEY)
        .value("FIELD_NOT_FOUND", REMDB_ERROR_FIELD_NOT_FOUND)
        .value("TYPE_MISMATCH", REMDB_ERROR_TYPE_MISMATCH)
        .value("TRANSACTION_ERROR", REMDB_ERROR_TRANSACTION_ERROR)
        .value("CONFIG_ERROR", REMDB_ERROR_CONFIG_ERROR)
        .value("UNSUPPORTED_OPERATION", REMDB_ERROR_UNSUPPORTED_OPERATION)
        .value("FILE_IO_ERROR", REMDB_ERROR_FILE_IO_ERROR)
        .value("SNAPSHOT_FORMAT_ERROR", REMDB_ERROR_SNAPSHOT_FORMAT_ERROR)
        .value("CRC32_ERROR", REMDB_ERROR_CRC32_ERROR)
        .value("LOG_FORMAT_ERROR", REMDB_ERROR_LOG_FORMAT_ERROR)
        .value("LOG_RECORD_NOT_FOUND", REMDB_ERROR_LOG_RECORD_NOT_FOUND)
        .value("LOG_CHECKSUM_ERROR", REMDB_ERROR_LOG_CHECKSUM_ERROR)
        .value("LOCK_CONFLICT", REMDB_ERROR_LOCK_CONFLICT)
        .value("LOCK_TIMEOUT", REMDB_ERROR_LOCK_TIMEOUT)
        .value("TABLE_NOT_FOUND", REMDB_ERROR_TABLE_NOT_FOUND)
        .value("INVALID_RECORD_SIZE", REMDB_ERROR_INVALID_RECORD_SIZE)
        .export_values();
}
