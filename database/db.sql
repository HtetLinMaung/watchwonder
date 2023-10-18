

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

CREATE TABLE user_addresses
(
    address_id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(user_id),
    home_address TEXT,
    street_address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(100) NOT NULL,
    township VARCHAR(100),
    ward VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

-- alter table user_addresses
-- add constraint user_id_deleted_at_unique unique (user_id, deleted_at);
CREATE UNIQUE INDEX idx_unique_user_not_deleted 
ON user_addresses(user_id) 
WHERE deleted_at IS NULL;



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

-- Active: The shop is currently operational and open for business.
-- Inactive: The shop is registered but not currently operational.
-- Closed: The shop has permanently closed down.
-- Suspended: The shop's operations are temporarily halted, possibly due to some regulatory or administrative reasons.
-- Pending Approval: The shop's details are under review, and it hasn't been approved to start operations yet.

INSERT INTO shops
    (
    name,
    description,
    cover_image,
    address,
    city,
    state,
    postal_code,
    country,
    phone,
    email,
    website_url,
    operating_hours,
    status
    )
VALUES
    ('Timeless Watches',
        'A shop offering a variety of timeless and classic watch designs.',
        '/images/timeless_cover.jpg',
        '123 Watch Street',
        'Watch City',
        'Watch State',
        '12345',
        'Watchland',
        '+1234567890',
        'info@timelesswatches.com',
        'http://www.timelesswatches.com',
        'Mon-Fri: 9am-6pm; Sat: 10am-4pm',
        'Active'),

    ('Modern Timepieces',
        'Specializing in modern and innovative watch designs for the contemporary individual.',
        '/images/modern_cover.jpg',
        '456 Modern Avenue',
        'Timepiece City',
        'Modern State',
        '67890',
        'Timepiece Country',
        '+0987654321',
        'info@moderntimepieces.com',
        'http://www.moderntimepieces.com',
        'Mon-Sun: 10am-8pm',
        'Active'),

    ('Vintage Horology',
        'A boutique shop offering a curated selection of vintage and antique watches.',
        '/images/vintage_cover.jpg',
        '789 Antique Road',
        'Vintage Town',
        'Horology State',
        '11223',
        'Horology Country',
        '+1122334455',
        'info@vintagehorology.com',
        'http://www.vintagehorology.com',
        'Tue-Sat: 11am-5pm',
        'Active');


CREATE TABLE categories
(
    category_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    cover_image VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

INSERT INTO categories
    (
    name,
    description,
    cover_image
    )
VALUES
    ('Luxury Watches',
        'High-end watches from renowned brands, showcasing craftsmanship and elegance.',
        '/images/luxury_category.jpg'),

    ('Sports Watches',
        'Durable and functional watches designed for active individuals and sports enthusiasts.',
        '/images/sports_category.jpg'),

    ('Vintage Watches',
        'Classic timepieces from past eras, offering a nostalgic touch and timeless beauty.',
        '/images/vintage_category.jpg'),

    ('Smart Watches',
        'Modern watches integrated with technology to offer features beyond just timekeeping.',
        '/images/smart_category.jpg'),

    ('Casual Watches',
        'Everyday watches that combine style and practicality for daily wear.',
        '/images/casual_category.jpg');


CREATE TABLE brands
(
    brand_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    logo_url VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

INSERT INTO brands
    (
    name,
    description,
    logo_url
    )
VALUES
    ('Rolex',
        'A Swiss luxury watch manufacturer known for its timeless designs and precision.',
        '/logos/rolex_logo.jpg'),

    ('Omega',
        'A Swiss luxury watchmaker, renowned for its performance and reliability.',
        '/logos/omega_logo.jpg'),

    ('Casio',
        'A Japanese multinational consumer electronics company, famous for its durable and innovative watches.',
        '/logos/casio_logo.jpg'),

    ('Seiko',
        'A Japanese company that produces watches, clocks, electronic devices, semiconductors, and optical products.',
        '/logos/seiko_logo.jpg'),

    ('TAG Heuer',
        'A Swiss luxury watchmaker known for its sports watches and chronographs.',
        '/logos/tagheuer_logo.jpg'),

    ('Fossil',
        'An American fashion designer and manufacturer known for its vintage-inspired watches.',
        '/logos/fossil_logo.jpg');



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

-- Inserting sample data into products table
INSERT INTO products
    (
    shop_id,
    category_id,
    brand_id,
    model,
    description,
    color,
    strap_material,
    strap_color,
    case_material,
    dial_color,
    movement_type,
    water_resistance,
    warranty_period,
    dimensions,
    price,
    stock_quantity,
    is_top_model
    )
VALUES
    (1, 1, 1, 'Submariner', 'Dive watch with automatic movement and date function.', 'Black', 'Steel', 'Silver', 'Steel', 'Black', 'Automatic', '300m', '5 years', '40mm', 9000.00, 10, TRUE),
    (2, 2, 2, 'Speedmaster', 'Chronograph watch used in space missions.', 'Black', 'Leather', 'Black', 'Steel', 'Black', 'Manual', '50m', '3 years', '42mm', 5000.00, 5, TRUE);

-- Inserting sample data into product_images table
INSERT INTO product_images
    (
    product_id,
    image_url
    )
VALUES
    (1, '/images/products/submariner_1.jpg'),
    (1, '/images/products/submariner_2.jpg'),
    (1, '/images/products/submariner_3.jpg'),
    (2, '/images/products/speedmaster_1.jpg'),
    (2, '/images/products/speedmaster_2.jpg');


CREATE TABLE orders
(
    order_id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(user_id),
    shipping_address_id INT REFERENCES order_addresses(address_id),
    status VARCHAR(50) DEFAULT 'Pending',
    order_total DECIMAL(10, 2) DEFAULT 0.0,
    item_counts INT DEFAULT 0,
    payment_type VARCHAR(50) DEFAULT 'Cash on Delivery',
    payslip_screenshot_path VARCHAR(255) DEFAULT '',
    commission_amount DECIMAL(10, 2) DEFAULT 0.0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);
-- Preorder || Cash on Delivery

-- Pending: The order has been placed but not yet processed.
-- Processing: The order is currently being prepared or packaged.
-- Shipped: The order has been dispatched and is on its way to the customer.
-- Delivered: The order has been delivered to the customer.
-- Completed: The order has been received by the customer and is considered complete.
-- Cancelled: The order was cancelled by the customer or the seller.
-- Refunded: The order has been refunded to the customer.
-- Failed: There was an issue processing the order, and it did not go through.
-- On Hold: The order is temporarily on hold, possibly due to payment issues or stock availability.
-- Backordered: Some or all of the items in the order are not currently in stock and will be shipped when available.
-- Returned: The customer has returned the order, and it's being processed for a refund or exchange.

CREATE TABLE order_addresses
(
    address_id SERIAL PRIMARY KEY,
    home_address TEXT,
    street_address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(100) NOT NULL,
    township VARCHAR(100),
    ward VARCHAR(100),
    note TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
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
    user_id INT REFERENCES users(user_id),
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'Unread',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);
-- Unread: The user hasn't seen or interacted with the notification yet.
-- Read: The user has seen the notification but hasn't taken any action on it.
-- Acted: The user has taken some action on the notification, such as clicking on a link or button associated with it.
-- Dismissed: The user has chosen to dismiss or ignore the notification without taking any further action.
-- Archived: The user has chosen to archive the notification for future reference.

CREATE TABLE fcm_tokens
(
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(user_id),
    token VARCHAR(255) NOT NULL UNIQUE,
    device_type VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_unique_user_device_type 
ON fcm_tokens(user_id, device_type);
-- e.g., 'android', 'ios', 'web'

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

CREATE TABLE terms_and_conditions
(
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE commission_rules
(
    rule_id SERIAL PRIMARY KEY,
    description TEXT,
    commission_percentage DECIMAL(5, 2) NOT NULL,
    min_order_amount DECIMAL(10, 2) NOT NULL,
    max_order_amount DECIMAL(10, 2) NOT NULL,
    effective_from TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    effective_to TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

INSERT INTO commission_rules
    (description, commission_percentage, min_order_amount, max_order_amount, effective_from, effective_to, created_at, updated_at)
VALUES
    ('Standard Insurance for Watches', 5.00, 100.00, 10000.00, '2023-01-01 00:00:00', '2033-01-01 00:00:00', '2023-01-01 00:00:00', '2023-01-01 00:00:00'),
    ('Premium Insurance for Luxury Watches', 7.00, 10001.00, 50000.00, '2023-01-01 00:00:00', '2033-01-01 00:00:00', '2023-01-01 00:00:00', '2023-01-01 00:00:00'),
    ('Basic Insurance for Affordable Watches', 3.00, 1.00, 99.99, '2023-01-01 00:00:00', '2033-01-01 00:00:00', '2023-01-01 00:00:00', '2023-01-01 00:00:00');


CREATE TABLE insurance_options
(
    option_id SERIAL PRIMARY KEY,
    order_id INT REFERENCES orders(order_id),
    rule_id INT REFERENCES commission_rules(rule_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

