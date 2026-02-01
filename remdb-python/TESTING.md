# RemDB Python 测试框架文档

本文档描述了 RemDB Python 绑定的测试框架结构、使用方法和最佳实践。

## 测试框架结构

测试框架采用分层架构，按照功能模块和测试类型进行组织：

```
tests/
├── fixtures/           # 测试夹具和基础测试类
│   ├── __init__.py
│   └── database.py     # 数据库连接和测试基础类
├── integration/        # 集成测试
│   └── __init__.py
├── unit/               # 单元测试
│   ├── __init__.py
│   ├── test_data_types.py    # 数据类型测试
│   ├── test_ddl.py           # DDL 语句测试
│   ├── test_dml.py           # DML 语句测试
│   ├── test_functions.py     # 函数测试
│   ├── test_operators.py     # 运算符测试
│   ├── test_indexes.py       # 索引测试
│   ├── test_timeseries.py    # 时间序列测试
│   ├── test_vectors.py       # 向量测试
│   └── test_transactions.py  # 事务测试
├── utils/              # 测试工具
│   ├── __init__.py
│   ├── data_generators.py    # 测试数据生成
│   └── validators.py         # 验证工具
├── __init__.py
├── test_config.py      # 测试配置
├── run_tests.py        # 测试运行器
└── .coveragerc         # 覆盖率配置
```

## 核心组件

### 基础测试类

- **LocalTestCase**: 用于测试本地文件数据库连接
- **NetworkTestCase**: 用于测试网络数据库连接
- **BaseTestCase**: 基础测试类，提供通用测试方法

### 测试工具

- **data_generators.py**: 生成各种类型的测试数据
- **validators.py**: 验证测试结果
- **test_config.py**: 测试配置和性能优化选项

### 测试运行器

- **run_tests.py**: 命令行测试运行器，支持多种测试配置选项

## 测试覆盖范围

测试框架覆盖了以下 SQL 功能：

### 数据类型
- INTEGER
- REAL
- TEXT
- BOOLEAN
- TIMESTAMP
- VECTOR

### DDL 语句
- CREATE TABLE
- ALTER TABLE
- DROP TABLE
- CREATE TIMESERIES TABLE
- SHOW TABLES

### DML 语句
- INSERT
- SELECT
- UPDATE
- DELETE

### 函数
- 聚合函数 (COUNT, SUM, AVG, MAX, MIN)
- 字符串函数 (SUBSTRING, LENGTH, CONCAT)
- 数学函数 (ABS, CEIL, FLOOR, ROUND)
- 时间函数 (CURRENT_TIMESTAMP, DATE_TRUNC)
- 移动窗口函数
- 向量搜索函数

### 运算符
- 比较运算符 (=, !=, <, >, <=, >=)
- 逻辑运算符 (AND, OR, NOT)
- LIKE 运算符
- 向量距离运算符
- 算术运算符 (+, -, *, /)

### 索引
- 标量索引 (BTREE, TTREE)
- 向量索引 (HNSW, IVF, HNSW_SQ, HNSW_BQ, IVF_FLAT, IVF_PQ)
- 索引构建状态
- REINDEX 操作
- 索引使用情况

### 时间序列
- 时间序列表操作
- 移动窗口函数
- 时间序列压缩
- 时间序列边界情况

### 向量
- 向量数据类型
- 向量搜索函数
- 混合搜索
- 向量边界情况
- 向量性能测试

### 事务
- 基础事务操作 (BEGIN, COMMIT, ROLLBACK)
- 事务隔离级别
- 事务错误处理
- 事务并发
- 事务边界情况

## 运行测试

### 使用测试运行器

```bash
# 运行所有测试
python run_tests.py

# 运行单元测试
python run_tests.py --unit

# 运行集成测试
python run_tests.py --integration

# 详细输出
python run_tests.py --verbose

# 列出所有测试
python run_tests.py --list

# 运行覆盖率测试
python run_tests.py --coverage

# 生成 HTML 覆盖率报告
python run_tests.py --coverage-html
```

