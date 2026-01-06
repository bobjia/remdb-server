-- remdb数据库模式定义

-- 创建用户表
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name varchar(16) UNIQUE NOT NULL,
    email TEXT NOT NULL,
    age INT,
    created_at timestamptz(6)
);

-- 创建产品表
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    price DOUBLE NOT NULL,
    description TEXT,
    stock INT NOT NULL DEFAULT 0
);

-- 创建订单表
CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INT NOT NULL,
    product_id INT NOT NULL,
    quantity INT NOT NULL,
    total_price DOUBLE NOT NULL,
    order_date BIGINT NOT NULL
);

-- 创建用户表的邮箱索引
CREATE INDEX idx_users_email ON users (email);

-- 创建产品表的价格索引
CREATE INDEX idx_products_price ON products (price);

-- 创建订单表的用户ID索引
CREATE INDEX idx_orders_user_id ON orders (user_id);

-- 创建订单表的产品ID索引
CREATE INDEX idx_orders_product_id ON orders (product_id);


CREATE TABLE test_types (
    id INTEGER PRIMARY KEY,
    timestamp_col TIMESTAMP,
    timestamptz_col TIMESTAMPTZ,
    timestamp_with_tz TIMESTAMP WITH TIME ZONE,
    smallint_col SMALLINT,
    tinyint_col TINYINT,
    mediumint_col MEDIUMINT,
    bigint_col BIGINT,
    decimal_col DECIMAL(10,2),
    numeric_col NUMERIC(8,3),
    real_col REAL,
    double_col DOUBLE PRECISION,
    text_col TEXT,
    varchar_col VARCHAR(255),
    boolean_col BOOLEAN,
    date_col DATE,
    datetime_col DATETIME,
    interval_col INTERVAL
);
