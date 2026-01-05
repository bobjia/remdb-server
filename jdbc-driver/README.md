# RemDb JDBC Driver

RemDb JDBC Driver 是一个用于连接 RemDb 数据库服务器的 JDBC 驱动程序，支持基本的 JDBC 操作，如连接数据库、执行查询和更新操作。

## 功能特性

- 支持基本的 JDBC 连接管理
- 支持 SQL 查询、插入、更新和删除操作
- 支持事务处理（自动提交模式）
- 支持结果集处理
- 支持预编译语句（PreparedStatement）
- 支持内嵌函数调用
- 支持时序表操作
- 支持 CREATE TABLE 和 CREATE INDEX 语句
- 支持 DESCRIBE TABLE 语句

## 系统要求

- Java 8 或更高版本
- RemDb Server 0.1.24 或更高版本

## 安装方法

### 方法一：使用 Maven 编译 JAR 文件

1. 编译 JDBC 驱动：
   ```bash
   cd jdbc-driver
   mvn clean package
   ```

2. 编译后的 JAR 文件将位于 `target/remdb-jdbc-driver-0.1.0.jar`

3. 将该 JAR 文件添加到你的 Java 项目的类路径中

### 方法二：使用 Maven 本地仓库

1. 发布到本地 Maven 仓库：
   ```bash
   cd jdbc-driver
   mvn clean install
   ```

2. 在你的 Maven 项目中添加依赖：
   ```xml
   <dependency>
       <groupId>cn.totaltrust.remdb</groupId>
       <artifactId>remdb-jdbc-driver</artifactId>
       <version>0.1.0</version>
   </dependency>
   ```

## 使用方法

### 1. 启动 RemDb Server

首先，你需要启动 RemDb Server，并配置 JDBC 监听端口：

```bash
# 使用默认配置启动（JDBC 端口 5432，最大连接数 100）
remdb-server --jdbc-port 5432 --max-connections 100
```

或者，你可以在配置文件 `remdb.toml` 中配置 JDBC 相关参数：

```toml
# remdb.toml
jdbc_port = 5432
max_connections = 100
```

然后使用配置文件启动：

```bash
remdb-server --config remdb.toml
```

### 2. 在 Java 代码中使用 JDBC 驱动

以下是一个简单的示例，展示如何使用 RemDb JDBC 驱动连接数据库、执行查询和更新操作：

```java
import java.sql.*;

public class RemDbExample {
    public static void main(String[] args) {
        String url = "jdbc:remdb://localhost:5432";
        String user = "";
        String password = "";

        try (Connection conn = DriverManager.getConnection(url, user, password);
             Statement stmt = conn.createStatement()) {

            // 创建表
            String createTableSQL = "CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY, name VARCHAR(50), age INT)";
            stmt.executeUpdate(createTableSQL);

            // 插入数据
            String insertSQL = "INSERT INTO users (id, name, age) VALUES (1, 'Alice', 25)";
            stmt.executeUpdate(insertSQL);

            // 查询数据
            String selectSQL = "SELECT id, name, age FROM users";
            try (ResultSet rs = stmt.executeQuery(selectSQL)) {
                while (rs.next()) {
                    int id = rs.getInt("id");
                    String name = rs.getString("name");
                    int age = rs.getInt("age");
                    System.out.printf("ID: %d, Name: %s, Age: %d%n", id, name, age);
                }
            }

        } catch (SQLException e) {
            e.printStackTrace();
        }
    }
}
```

### 3. 运行示例代码

#### 方法一：使用 Java 命令运行

1. 编译示例代码：
   ```bash
   cd jdbc-driver
   javac -cp target/remdb-jdbc-driver-0.1.0.jar src/main/java/cn/totaltrust/remdb/RemDbJdbcExample.java
   ```

2. 运行示例代码：
   ```bash
   java -cp .;target/remdb-jdbc-driver-0.1.0.jar cn.totaltrust.remdb.RemDbJdbcExample
   ```