### 使用标准 unittest

```bash
# 发现并运行所有测试
python -m unittest discover tests

# 运行特定测试文件
python -m unittest tests.unit.test_data_types

# 运行特定测试类
python -m unittest tests.unit.test_data_types.TestDataTypeINTEGER

# 运行特定测试方法
python -m unittest tests.unit.test_data_types.TestDataTypeINTEGER.test_integer_column_creation
```

## 测试配置

测试框架通过 `tests/test_config.py` 提供了丰富的配置选项：

### 性能优化设置
- `timeout_seconds`: 默认测试超时时间
- `retry_attempts`: 不稳定测试的重试次数
- `sleep_interval`: 重试间隔
- `default_num_rows`: 测试表的默认行数
- `default_num_vectors`: 向量测试的默认向量数
- `default_num_timeseries`: 时间序列测试的默认点数

### 环境检测
测试框架会自动检测运行环境并调整配置：
- CI 环境：启用详细日志，延长超时时间
- 资源受限环境：减少测试数据量
- Windows 环境：调整睡眠间隔

### 跳过测试条件
可以通过环境变量或配置文件设置跳过特定类型的测试：
- `skip_network_tests`: 跳过网络测试
- `skip_vector_tests`: 跳过向量测试
- `skip_timeseries_tests`: 跳过时间序列测试
- `skip_index_tests`: 跳过索引测试
- `skip_transaction_tests`: 跳过事务测试

## 添加新测试

### 创建新测试模块

1. **在 `tests/unit/` 目录下创建新的测试文件**，例如 `test_new_feature.py`

2. **编写测试类**，继承自 `LocalTestCase` 或 `NetworkTestCase`：

```python
import unittest
from tests.fixtures import LocalTestCase

class TestNewFeature(LocalTestCase):
    def setUp(self):
        super().setUp()
        # 初始化测试环境
    
    def test_new_feature_basic(self):
        # 测试基本功能
        pass
    
    def test_new_feature_edge_cases(self):
        # 测试边界情况
        pass
```

3. **在 `tests/unit/__init__.py` 中导出测试类**：

```python
from .test_new_feature import TestNewFeature

__all__ = [
    # 现有测试类...
    'TestNewFeature'
]
```

### 测试最佳实践

1. **测试隔离**：每个测试应该独立运行，不依赖其他测试的状态
2. **边界情况**：测试各种边界情况和异常输入
3. **性能测试**：对于性能敏感的功能，添加性能测试
4. **错误处理**：测试错误处理和异常情况
5. **覆盖率**：确保新功能的测试覆盖率达到 100%

### 使用测试工具

```python
from tests.utils.data_generators import generate_test_table_data, generate_vector_test_data
from tests.utils.validators import validate_vector_similarity

# 生成测试数据
data = generate_test_table_data(100, schema={
    'id': 'INTEGER',
    'name': 'TEXT',
    'value': 'REAL'
})

# 生成向量测试数据
vector_data = generate_vector_test_data(1000, dimensions=128)

# 验证向量相似度
is_similar = validate_vector_similarity(vector1, vector2, threshold=0.9)
```

## 测试覆盖率

### 运行覆盖率测试

```bash
# 安装 coverage 模块
pip install coverage

# 运行覆盖率测试
python run_tests.py --coverage

# 生成 HTML 报告
python run_tests.py --coverage-html

# 生成 XML 报告（用于 CI）
python run_tests.py --coverage-xml
```

### 覆盖率报告

- **HTML 报告**: 生成在 `htmlcov/` 目录，可在浏览器中查看
- **XML 报告**: 生成 `coverage.xml` 文件，用于 CI 系统
- **控制台报告**: 显示每个模块的覆盖率百分比

### 覆盖率目标

