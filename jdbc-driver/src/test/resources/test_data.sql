-- 测试数据文件
-- 包含用于各种测试的样本数据

-- 创建测试表
CREATE TABLE IF NOT EXISTS test_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    age INTEGER,
    active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS test_products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    price REAL NOT NULL,
    stock INTEGER DEFAULT 0,
    description TEXT,
    category TEXT
);

CREATE TABLE IF NOT EXISTS test_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    total_amount REAL NOT NULL,
    status TEXT DEFAULT 'pending',
    FOREIGN KEY (user_id) REFERENCES test_users(id)
);

CREATE TABLE IF NOT EXISTS test_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER,
    product_id INTEGER,
    quantity INTEGER NOT NULL,
    unit_price REAL NOT NULL,
    FOREIGN KEY (order_id) REFERENCES test_orders(id),
    FOREIGN KEY (product_id) REFERENCES test_products(id)
);

CREATE TABLE IF NOT EXISTS test_sensor_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sensor_id INTEGER,
    timestamp TIMESTAMP NOT NULL,
    value REAL NOT NULL,
    location TEXT
);

CREATE TABLE IF NOT EXISTS test_vectors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    vec VECTOR(3) WITH DISTANCE=L2,
    label TEXT
);

-- 插入测试数据
-- 用户数据
INSERT INTO test_users (name, email, age, active) VALUES
('Alice', 'alice@example.com', 30, TRUE),
('Bob', 'bob@example.com', 25, TRUE),
('Charlie', 'charlie@example.com', 35, FALSE),
('David', 'david@example.com', 40, TRUE),
('Eve', 'eve@example.com', 28, TRUE);

-- 产品数据
INSERT INTO test_products (name, price, stock, description, category) VALUES
('Product A', 100.0, 100, 'High quality product A', 'Electronics'),
('Product B', 200.0, 50, 'High quality product B', 'Electronics'),
('Product C', 50.0, 200, 'Affordable product C', 'Home'),
('Product D', 150.0, 75, 'Mid-range product D', 'Home'),
('Product E', 300.0, 25, 'Premium product E', 'Electronics');

-- 订单数据
INSERT INTO test_orders (user_id, order_date, total_amount, status) VALUES
(1, '2023-01-01 10:00:00', 300.0, 'completed'),
(1, '2023-02-01 11:00:00', 150.0, 'completed'),
(2, '2023-01-02 12:00:00', 200.0, 'completed'),
(3, '2023-01-03 13:00:00', 50.0, 'pending'),
(4, '2023-01-04 14:00:00', 350.0, 'completed');

-- 订单项数据
INSERT INTO test_order_items (order_id, product_id, quantity, unit_price) VALUES
(1, 1, 1, 100.0),
(1, 2, 1, 200.0),
(2, 4, 1, 150.0),
(3, 3, 1, 50.0),
(4, 2, 1, 200.0),
(4, 4, 1, 150.0);

-- 传感器数据
INSERT INTO test_sensor_data (sensor_id, timestamp, value, location) VALUES
(1, '2023-01-01 00:00:00', 25.5, 'Room 1'),
(1, '2023-01-01 01:00:00', 25.6, 'Room 1'),
(1, '2023-01-01 02:00:00', 25.7, 'Room 1'),
(2, '2023-01-01 00:00:00', 22.0, 'Room 2'),
(2, '2023-01-01 01:00:00', 22.1, 'Room 2'),
(3, '2023-01-01 00:00:00', 18.5, 'Room 3'),
(3, '2023-01-01 01:00:00', 18.6, 'Room 3'),
(3, '2023-01-01 02:00:00', 18.7, 'Room 3'),
(3, '2023-01-01 03:00:00', 18.8, 'Room 3');

-- 向量数据
INSERT INTO test_vectors (vec, label) VALUES
(VECTOR(1.0, 2.0, 3.0), 'vector1'),
(VECTOR(1.1, 2.1, 3.1), 'vector2'),
(VECTOR(4.0, 5.0, 6.0), 'vector3'),
(VECTOR(0.0, 0.0, 0.0), 'vector4'),
(VECTOR(1.0, 0.0, 0.0), 'vector5');

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_users_email ON test_users(email);
CREATE INDEX IF NOT EXISTS idx_products_category ON test_products(category);
CREATE INDEX IF NOT EXISTS idx_orders_user_id ON test_orders(user_id);
CREATE INDEX IF NOT EXISTS idx_sensor_timestamp ON test_sensor_data(timestamp);

-- 查看创建的表
SELECT name FROM sqlite_master WHERE type='table';

-- 查看表中的数据量
SELECT 'test_users' AS table_name, COUNT(*) AS row_count FROM test_users UNION ALL
SELECT 'test_products' AS table_name, COUNT(*) AS row_count FROM test_products UNION ALL
SELECT 'test_orders' AS table_name, COUNT(*) AS row_count FROM test_orders UNION ALL
SELECT 'test_order_items' AS table_name, COUNT(*) AS row_count FROM test_order_items UNION ALL
SELECT 'test_sensor_data' AS table_name, COUNT(*) AS row_count FROM test_sensor_data UNION ALL
SELECT 'test_vectors' AS table_name, COUNT(*) AS row_count FROM test_vectors;
