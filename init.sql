-- Create sample tables for testing the MCP server

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    age INTEGER,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2),
    stock INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    product_id INTEGER REFERENCES products(id),
    quantity INTEGER NOT NULL,
    total_price DECIMAL(10, 2),
    order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample data
INSERT INTO users (name, email, age, active) VALUES
    ('Alice Johnson', 'alice@example.com', 28, true),
    ('Bob Smith', 'bob@example.com', 35, true),
    ('Charlie Brown', 'charlie@example.com', 42, false),
    ('Diana Prince', 'diana@example.com', 30, true),
    ('Eve Anderson', 'eve@example.com', 26, true);

INSERT INTO products (name, description, price, stock) VALUES
    ('Laptop', 'High-performance laptop for developers', 1299.99, 50),
    ('Wireless Mouse', 'Ergonomic wireless mouse', 29.99, 200),
    ('Mechanical Keyboard', 'RGB mechanical keyboard', 89.99, 100),
    ('Monitor', '27-inch 4K monitor', 499.99, 30),
    ('Webcam', '1080p HD webcam', 79.99, 75);

INSERT INTO orders (user_id, product_id, quantity, total_price) VALUES
    (1, 1, 1, 1299.99),
    (2, 2, 2, 59.98),
    (3, 3, 1, 89.99),
    (4, 4, 1, 499.99),
    (5, 5, 1, 79.99),
    (1, 2, 1, 29.99),
    (2, 3, 1, 89.99);
