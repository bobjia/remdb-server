#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <pybind11/numpy.h>
#include <string>
#include <vector>
#include <map>
#include <memory>
#include <cstdio>
#include <iostream>
#include <mutex>
#include <atomic>

// Include RemDB C API
#include "remdb.h"

namespace py = pybind11;

// Forward declaration
class RemDb;

// Global state management
static std::mutex g_db_mutex;
static std::atomic<bool> g_db_initialized(false);
static RemDbHandle g_global_db_handle = nullptr;

class ZeroCopyData {
private:
    std::string data_;

public:
    ZeroCopyData(const std::string& data) : data_(data) {}
    
    std::string tobytes() const {
        return data_;
    }
};

class RemDbTable {
private:
    std::string table_name_;
    RemDbHandle db_handle_;
    size_t table_id_;

public:
    RemDbTable(const std::string& table_name, RemDbHandle db_handle, size_t table_id) 
        : table_name_(table_name), db_handle_(db_handle), table_id_(table_id) {}
    
    bool execute_sql(const std::string& sql) {
        ::RemDbResultSet* result_set = nullptr;
        enum RemDbError err = remdb_sql_query(db_handle_, sql.c_str(), &result_set);
        
        if (err == REMDB_SUCCESS && result_set) {
            remdb_free_result_set(result_set);
            return true;
        }
        
        if (result_set) {
            remdb_free_result_set(result_set);
        }
        
        return false;
    }
    
    bool insert(const std::map<std::string, std::string>& record) {
        if (record.empty()) {
            return false;
        }
        
        // Build SQL INSERT statement
        std::string sql = "INSERT INTO " + table_name_ + " (";
        std::string columns;
        std::string values;
        
        printf("DEBUG C++ insert(): Building INSERT statement\n");
        fflush(stdout);
        
        for (const auto& pair : record) {
            if (!columns.empty()) {
                columns += ", ";
                values += ", ";
            }
            columns += pair.first;
            
            // Check if value is JSON (starts with { or [) and doesn't already have quotes
            const std::string& value = pair.second;
            bool is_json = (value.length() >= 2 && 
                           (value[0] == '{' || value[0] == '[') &&
                           (value[value.length()-1] == '}' || value[value.length()-1] == ']'));
            
            if (is_json) {
                // JSON value: use as-is without extra quotes
                printf("DEBUG C++ insert(): field='%s', value='%s' (JSON, no quotes)\n", pair.first.c_str(), value.c_str());
                fflush(stdout);
                values += value;
            } else {
                // Regular value: wrap in quotes
                printf("DEBUG C++ insert(): field='%s', value='%s' (regular, with quotes)\n", pair.first.c_str(), value.c_str());
                fflush(stdout);
                values += "'" + value + "'";
            }
        }
        
        sql += columns + ") VALUES (" + values + ")";
        
        printf("DEBUG C++ insert(): Final SQL: %s\n", sql.c_str());
        fflush(stdout);
        
        return execute_sql(sql);
    }
    
    bool insert(const std::vector<std::string>& record) {
        if (record.empty()) {
            return false;
        }
        
        // Build SQL INSERT statement
        std::string sql = "INSERT INTO " + table_name_ + " VALUES (";
        
        for (size_t i = 0; i < record.size(); i++) {
            if (i > 0) {
                sql += ", ";
            }
            sql += "'" + record[i] + "'";
        }
        
        sql += ")";
        
        return execute_sql(sql);
    }
    