- **核心功能**: 100%
- **辅助功能**: 90%+
- **测试工具**: 80%+

## 性能优化

### 测试性能建议

1. **使用内存高效测试**: 设置 `use_memory_efficient_testing=True`
2. **减少测试数据量**: 在资源受限环境中使用较小的测试数据集
3. **并行测试**: 在 CI 环境中启用并行测试
4. **合理设置超时**: 根据测试环境调整超时时间
5. **避免重复设置**: 使用 setUpClass() 进行一次性设置

### 环境变量优化

```bash
# 资源受限环境
set RESOURCE_CONSTRAINED=1

# CI 环境
set CI=1

# 启用详细日志
set TEST_LOGGING=1
```

## 故障排除

### 常见问题

1. **ModuleNotFoundError: No module named '_remdb'**
   - 原因: C 扩展模块未编译
   - 解决: 运行 `python setup.py build_ext --inplace` 编译扩展

2. **Network server not available**
   - 原因: 网络服务器未运行
   - 解决: 启动 RemDB 服务器或使用本地测试模式

3. **Test timeout**
   - 原因: 测试执行时间超过限制
   - 解决: 增加超时时间或优化测试数据量

4. **MemoryError**
   - 原因: 测试数据量过大
   - 解决: 减少测试数据量或使用内存高效测试

### 调试技巧

1. **启用详细日志**: `python run_tests.py --verbose`
2. **单独运行失败的测试**: `python -m unittest tests.unit.test_module.TestClass.test_method`
3. **使用 pdb 调试**: 在测试代码中添加 `import pdb; pdb.set_trace()`
4. **检查测试输出**: 查看测试输出的详细错误信息

## CI/CD 集成

### GitHub Actions 配置示例

```yaml
name: Python Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python
      uses: actions/setup-python@v5
      with:
        python-version: '3.10'
    
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install -e .[numpy,pandas]
        pip install coverage
    
    - name: Run tests with coverage
      run: |
        python run_tests.py --coverage
    
    - name: Upload coverage report
      uses: codecov/codecov-action@v4
      with:
        file: ./coverage.xml
```

### Jenkins 配置示例

```groovy
pipeline {
    agent any
    
    stages {
        stage('Install') {
            steps {
                sh 'pip install -e .[numpy,pandas]'
                sh 'pip install coverage'
            }
        }
        
        stage('Test') {
            steps {
                sh 'python run_tests.py --coverage'
            }
        }
        
        stage('Coverage') {
            steps {
                sh 'python run_tests.py --coverage-html'
            }
            post {
                always {
                    publishHTML(
                        target: [
                            allowMissing: false,
                            alwaysLinkToLastBuild: true,
                            keepAll: true,
                            reportDir: 'htmlcov',
                            reportName: 'Coverage Report'
                        ]
                    )
                }
            }
        }
    }
}
```

## 贡献指南

### 添加新测试

1. **遵循现有测试结构**：按照现有测试模块的结构和命名规范
2. **添加到正确的模块**：根据功能添加到相应的测试文件
3. **更新 __init__.py**：在导出列表中添加新的测试类
4. **运行测试**：确保新测试通过且不破坏现有测试
5. **检查覆盖率**：确保新功能的覆盖率达到目标

### 测试框架改进

1. **提交问题**：在 GitHub Issues 中报告测试框架的问题
2. **提交 PR**：提供测试框架改进的 Pull Request
3. **遵循编码规范**：使用与现有代码一致的编码风格
4. **添加文档**：为新功能添加相应的文档

## 结论

RemDB Python 测试框架提供了全面的测试覆盖，支持各种测试场景和环境。通过遵循本文档的指南，您可以：

- 运行现有的测试套件
- 添加新的测试用例
- 优化测试性能
- 集成测试到 CI/CD 系统
- 确保代码质量和稳定性

测试框架是确保 RemDB Python 绑定质量的重要工具，我们鼓励所有贡献者使用和改进它。