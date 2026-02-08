-- remdb数据库模式定义

-- 创建用户表
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name varchar(16) UNIQUE NOT NULL,
    email char(32) NOT NULL,
    age INT default 20,
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
-- CREATE INDEX idx_orders_product_id ON orders (product_id);


insert into users (name, email, age, created_at) values ('bob1', 'a9', 1, 111111);
insert into users (name, email, age, created_at) values ('bob2', 'a8', 2, 111112);
insert into users (name, email, age, created_at) values ('bob3', 'a0', 3, 111113);
insert into users (name, email, age, created_at) values ('bob4', 'a', 1, 111211);
insert into users (name, email, age, created_at) values ('bob5', 'ba', 4, 111114);
insert into users (name, email, age, created_at) values ('bob6', 'ac', 1, 111111);
insert into users (name, email, age, created_at) values ('bob7', 'ab', 2, 111112);
insert into users (name, email, age, created_at) values ('bob8', 'ar', 3, 111113);
insert into users (name, email, age, created_at) values ('bob9', 'at', 1, 111211);
insert into users (name, email, age, created_at) values ('bob10', 'a1', 4, 111114);
insert into users (name, email, age, created_at) values ('bob11', 'a2', 4, 111114);
insert into users (name, email, age, created_at) values ('bob12', 'a3', 4, 111114);
insert into users (name, email, age, created_at) values ('bob13', 'a4', 4, 111114);
insert into users (name, email, age, created_at) values ('bob14', 'a5', 4, 111114);
insert into users (name, email, age, created_at) values ('bob15', 'a6', 4, 111114);

CREATE TABLE products_vec (    id INT32 PRIMARY KEY,    name TEXT,    embedding VECTOR(4) WITH DISTANCE=IP);
CREATE INDEX idx_products_embedding ON products_vec (embedding) USING HNSW WITH (M=16, ef_construction=200);
INSERT INTO products_vec (id, name, embedding) VALUES         (1, 'product1', '[0.1, 0.2, 0.3, 0.4]'),        (2, 'product2', '[1.0, 0.9, 0.8, 0.7]');

CREATE TABLE datatype_test (id INTEGER PRIMARY KEY AUTO_INCREMENT, bool_col BOOLEAN NOT NULL DEFAULT TRUE, char_col CHAR(10), varchar_col VARCHAR(50), text_col TEXT, int_col INTEGER, real_col REAL, double_col DOUBLE, timestamp_col TIMESTAMP, timestamptz_col TIMESTAMPTZ(6), json_col JSON);

INSERT INTO datatype_test (bool_col, char_col, varchar_col, text_col, int_col, real_col, double_col, timestamp_col, timestamptz_col, json_col) VALUES
(TRUE, 'hello', 'varchar1', 'This is a text column', 100, 3.14, 2.71828, '2024-01-01 12:00:00', '2024-01-01 12:00:00+00', '{"name": "Alice", "age": 30, "active": true}'),
(FALSE, 'world', 'varchar2', 'Another text value', 200, 6.28, 1.41421, '2024-02-01 13:30:00', '2024-02-01 13:30:00+00', '{"name": "Bob", "age": 25, "active": false, "tags": ["user", "admin"]}'),
(TRUE, 'test', 'varchar3', 'Third text record', 300, 9.99, 3.14159, '2024-03-01 14:45:00', '2024-03-01 14:45:00+00', '{"product": "item1", "price": 19.99, "stock": 100}');