    bool get(const std::string& key, std::vector<std::string>& record) {
        // Build SQL SELECT statement
        std::string sql = "SELECT * FROM " + table_name_ + " WHERE 1=1 LIMIT 1";
        
        ::RemDbResultSet* result_set = nullptr;
        enum RemDbError err = remdb_sql_query(db_handle_, sql.c_str(), &result_set);
        
        if (err == REMDB_SUCCESS && result_set && result_set->rows_count > 0) {
            const ::RemDbResultRow* row = &result_set->rows[0];
            record.clear();
            
            for (size_t j = 0; j < row->values_count; j++) {
                const ::RemDbTypedValue* value = &row->values[j];
                std::string value_str;
                
                // Convert value based on data type
                    switch (value->data_type) {
                        case REMDB_TYPE_UINT8:
                            value_str = std::to_string(value->value.u8);
                            break;
                        case REMDB_TYPE_UINT16:
                            value_str = std::to_string(value->value.u16);
                            break;
                        case REMDB_TYPE_UINT32:
                            {
                                // Interpret as signed 32-bit integer (INTEGER type)
                                int32_t signed_val = static_cast<int32_t>(value->value.u32);
                                value_str = std::to_string(signed_val);
                            }
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
                            value_str = value->value.boolean ? "true" : "false";
                            break;
                        case REMDB_TYPE_TIMESTAMP:
                            value_str = std::to_string(value->value.timestamp);
                            break;
                        case REMDB_TYPE_STRING:
                            value_str = std::string(reinterpret_cast<const char*>(value->value.string));
                            // Remove trailing null character
                            value_str = value_str.substr(0, value_str.find('\0'));
                            break;
                        case REMDB_TYPE_JSON: {
                            // For JSON type, we need to get the actual JSON string using the C API
                            const char* json_c_str = nullptr;
                            size_t json_length = 0;
                            enum RemDbError json_err = remdb_get_json_string(value, 
                                &json_c_str, &json_length);
                            
                            if (json_err == REMDB_SUCCESS && json_c_str != nullptr) {
                                value_str = std::string(json_c_str, json_length);
                                remdb_free_string(json_c_str);
                                printf("DEBUG C++ get(): JSON value: '%s'\n", value_str.c_str());
                                fflush(stdout);
                            } else {
                                value_str = "{}";
                                printf("DEBUG C++ get(): Failed to get JSON string, error: %d\n", json_err);
                                fflush(stdout);
                            }
                            break;
                        }
                        default:
                            value_str = "";
                            break;
                    }
                
                record.push_back(value_str);
            }
            
            remdb_free_result_set(result_set);
            return true;
        }
        
        if (result_set) {
            remdb_free_result_set(result_set);
        }
        
        return false;
    }
    
    std::shared_ptr<ZeroCopyData> get_zero_copy(const std::string& key) {
        // Build SQL SELECT statement
        std::string sql = "SELECT * FROM " + table_name_ + " WHERE 1=1 LIMIT 1";
        
        ::RemDbResultSet* result_set = nullptr;
        enum RemDbError err = remdb_sql_query(db_handle_, sql.c_str(), &result_set);
        
        if (err == REMDB_SUCCESS && result_set && result_set->rows_count > 0) {
            // Simple implementation: convert first row to string
            const ::RemDbResultRow* row = &result_set->rows[0];
            std::string data;
            
            for (size_t j = 0; j < row->values_count; j++) {
                const ::RemDbTypedValue* value = &row->values[j];
                std::string value_str;
                
                // Convert value based on data type
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
                            value_str = value->value.boolean ? "true" : "false";
                            break;
                        case REMDB_TYPE_TIMESTAMP:
                            value_str = std::to_string(value->value.timestamp);
                            break;
                        case REMDB_TYPE_STRING:
                            value_str = std::string(reinterpret_cast<const char*>(value->value.string));
                            // Remove trailing null character
                            value_str = value_str.substr(0, value_str.find('\0'));
                            break;
                        case REMDB_TYPE_JSON: {
                            // For JSON type, we need to get actual JSON string using the C API
                            const char* json_c_str = nullptr;
                            size_t json_length = 0;
                            enum RemDbError json_err = remdb_get_json_string(value, 
                                &json_c_str, &json_length);
                            
                            if (json_err == REMDB_SUCCESS && json_c_str != nullptr) {
                                value_str = std::string(json_c_str, json_length);
                                remdb_free_string(json_c_str);
                                printf("DEBUG C++ get_zero_copy(): JSON value: '%s'\n", value_str.c_str());
                                fflush(stdout);
                            } else {
                                value_str = "{}";
                                printf("DEBUG C++ get_zero_copy(): Failed to get JSON string, error: %d\n", json_err);
                                fflush(stdout);
                            }
                            break;
                        }
                        default:
                            value_str = "";
                            break;
                    }
                
                if (j > 0) {
                    data += ",";
                }
                data += value_str;
            }
            
            remdb_free_result_set(result_set);
            return std::make_shared<ZeroCopyData>(data);
        }
        
        if (result_set) {
            remdb_free_result_set(result_set);
        }
        
