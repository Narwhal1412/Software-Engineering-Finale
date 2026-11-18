-- Test database
CREATE TABLE IF NOT EXISTS products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price INT NOT NULL,
    description TEXT NOT NULL
);

INSERT INTO products (name, price, description) VALUES 
('Testnew', 10000000, 'Initial test product from init.sql');
