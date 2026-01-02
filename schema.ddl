-- remdb数据库模式定义

CREATE TABLE iotdevices (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,  
    device_id TEXT,  
    created_at BIGINT,  
    temperature DOUBLE,  
    humidity DOUBLE,  
    pressure DOUBLE,  
    battery_level INT
);


CREATE INDEX idx_iot_time ON iotdevices (created_at);


-- 创建用户表
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    age INT,
    created_at BIGINT NOT NULL
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