        return std::make_shared<ZeroCopyData>("");
    }
    
    bool update(const std::string& key, const std::map<std::string, std::string>& record) {
        if (record.empty()) {
            return false;
        }
        
        // Build SQL UPDATE statement
        std::string sql = "UPDATE " + table_name_ + " SET ";
        
        for (const auto& pair : record) {
            if (sql.back() != ' ') {
                sql += ", ";
            }
            sql += pair.first + " = '" + pair.second + "'";
        }
        
        // Assume first field is primary key
        sql += " WHERE 1=1";
        
        return execute_sql(sql);
    }
    
    bool update(const std::string& key, const std::vector<std::string>& record) {
        // Not implemented yet
        return false;
    }
    
    bool delete_record(const std::string& key) {
        // Build SQL DELETE statement
        std::string sql = "DELETE FROM " + table_name_ + " WHERE 1=1";
        
        return execute_sql(sql);
    }
    
    int get_record_count() {
        size_t count = 0;
        enum RemDbError err = remdb_table_get_record_count(db_handle_, table_id_, &count);
        if (err == REMDB_SUCCESS) {
            return static_cast<int>(count);
        }
        return 0;
    }
    
    py::array get_column_as_numpy(const std::string& column_name) {
        // Build SQL SELECT statement
        std::string sql = "SELECT " + column_name + " FROM " + table_name_;
        
        ::RemDbResultSet* result_set = nullptr;
        enum RemDbError err = remdb_sql_query(db_handle_, sql.c_str(), &result_set);
        
        if (err == REMDB_SUCCESS && result_set) {
            std::vector<double> values;
            
            for (size_t i = 0; i < result_set->rows_count; i++) {
                const ::RemDbResultRow* row = &result_set->rows[i];
                if (row->values_count > 0) {
                    const ::RemDbTypedValue* value = &row->values[0];
                    double double_value = 0.0;
                    
                    // Convert value based on data type
                    switch (value->data_type) {
                        case REMDB_TYPE_UINT8:
                            double_value = static_cast<double>(value->value.u8);
                            break;
                        case REMDB_TYPE_UINT16:
                            double_value = static_cast<double>(value->value.u16);
                            break;
                        case REMDB_TYPE_UINT32:
                            double_value = static_cast<double>(value->value.u32);
                            break;
                        case REMDB_TYPE_UINT64:
                            double_value = static_cast<double>(value->value.u64);
                            break;
                        case REMDB_TYPE_FLOAT32:
                            double_value = static_cast<double>(value->value.float32);
                            break;
                        case REMDB_TYPE_FLOAT64:
                            double_value = value->value.float64;
                            break;
                        case REMDB_TYPE_BOOL:
                            double_value = value->value.boolean ? 1.0 : 0.0;
                            break;
                        case REMDB_TYPE_TIMESTAMP:
                            double_value = static_cast<double>(value->value.timestamp);
                            break;
                        default:
                            double_value = 0.0;
                            break;
                    }
                    
                    values.push_back(double_value);
                }
            }
            
            remdb_free_result_set(result_set);
            return py::array_t<double>(values.size(), values.data());
        }
        
        std::vector<double> values;
        return py::array_t<double>(values.size(), values.data());
    }
    
    std::vector<std::pair<std::string, double>> vector_search(const std::string& field_name, const std::vector<double>& query_vector, int k) {
        std::vector<std::pair<std::string, double>> results;
        
        // Build SQL SELECT statement
        std::string sql = "SELECT *, DISTANCE(" + field_name + ", [";
        
        for (size_t i = 0; i < query_vector.size(); i++) {
            if (i > 0) {
                sql += ", ";
            }
            sql += std::to_string(query_vector[i]);
        }
        
        sql += ") AS distance FROM " + table_name_ + " ORDER BY distance LIMIT " + std::to_string(k);
        
        ::RemDbResultSet* result_set = nullptr;
        enum RemDbError err = remdb_sql_query(db_handle_, sql.c_str(), &result_set);
        
        if (err == REMDB_SUCCESS && result_set) {
            for (size_t i = 0; i < result_set->rows_count; i++) {
                const ::RemDbResultRow* row = &result_set->rows[i];
                if (row->values_count > 1) {
                    // Assume first field is primary key
                    const ::RemDbTypedValue* key_value = &row->values[0];
                    const ::RemDbTypedValue* distance_value = &row->values[row->values_count - 1];
                    
                    std::string key_str;
                    if (key_value->data_type == REMDB_TYPE_STRING) {
                        key_str = std::string(reinterpret_cast<const char*>(key_value->value.string));
                        key_str = key_str.substr(0, key_str.find('\0'));
                    } else {
                        key_str = std::to_string(key_value->value.u64);
                    }
                    
                    double distance = 0.0;
                    if (distance_value->data_type == REMDB_TYPE_FLOAT64) {
                        distance = distance_value->value.float64;
                    } else if (distance_value->data_type == REMDB_TYPE_FLOAT32) {
                        distance = static_cast<double>(distance_value->value.float32);
                    }
                    
                    results.emplace_back(key_str, distance);
                }
            }
            
            remdb_free_result_set(result_set);
        }
        
        return results;
    }
};

