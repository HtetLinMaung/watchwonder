

-- Users Table
CREATE TABLE users
(
    user_id SERIAL PRIMARY KEY,
    name varchar(255) not null,
    username VARCHAR(100) NOT NULL,
    password TEXT NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    email VARCHAR(255),
    phone VARCHAR(15),
    profile_image VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE shops
(
    shop_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    cover_image VARCHAR(255),
    address TEXT,
    city VARCHAR(100),
    state VARCHAR(100),
    postal_code VARCHAR(20),
    country VARCHAR(100),
    phone VARCHAR(20),
    email VARCHAR(255),
    website_url VARCHAR(255),
    operating_hours TEXT,
    status VARCHAR(50) DEFAULT 'Active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE categories
(
    category_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    cover_image VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE brands
(
    brand_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    logo_url VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE product_images
(
    image_id SERIAL PRIMARY KEY,
    product_id INT REFERENCES products(product_id),
    image_url VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);


CREATE TABLE products
(
    product_id SERIAL PRIMARY KEY,
    shop_id INT REFERENCES shops(shop_id),
    category_id INT REFERENCES categories(category_id),
    brand_id INT REFERENCES brands(brand_id),
    model VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(50),
    strap_material VARCHAR(50),
    strap_color VARCHAR(50),
    case_material VARCHAR(50),
    dial_color VARCHAR(50),
    movement_type VARCHAR(50),
    water_resistance VARCHAR(50),
    warranty_period VARCHAR(50),
    dimensions VARCHAR(50),
    price DECIMAL(10, 2) NOT NULL,
    stock_quantity INT DEFAULT 0,
    is_top_model BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

-- strap_material: Material of the watch strap (e.g., Leather, Stainless Steel, Rubber).
-- strap_color: Color of the strap.
-- case_material: Material of the watch case (e.g., Stainless Steel, Titanium, Ceramic).
-- dial_color: Color of the watch dial.
-- movement_type: Type of watch movement (e.g., Automatic, Quartz, Manual).
-- water_resistance: Water resistance level of the watch (e.g., 30m, 100m).
-- warranty_period: Warranty period offered with the watch (e.g., 1 Year, 2 Years).
-- dimensions: Dimensions of the watch (e.g., Case Diameter, Thickness).
-- stock_quantity: Quantity of the watch available in stock.
-- image_url: URL or path to the image of the watch.

CREATE TABLE orders
(
    order_id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(user_id),
    shipping_address_id INT REFERENCES addresses(address_id),
    status VARCHAR(50) DEFAULT 'Pending',
    order_total DECIMAL(10, 2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE addresses
(
    address_id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(user_id),
    street_address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE order_items
(
    order_item_id SERIAL PRIMARY KEY,
    order_id INT REFERENCES orders(order_id),
    product_id INT REFERENCES products(product_id),
    quantity INT DEFAULT 1,
    price DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);



CREATE TABLE notifications
(
    notification_id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id),
    message TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'Unread',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE INDEX idx_user_addresses ON addresses(user_id);

CREATE TABLE user_activity
(
    activity_id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(user_id),
    product_id INT REFERENCES products(product_id),
    activity_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE recommended_products
(
    recommended_id SERIAL PRIMARY KEY,
    product_id INT REFERENCES products(product_id),
    recommended_product_id INT REFERENCES products(product_id),
    deleted_at TIMESTAMP DEFAULT null
);