#### 方法二：使用 Maven 运行

1. 运行示例代码：
   ```bash
   cd jdbc-driver
   mvn exec:java -Dexec.mainClass="cn.totaltrust.remdb.RemDbJdbcExample"
   ```

## JDBC URL 格式

RemDb JDBC 驱动的 URL 格式如下：

```
jdbc:remdb://host:port
```

- `host`: RemDb Server 的主机名或 IP 地址，默认为 `localhost`
- `port`: RemDb Server 的 JDBC 监听端口，默认为 `5432`

示例：
- 连接本地服务器，使用默认端口：`jdbc:remdb://localhost:5432`
- 连接远程服务器：`jdbc:remdb://192.168.1.100:5432`

## 配置参数

### 服务器端配置

在 RemDb Server 中，你可以配置以下 JDBC 相关参数：

| 参数名 | 类型 | 描述 | 默认值 |
|--------|------|------|--------|
| jdbc_port | u16 | JDBC 监听端口 | 6666 |
| max_connections | usize | 最大允许的并发客户端连接数 | 100 |

这些参数可以通过命令行参数或配置文件进行设置。

### 客户端配置

在 Java 应用程序中，你可以通过 JDBC URL 或连接属性配置以下参数：

| 属性名 | 类型 | 描述 | 默认值 |
|--------|------|------|--------|
| user | String | 用户名（目前未使用） | "" |
| password | String | 密码（目前未使用） | "" |

## 支持的 SQL 语句

RemDb JDBC 驱动支持以下 SQL 语句：

- `CREATE TABLE` - 创建表
- `CREATE TIMESERIES TABLE` - 创建时序表
- `CREATE INDEX` - 创建索引
- `DESCRIBE TABLE` - 描述表结构
- `INSERT` - 插入数据
- `SELECT` - 查询数据
- `UPDATE` - 更新数据
- `DELETE` - 删除数据
- `DROP TABLE` - 删除表

### 支持的函数

#### 基础统计聚合函数

- `COUNT` - 统计记录数量
- `SUM` - 计算数值总和
- `AVG` - 计算平均值
- `MIN` - 计算最小值
- `MAX` - 计算最大值

#### 时间窗口函数

- `TIME_BUCKET` - 将时间戳分组到指定的时间窗口

#### 时间间隔格式

`TIME_BUCKET` 函数支持多种时间间隔格式：

- 数值：微秒数
- `ns`：纳秒
- `us`：微秒
- `ms`：毫秒
- `s`/`sec`/`second`：秒
- `m`/`min`/`minute`：分钟
- `h`/`hr`/`hour`：小时
- `d`/`day`：天
- `w`/`week`：周

## 注意事项

1. 目前 RemDb 不支持事务，所有操作都是自动提交的
2. 不支持存储过程
3. 支持基本函数调用（聚合函数和窗口函数）
4. 支持索引（包括主键索引和辅助索引）
5. 不支持视图
6. 不支持 BLOB、CLOB 等大对象类型
7. 不支持批量操作
8. GROUP BY 支持有限（仅与聚合函数结合使用）

## 开发计划

- 支持事务处理
- 支持批量操作
- 支持更复杂的 SQL 语句
- 支持连接池
- 支持 SSL 加密连接

## 故障排除

### 连接失败

如果无法连接到 RemDb Server，请检查以下几点：

1. 确保 RemDb Server 已启动
2. 确保 JDBC 端口配置正确
3. 确保网络连接正常，防火墙没有阻止连接
4. 检查 RemDb Server 的日志，查看是否有错误信息

### SQL 执行失败

如果 SQL 执行失败，请检查以下几点：

1. 确保 SQL 语句语法正确
2. 确保表和列名正确
3. 确保数据类型匹配
4. 检查 RemDb Server 的日志，查看详细错误信息

## 许可证

RemDb JDBC Driver 采用 MIT 许可证开源。

## 联系方式

如有问题或建议，请联系 RemDb 开发团队。