class RemDbTransaction {
private:
    bool active_;
    RemDbHandle db_handle_;

public:
    RemDbTransaction(RemDbHandle db_handle) : active_(true), db_handle_(db_handle) {}
    
    bool commit() {
        if (active_) {
            enum RemDbError err = remdb_commit_transaction(db_handle_);
            active_ = false;
            return err == REMDB_SUCCESS;
        }
        return false;
    }
    
    bool rollback() {
        if (active_) {
            enum RemDbError err = remdb_rollback_transaction(db_handle_);
            active_ = false;
            return err == REMDB_SUCCESS;
        }
        return false;
    }
    
    bool is_active() {
        return active_;
    }
};

class RemDbPythonResultSet {
private:
    std::vector<std::string> columns_;
    std::vector<std::vector<std::string>> rows_;
    enum RemDbError error_code_;

public:
    RemDbPythonResultSet(RemDbHandle db_handle, const std::string& sql) {
        std::cerr << "DEBUG C++ RemDbPythonResultSet constructor: SQL='" << sql << "'" << std::endl;
        
        ::RemDbResultSet* result_set = nullptr;
        enum RemDbError err = remdb_sql_query(db_handle, sql.c_str(), &result_set);
        error_code_ = err;
        
        std::cerr << "DEBUG C++ RemDbPythonResultSet constructor: err=" << (int)err << ", result_set=" << result_set << std::endl;
        
        if (err == REMDB_SUCCESS && result_set) {
            std::cerr << "DEBUG C++ RemDbPythonResultSet constructor: columns_count=" << result_set->columns_count << ", rows_count=" << result_set->rows_count << std::endl;
            
            // Extract column names
            for (size_t i = 0; i < result_set->columns_count; i++) {
                if (result_set->columns[i]) {
                    columns_.push_back(result_set->columns[i]);
                    std::cerr << "DEBUG C++ RemDbPythonResultSet constructor: column " << i << ": '" << result_set->columns[i] << "'" << std::endl;
                } else {
                    columns_.push_back("column_" + std::to_string(i));
                }
            }
            
            // Extract row data
            for (size_t i = 0; i < result_set->rows_count; i++) {
                const ::RemDbResultRow* row = &result_set->rows[i];
                std::vector<std::string> row_data;
                
                for (size_t j = 0; j < row->values_count; j++) {
                    // 注意：row->values 是一个指针数组，所以我们需要直接使用 &row->values[j]
                    const ::RemDbTypedValue* value = &row->values[j];
                    std::string value_str;
                    
                    // Debug: print data type
                    std::cerr << "DEBUG C++ get_row(): Processing value, type: " << static_cast<unsigned long long>(value->data_type) << ", REMDB_TYPE_JSON: " << static_cast<unsigned long long>(REMDB_TYPE_JSON) << std::endl;
                    
                    // Handle different data types
                    // Extract low byte of data_type (may be corrupted due to ABI mismatch)
                    uint8_t effective_type = static_cast<uint8_t>(value->data_type & 0xFF);
                    switch (effective_type) {
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
                            value_str = value->value.boolean ? "TRUE" : "FALSE";
                            break;
                        case REMDB_TYPE_TIMESTAMP:
                            value_str = std::to_string(value->value.timestamp);
                            break;
                        case REMDB_TYPE_STRING:
                            {
                                value_str = std::string(reinterpret_cast<const char*>(value->value.string), REMDB_MAX_STRING_LEN);
                                // Trim null terminator
                                size_t len = strnlen(value_str.c_str(), REMDB_MAX_STRING_LEN);
                                value_str.resize(len);
                            }
                            break;
                        case REMDB_TYPE_JSON:
                            {
                                const char* json_str = nullptr;
                                size_t json_len = 0;
                                enum RemDbError err = remdb_get_json_string(value, &json_str, &json_len);
                                if (err == REMDB_SUCCESS && json_str) {
                                    value_str = std::string(json_str, json_len);
                                } else {
                                    value_str = "{}";
                                }
                            }
                            break;
                        case REMDB_TYPE_VECTOR:
                            // Not implemented
                            value_str = "[VECTOR]";
                            break;
                        default:
                            // Unknown type, try to interpret as integer
                            try {
                                int32_t int32_value;
                                memcpy(&int32_value, &value->value.u32, sizeof(int32_t));
                                value_str = std::to_string(int32_value);
                                std::cerr << "DEBUG C++ get_row(): Fallback Int32 value: '" << value_str << "'" << std::endl;
                            } catch (...) {
                                value_str = "";
                            }
                            break;
                    }
                    
                    row_data.push_back(value_str);
                }
                
                rows_.push_back(row_data);
            }
            
            // Free result set
            remdb_free_result_set(result_set);
        } else {
            // Query failed, return empty result
            columns_ = std::vector<std::string>();
            rows_ = std::vector<std::vector<std::string>>();
        }
    }
    
