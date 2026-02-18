## 实现计划

### 目标
在 `remdb/src/sql/query_executor.rs` 中实现 sql_language.md 文档中定义的 12 个 JSON 函数。

### 实现步骤

#### 1. 在 `execute_function_call` 函数中添加 JSON 函数分支
在 `match name.to_uppercase().as_str()` 中添加以下分支：
- `JSON_EXTRACT` → execute_json_extract
- `JSON_VALUE` → execute_json_value
- `JSON_QUERY` → execute_json_query
- `JSON_HAS` → execute_json_has
- `JSON_TYPE` → execute_json_type
- `JSON_ARRAY_LENGTH` → execute_json_array_length
- `JSON_ARRAY` → execute_json_array
- `JSON_OBJECT` → execute_json_object
- `JSON_SET` → execute_json_set
- `JSON_REMOVE` → execute_json_remove
- `JSON_MERGE_PATCH` → execute_json_merge_patch
- `JSON_ARRAY_APPEND` → execute_json_array_append

#### 2. 实现各个函数的执行逻辑

**JSON 提取函数：**
- `execute_json_extract(args)` - 使用 `JsonPath::execute()` 提取值，返回 JSON 类型
- `execute_json_value(args)` - 提取标量值，返回对应的 SQL 类型
- `execute_json_query(args)` - 提取对象或数组，返回 JSON 类型
- `execute_json_has(args)` - 检查路径是否存在，返回 BOOLEAN 类型
- `execute_json_type(args)` - 返回值的类型字符串（如 "string", "number", "boolean"）

**JSON 创建函数：**
- `execute_json_array_length(args)` - 计算数组长度，返回 INTEGER 类型
- `execute_json_array(args)` - 将参数转换为 JSON 数组，返回 JSON 类型
- `execute_json_object(args)` - 将键值对转换为 JSON 对象，返回 JSON 类型

**JSON 修改函数：**
- `execute_json_set(args)` - 设置路径的值，返回修改后的 JSON
- `execute_json_remove(args)` - 删除路径的值，返回修改后的 JSON
- `execute_json_merge_patch(args)` - 合并两个 JSON，返回合并后的 JSON
- `execute_json_array_append(args)` - 向数组追加元素，返回修改后的 JSON

#### 3. 运行测试验证
运行测试文件 `remdb/tests/json_functions_test.rs` 中的 14 个测试用例，确保所有函数正常工作。

### 技术要点
- 使用现有的 `JsonPath::new()` 和 `execute()` 方法进行路径查询
- 使用 `JsonDocument::from_json()` 和 `to_json()` 进行 JSON 序列化/反序列化
- 正确处理 `JsonQueryResult` 到 `TypedValue` 的转换
- 处理错误情况（无效路径、无效 JSON 等）
- 确保类型转换正确（JSON 值到 SQL 类型的映射）

### 预期结果
- 所有 12 个 JSON 函数可以在 SQL 查询中使用
- 测试文件中的 14 个测试用例全部通过
- 符合 sql_language.md 文档中的函数规范