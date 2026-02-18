# 测试框架实现计划

## 目标
为remdb-python创建一个全面的测试框架，对`sql_language.md`文档中描述的所有SQL语法和逻辑实现测试全覆盖。

## 现有测试分析
当前remdb-python/tests/目录包含6个测试文件，主要测试核心功能、网络连接和基本操作，但缺乏对SQL语法的系统测试覆盖。

## 测试框架设计

### 1. 目录结构重组
```
remdb-python/tests/
├── __init__.py
├── conftest.py (pytest配置文件，可选)
├── unit/                    # 单元测试
│   ├── test_data_types.py
│   ├── test_ddl.py
│   ├── test_dml.py
│   ├── test_functions.py
│   ├── test_operators.py
│   ├── test_indexes.py
│   ├── test_transactions.py
│   ├── test_timeseries.py
│   ├── test_vectors.py
│   └── test_constraints.py
├── integration/            # 集成测试
│   ├── test_sql_compliance.py
│   ├── test_network_sql.py
│   └── test_mixed_operations.py
├── fixtures/              # 测试夹具
│   ├── database.py
│   └── data_generators.py
└── utils/                 # 测试工具
    ├── validators.py
    └── coverage_tracker.py
```

### 2. 核心测试组件

#### 基础测试类 (BaseTestCase)
- 提供`setUp()`和`tearDown()`方法管理数据库连接
- 支持本地文件模式和网络连接模式
- 提供通用的断言方法用于SQL结果验证

#### 测试覆盖范围（基于sql_language.md）

##### 数据类型测试
- INTEGER (Int32/Int64/UInt32/UInt64)
- REAL (Float32/Float64)
- TEXT (字符串，最大64字节)
- BOOLEAN
- TIMESTAMP
- VECTOR(dim) (1-4096维度)

##### SQL语法测试
1. **数据库管理语句**
   - CREATE DATABASE
   - USE DATABASE
   - CLOSE DATABASE
   - DROP DATABASE

2. **SELECT语句及其子句**
   - DISTINCT子句
   - 列别名和表别名
   - GROUP BY子句（单列、多列、与聚合函数组合）
   - JOIN子句（INNER, LEFT, RIGHT, FULL）
   - WHERE条件（比较运算符、逻辑运算符）
   - ORDER BY子句（ASC/DESC）
   - LIMIT子句

3. **DML语句**
   - INSERT (指定列名插入、全列插入)
   - UPDATE (带WHERE条件、不带WHERE条件)
   - DELETE (带WHERE条件、不带WHERE条件)

4. **DDL语句**
   - CREATE TABLE (各种数据类型、约束)
   - ALTER TABLE (ADD COLUMN, MODIFY COLUMN, DROP COLUMN)
   - DROP TABLE (IF EXISTS, CASCADE, RESTRICT, DEFERRED)
   - CREATE TIMESERIES TABLE (WITH COMPRESSION, WITH TTL)

5. **索引管理**
   - CREATE INDEX (标量索引：BTREE, TTREE; 向量索引：HNSW, IVF等)
   - SHOW INDEX BUILD STATUS
   - 索引参数测试
   - 在线/离线索引创建

6. **事务管理**
   - BEGIN TRANSACTION
   - COMMIT
   - ROLLBACK
   - 事务隔离级别测试

##### 运算符测试
- 比较运算符: =, <>, !=, >, >=, <, <=
- LIKE运算符: %, _, 转义字符
- 逻辑运算符: AND, OR
- 向量距离运算符: <-> (L2), <#> (内积), <=> (余弦相似度)

##### 函数测试
1. **聚合函数**
   - COUNT, SUM, AVG, MIN, MAX
   - VAR, STDDEV (总体)
   - VAR_SAMP, STDDEV_SAMP (样本)

2. **滑动窗口函数**
   - MOVING_SUM, MOVING_AVERAGE

3. **字符串函数**
   - CONCAT, SUBSTRING, UPPER, LOWER

4. **数学函数**
   - ABS, SQRT, POWER, SIN, COS, LOG, EXP, ROUND, CEIL, FLOOR, MOD

5. **时间函数**
   - TIME_BUCKET (多种时间间隔格式)
   - TO_ISO8601, TO_CHAR, TO_EPOCH

##### 时序功能测试
- 时序表创建语法
- 时间范围查询
- 时间转换函数
- 压缩算法测试 (delta, runlength, delta-delta等)

##### 向量功能测试
- 向量数据类型创建
- 向量距离运算符
- 向量搜索 (精确搜索、近似搜索)
- 混合搜索 (向量+标量过滤)
- 向量索引性能测试

##### 边界条件和错误处理
- 无效SQL语法测试
- 数据类型不匹配测试
- 约束违反测试
- 权限错误测试
- 资源限制测试

### 3. 测试数据管理
- 创建标准化的测试数据集
- 支持不同数据类型的样本数据
- 为时序测试生成时间序列数据
- 为向量测试生成高维向量数据

### 4. 测试执行策略
- 单元测试：快速验证单个功能
- 集成测试：验证多个组件协同工作
- 性能测试：针对关键操作进行基准测试
- 并发测试：测试多线程/多进程场景

### 5. 测试工具和基础设施
- 使用pytest或unittest框架
- 添加测试覆盖率报告 (coverage.py)
- 创建测试配置文件
- 支持测试参数化
- 添加测试日志记录

### 6. 与现有测试的集成
- 保持向后兼容性
- 将现有测试迁移到新结构
- 确保所有现有测试继续通过

## 实施步骤

1. **第一阶段：基础设施搭建**
   - 创建新的测试目录结构
   - 实现BaseTestCase基础类
   - 设置测试数据库管理工具

2. **第二阶段：核心SQL功能测试**
   - 实现数据类型测试
   - 实现DDL语句测试
   - 实现DML语句测试

3. **第三阶段：高级功能测试**
   - 实现函数测试
   - 实现运算符测试
   - 实现索引测试

4. **第四阶段：专业功能测试**
   - 实现时序功能测试
   - 实现向量功能测试
   - 实现事务测试

5. **第五阶段：集成和优化**
   - 集成所有测试模块
   - 优化测试性能
   - 添加测试覆盖率报告
   - 创建CI/CD集成脚本

## 预期成果
1. 完整的SQL语法测试覆盖率达到90%以上
2. 可维护的测试框架，便于添加新测试
3. 详细的测试报告和覆盖率报告
4. 支持本地和网络两种连接模式的测试
5. 与现有CI/CD流程集成

## 技术栈
- Python 3.7+
- unittest/pytest
- coverage.py
- 可能需要的额外库：numpy, pandas (用于向量和数据分析测试)

这个测试框架将确保RemDB的Python绑定能够正确处理所有文档化的SQL功能，并提供高质量的错误处理和边界条件测试。