    std::vector<std::string> get_columns() {
        return columns_;
    }
    
    int get_rows_count() {
        return static_cast<int>(rows_.size());
    }
    
    int get_error() {
        return static_cast<int>(error_code_);
    }
    
    py::dict get_row(int index) {
        py::dict row_dict;
        if (index >= 0 && index < static_cast<int>(rows_.size())) {
            const std::vector<std::string>& row_data = rows_[index];
            for (size_t i = 0; i < row_data.size() && i < columns_.size(); i++) {
                row_dict[py::cast(columns_[i])] = py::cast(row_data[i]);
            }
        }
        return row_dict;
    }
};

class RemDb {
private:
    bool connected_;
    std::string db_path_;
    RemDbHandle db_handle_;
    bool owns_handle_;  // Whether this instance owns the handle

public:
    RemDb() : connected_(false), db_handle_(nullptr), owns_handle_(false) {
        // Initialize with null handle, connect() will initialize the database
    }

    ~RemDb() {
        // Don't close the global handle - it's shared across all instances
        // The global handle is managed by the module lifecycle
        connected_ = false;
        db_handle_ = nullptr;
    }

    bool connect(const std::string& db_path) {
        db_path_ = db_path;

        // Use mutex to ensure thread-safe initialization
        std::lock_guard<std::mutex> lock(g_db_mutex);

        // Check if we already have a global handle
        if (g_db_initialized.load() && g_global_db_handle != nullptr) {
            // Reuse the existing global handle
            db_handle_ = g_global_db_handle;
            owns_handle_ = false;
            connected_ = true;
            return true;
        }

        // Try to initialize the database
        enum RemDbError err = remdb_get_global(&db_handle_);
        if (err == REMDB_SUCCESS) {
            g_global_db_handle = db_handle_;
            g_db_initialized.store(true);
            owns_handle_ = false;
            connected_ = true;
        } else {
            // If remdb_get_global fails, try remdb_init_global with a struct that matches Rust's layout
            // Create a struct that matches the Rust RemDbConfig layout exactly
            struct RustRemDbConfig {
                const void* tables;
                size_t tables_count;
                const void* time_series_tables;
                size_t time_series_tables_count;
                size_t total_memory;
                uint8_t low_power_mode_supported;
                int32_t low_power_max_records;
                const void* ha_config;
            };

            RustRemDbConfig config;
            config.tables = nullptr;
            config.tables_count = 0;
            config.time_series_tables = nullptr;
            config.time_series_tables_count = 0;
            config.total_memory = 1024 * 1024 * 1024; // 1GB
            config.low_power_mode_supported = 0;
            config.low_power_max_records = -1;
            config.ha_config = nullptr;

            err = remdb_init_global((const ::RemDbConfig*)&config, &db_handle_);
            if (err == REMDB_SUCCESS) {
                g_global_db_handle = db_handle_;
                g_db_initialized.store(true);
                owns_handle_ = false;
                connected_ = true;
            } else {
                // Print error for debugging
                printf("Both remdb_get_global and remdb_init_global failed with error code: %d\n", err);
                fflush(stdout);
            }
        }

        return connected_;
    }

    bool is_connected() {
        return connected_;
    }

    std::shared_ptr<RemDbTable> get_table(const std::string& table_name) {
        if (!connected_) {
            return nullptr;
        }

        size_t table_id = 0;
        enum RemDbError err = remdb_table_get_by_name(db_handle_, table_name.c_str(), &table_id);
        if (err == REMDB_SUCCESS) {
            return std::make_shared<RemDbTable>(table_name, db_handle_, table_id);
        }

        return nullptr;
    }

    std::shared_ptr<RemDbTransaction> begin_transaction() {
        if (!connected_) {
            return nullptr;
        }

        enum RemDbError err = remdb_begin_transaction(db_handle_, REMDB_TX_WRITE, REMDB_ISO_READ_COMMITTED);
        if (err == REMDB_SUCCESS) {
            return std::make_shared<RemDbTransaction>(db_handle_);
        }

        return nullptr;
    }

    std::shared_ptr<RemDbPythonResultSet> execute_query(const std::string& sql) {
        if (!connected_) {
            return nullptr;
        }

        fprintf(stderr, "DEBUG C++ RemDb::execute_query: sql='%s'\n", sql.c_str());
        fflush(stderr);
        return std::make_shared<RemDbPythonResultSet>(db_handle_, sql);
    }

    bool save_snapshot(const std::string& path) {
        if (!connected_) {
            return false;
        }

        enum RemDbError err = remdb_save_snapshot(db_handle_, path.c_str());
        return err == REMDB_SUCCESS;
    }

    bool restore_snapshot(const std::string& path) {
        if (!connected_) {
            return false;
        }

        enum RemDbError err = remdb_restore_snapshot(db_handle_, path.c_str());
        return err == REMDB_SUCCESS;
    }
};

PYBIND11_MODULE(_remdb, m) {
    std::cerr << "DEBUG C++: _remdb module loading" << std::endl;
    m.doc() = "RemDB Python bindings";
    
    py::class_<ZeroCopyData, std::shared_ptr<ZeroCopyData>>(m, "ZeroCopyData")
        .def(py::init<const std::string&>())
        .def("tobytes", &ZeroCopyData::tobytes);
    
    py::class_<RemDbTable, std::shared_ptr<RemDbTable>>(m, "RemDbTable")
        .def(py::init<const std::string&, RemDbHandle, size_t>())
        .def("insert", py::overload_cast<const std::map<std::string, std::string>&>(&RemDbTable::insert))
        .def("insert", py::overload_cast<const std::vector<std::string>&>(&RemDbTable::insert))
        .def("get", &RemDbTable::get)
        .def("get_zero_copy", &RemDbTable::get_zero_copy)
        .def("update", py::overload_cast<const std::string&, const std::map<std::string, std::string>&>(&RemDbTable::update))
        .def("update", py::overload_cast<const std::string&, const std::vector<std::string>&>(&RemDbTable::update))
        .def("delete_record", &RemDbTable::delete_record)
        .def("get_record_count", &RemDbTable::get_record_count)
        .def("get_column_as_numpy", &RemDbTable::get_column_as_numpy)
        .def("vector_search", &RemDbTable::vector_search);
    
    py::class_<RemDbTransaction, std::shared_ptr<RemDbTransaction>>(m, "RemDbTransaction")
        .def(py::init<RemDbHandle>())
        .def("commit", &RemDbTransaction::commit)
        .def("rollback", &RemDbTransaction::rollback)
        .def("is_active", &RemDbTransaction::is_active);
    
    py::class_<RemDbPythonResultSet, std::shared_ptr<RemDbPythonResultSet>>(m, "RemDbResultSet")
        .def(py::init<RemDbHandle, const std::string&>())
        .def("get_columns", &RemDbPythonResultSet::get_columns)
        .def("get_rows_count", &RemDbPythonResultSet::get_rows_count)
        .def("get_row", &RemDbPythonResultSet::get_row)
        .def("get_error", &RemDbPythonResultSet::get_error);
    
    py::class_<RemDb>(m, "RemDb")
        .def(py::init<>())
        .def("connect", &RemDb::connect)
        .def("is_connected", &RemDb::is_connected)
        .def("get_table", &RemDb::get_table)
        .def("begin_transaction", &RemDb::begin_transaction)
        .def("execute_query", &RemDb::execute_query)
        .def("save_snapshot", &RemDb::save_snapshot)
        .def("restore_snapshot", &RemDb::restore_snapshot);
    
    m.attr("__version__") = "0.1.0";
    m.attr("connected") = false;
    
    m.def("test", []() {
        return "RemDB Python bindings initialized successfully!";
    });
}