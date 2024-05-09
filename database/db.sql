

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
    account_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    can_modify_order_status BOOLEAN DEFAULT FALSE,
    can_view_address BOOLEAN DEFAULT FALSE,
    can_view_phone BOOLEAN DEFAULT FALSE,
    google_id VARCHAR(255),
    facebook_id VARCHAR(255),
    apple_id VARCHAR(255),
    last_active_at TIMESTAMP,
    is_online BOOLEAN DEFAULT FALSE,
    request_to_agent BOOLEAN DEFAULT FALSE,
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
    creator_id INT REFERENCES users(user_id),
    is_demo BOOLEAN DEFAULT FALSE,
    level INT DEFAULT 0,
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
    creator_id INT REFERENCES users(user_id),
    is_demo BOOLEAN DEFAULT FALSE,
    level INT DEFAULT 0,
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
    creator_id INT REFERENCES users(user_id),
    is_demo BOOLEAN DEFAULT FALSE,
    level INT DEFAULT 0,
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
    warranty_type_id INT REFERENCES warranty_types(warranty_type_id) DEFAULT 1,
    dial_glass_type_id INT REFERENCES dial_glass_types(dial_glass_type_id) DEFAULT 1,
    other_accessories_type_id INT REFERENCES other_accessories_types(other_accessories_type_id) DEFAULT 1,
    gender_id INT REFERENCES genders(gender_id) DEFAULT 1,
    waiting_time VARCHAR(50) DEFAULT '',
    dimensions VARCHAR(50),
    case_diameter VARCHAR(50) default '',
    case_depth VARCHAR(50) default '',
    case_width VARCHAR(50) default '',
    price DECIMAL(10, 2) NOT NULL,
    stock_quantity INT DEFAULT 0,
    condition VARCHAR(255) DEFAULT '',
    movement_caliber VARCHAR(255) DEFAULT '',
    movement_country VARCHAR(255) DEFAULT '',
    is_top_model BOOLEAN DEFAULT FALSE,
    is_preorder BOOLEAN DEFAULT FALSE,
    creator_id INT REFERENCES users
    (user_id),
    currency_id INT REFERENCES currencies
    (currency_id) DEFAULT 1,
    is_demo BOOLEAN DEFAULT FALSE,
    discount_percent DECIMAL(10, 2) DEFAULT 0,
    discount_expiration TIMESTAMP DEFAULT null,
    discount_reason TEXT DEFAULT '',
    discounted_price DECIMAL(18, 2) DEFAULT 0.0,
    discount_type VARCHAR(255) DEFAULT 'Discount by Specific Percentage',
    coupon_code VARCHAR(255) DEFAULT NULL,
    discount_updated_by VARCHAR(255) DEFAULT 'product',
    level INT DEFAULT 0,
    is_auction_product BOOLEAN DEFAULT FALSE,
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
    currency_id INT REFERENCES currencies(currency_id) DEFAULT 1,
    invoice_id VARCHAR(255) DEFAULT '',
    invoice_url VARCHAR(255) DEFAULT '',
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
    currency_id INT REFERENCES currencies(currency_id) DEFAULT 1,
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


CREATE TABLE seller_reviews
(
    review_id SERIAL PRIMARY KEY,
    shop_id INT REFERENCES shops(shop_id),
    user_id INT REFERENCES users(user_id),
    rating DECIMAL(2,1) NOT NULL,
    comment TEXT,
    review_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

-- e.g., 4.5

CREATE TABLE currencies
(
    currency_id SERIAL PRIMARY KEY,
    currency_code CHAR(3) NOT NULL UNIQUE,
    currency_name VARCHAR(255) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

-- ISO 4217 currency code, e.g., USD, EUR, MMK
-- e.g., $, €, ကျပ်

INSERT INTO currencies
    (currency_code, currency_name, symbol)
VALUES
    ('USD', 'United States Dollar', '$'),
    ('SGD', 'Singapore Dollar', 'S$'),
    ('THB', 'Thai Baht', '฿'),
    ('CNY', 'China Yuan Renminbi', '¥'),
    ('MMK', 'Myanmar Kyat', 'Ks');


CREATE TABLE bank_accounts
(
    account_id SERIAL PRIMARY KEY,
    account_type VARCHAR(255) DEFAULT 'mbanking',
    account_holder_name VARCHAR(255) NOT NULL,
    account_number VARCHAR(50) NOT NULL,
    bank_logo VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into bank_accounts
    (account_holder_name, account_number, bank_logo)
values
    ('U La Min Tun', '002211188001232', '');
insert into bank_accounts
    (account_holder_name, account_number, bank_logo)
values
    ('Daw Yin Yin Myo', '27030127000219801', '');
insert into bank_accounts
    (account_holder_name, account_number, bank_logo)
values
    ('Daw Yin Yin Myo', '0014600100018578', '');

CREATE TABLE warranty_types
(
    warranty_type_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into warranty_types
    (description)
values
    ('Local Seller Warranty');
insert into warranty_types
    (description)
values
    ('International Warranty');
insert into warranty_types
    (description)
values
    ('Authorized Distributor Warranty');

CREATE TABLE buyer_protections
(
    buyer_protection_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into buyer_protections
    (description)
values
    ('Authenticity Guarantee');
insert into buyer_protections
    (description)
values
    ('14-day money-back guarantee');


CREATE TABLE seller_informations
(
    seller_id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(user_id),
    company_name VARCHAR(255) NOT NULL,
    professional_title VARCHAR(255) NOT NULL,
    active_since_year INT NOT NULL,
    location VARCHAR(255) NOT NULL,
    offline_trader BOOLEAN DEFAULT FALSE,

    facebook_profile_image VARCHAR(255) DEFAULT '',
    shop_or_page_name VARCHAR(255) DEFAULT '',
    facebook_page_image VARCHAR(255) DEFAULT '',
    bussiness_phone VARCHAR(15) DEFAULT '',
    address TEXT DEFAULT '',
    nrc VARCHAR(255) DEFAULT '',
    nrc_front_image VARCHAR(255) DEFAULT '',
    nrc_back_image VARCHAR(255) DEFAULT '',
    passport_image VARCHAR(255) DEFAULT '',
    driving_licence_image VARCHAR(255) DEFAULT '',
    signature_image VARCHAR(255) DEFAULT '',
    bank_code VARCHAR(15) DEFAULT '',
    bank_account VARCHAR(255) DEFAULT '',
    bank_account_image VARCHAR(255) DEFAULT '',
    wallet_type VARCHAR(15) DEFAULT '',
    wallet_account VARCHAR(255) DEFAULT '',
    fee_id INT REFERENCES seller_registration_fees(fee_id) DEFAULT 1,
    monthly_transaction_screenshot VARCHAR(255) DEFAULT '',

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE dial_glass_types
(
    dial_glass_type_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into dial_glass_types
    (description)
values
    ('Sapphire Crystal');
insert into dial_glass_types
    (description)
values
    ('Hardlex Crystal');
insert into dial_glass_types
    (description)
values
    ('Mineral Glass');
insert into dial_glass_types
    (description)
values
    ('Acrylic Crystal');

CREATE TABLE conditions
(
    condition_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into conditions
    (description)
values
    ('Brand New');
insert into conditions
    (description)
values
    ('Unworn');
insert into conditions
    (description)
values
    ('Very Good');
insert into conditions
    (description)
values
    ('Good');
insert into conditions
    (description)
values
    ('Fair');
insert into conditions
    (description)
values
    ('Poor');
insert into conditions
    (description)
values
    ('Incomplete');

CREATE TABLE other_accessories_types
(
    other_accessories_type_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into other_accessories_types
    (description)
values
    ('Original box, warranty card, manual book');
insert into other_accessories_types
    (description)
values
    ('No original box, warranty card, manual book');
insert into other_accessories_types
    (description)
values
    ('Only original box');
insert into other_accessories_types
    (description)
values
    ('Watch only');

CREATE TABLE genders
(
    gender_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into genders
    (description)
values
    ('Men');
insert into genders
    (description)
values
    ('Women');
insert into genders
    (description)
values
    ('Unisex');

CREATE TABLE chats
(
    chat_id SERIAL PRIMARY KEY,
    is_group BOOLEAN DEFAULT FALSE,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE chat_deletes
(
    chat_delete_id SERIAL PRIMARY KEY,
    chat_id INT REFERENCES chats(chat_id),
    user_id INT REFERENCES users(user_id),
    deleted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE chat_participants
(
    chat_id INT REFERENCES chats(chat_id),
    user_id INT REFERENCES users(user_id),
    PRIMARY KEY (chat_id, user_id)
);

CREATE TABLE audio_messages
(
    audio_message_id SERIAL PRIMARY KEY,
    audio_url VARCHAR(255) NOT NULL,
    duration INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);


CREATE TABLE messages
(
    message_id SERIAL PRIMARY KEY,
    chat_id INT REFERENCES chats(chat_id),
    sender_id INT REFERENCES users(user_id),
    message_text TEXT,
    audio_message_id INT REFERENCES audio_messages(audio_message_id),
    status VARCHAR(50) DEFAULT 'sent',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

-- Sent: The message has been sent from the user’s device and received by the server, but it has not yet been delivered to the recipient's device.

-- Delivered: The message has been sent from the user’s device, received by the server, and successfully delivered to the recipient’s device. However, it has not been read yet.

-- Read: The message has been read by the recipient. This status implies that the message was also delivered.

-- Failed: The message could not be sent. This status could be due to a variety of reasons, such as network issues, server problems, or other technical issues.

-- Pending: The message is in the process of being sent but has not been fully transmitted to the server yet. This status could be used when there is a network delay or other issue preventing immediate sending.

-- Deleted: The message has been deleted by the sender or recipient. The application might implement this as a "soft delete," where the message is marked as deleted but not actually removed from the database.

-- Edited: The message has been edited after it was sent. This status could be used in conjunction with a timestamp to indicate when the message was last edited.

-- Archived: The message has been archived, meaning it is no longer active in the chat but is retained for historical purposes.

-- Flagged: The message has been flagged, possibly by a user or an automated system, for review due to potentially inappropriate content or behavior.

-- Recalled/Revoked: The message has been recalled or revoked by the sender after it was sent. Depending on the application’s functionality, this might make the message disappear from the recipient’s view or be replaced with a notice that the message was recalled.

CREATE TABLE message_images
(
    message_image_id SERIAL PRIMARY KEY,
    message_id INT REFERENCES messages(message_id),
    image_url VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE movement_types
(
    movement_type_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
INSERT INTO movement_types
    (description)
VALUES
    ('Automatic with manual winding');
INSERT INTO movement_types
    (description)
VALUES
    ('Automatic with self-winding');
INSERT INTO movement_types
    (description)
VALUES
    ('Digital');
INSERT INTO movement_types
    (description)
VALUES
    ('Mechanical');
INSERT INTO movement_types
    (description)
VALUES
    ('Japanese Quartz');
INSERT INTO movement_types
    (description)
VALUES
    ('Chinese Quartz');
INSERT INTO movement_types
    (description)
VALUES
    ('German Quartz');
INSERT INTO movement_types
    (description)
VALUES
    ('Russian Quartz');
INSERT INTO movement_types
    (description)
VALUES
    ('American Quartz');
INSERT INTO movement_types
    (description)
VALUES
    ('Swiss Quartz');
INSERT INTO movement_types
    (description)
VALUES
    ('Smart');
INSERT INTO movement_types
    (description)
VALUES
    ('Chronograph');
INSERT INTO movement_types
    (description)
VALUES
    ('Solar');
INSERT INTO movement_types
    (description)
VALUES
    ('Analog');
INSERT INTO movement_types
    (description)
VALUES
    ('Kinetic');

CREATE TABLE strap_materials
(
    strap_material_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
INSERT INTO strap_materials
    (description)
VALUES
    ('Stainless steel');
INSERT INTO strap_materials
    (description)
VALUES
    ('Steel');
INSERT INTO strap_materials
    (description)
VALUES
    ('Brass');
INSERT INTO strap_materials
    (description)
VALUES
    ('Artificial leather');
INSERT INTO strap_materials
    (description)
VALUES
    ('Alloy');
INSERT INTO strap_materials
    (description)
VALUES
    ('Titanium');
INSERT INTO strap_materials
    (description)
VALUES
    ('Leather');
INSERT INTO strap_materials
    (description)
VALUES
    ('Metal');
INSERT INTO strap_materials
    (description)
VALUES
    ('Fabric');
INSERT INTO strap_materials
    (description)
VALUES
    ('Plastic');
INSERT INTO strap_materials
    (description)
VALUES
    ('Rubber');
INSERT INTO strap_materials
    (description)
VALUES
    ('Wood');
INSERT INTO strap_materials
    (description)
VALUES
    ('Ceramic');
INSERT INTO strap_materials
    (description)
VALUES
    ('Nylon');
INSERT INTO strap_materials
    (description)
VALUES
    ('Silicone');
-- Corrected from 'Silicon'
INSERT INTO strap_materials
    (description)
VALUES
    ('Carbon fiber');

CREATE TABLE case_materials
(
    case_material_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
insert into case_materials
    (description)
values
    ('Stainless steel');
insert into case_materials
    (description)
values
    ('Steel');
insert into case_materials
    (description)
values
    ('Titanium');
insert into case_materials
    (description)
values
    ('Carbon');
insert into case_materials
    (description)
values
    ('Wood');
insert into case_materials
    (description)
values
    ('Alloy');
insert into case_materials
    (description)
values
    ('Plastic');
insert into case_materials
    (description)
values
    ('Bronze');
insert into case_materials
    (description)
values
    ('Pure gold');
insert into case_materials
    (description)
values
    ('Ceramic');
insert into case_materials
    (description)
values
    ('Platinum');
insert into case_materials
    (description)
values
    ('Diamond');

CREATE TABLE water_resistances
(
    water_resistance_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
INSERT INTO water_resistances
    (description)
VALUES
    ('3 bar');
INSERT INTO water_resistances
    (description)
VALUES
    ('5 bar');
INSERT INTO water_resistances
    (description)
VALUES
    ('10 bar');
INSERT INTO water_resistances
    (description)
VALUES
    ('30 bar');
INSERT INTO water_resistances
    (description)
VALUES
    ('100 bar');
INSERT INTO water_resistances
    (description)
VALUES
    ('300 bar');
INSERT INTO water_resistances
    (description)
VALUES
    ('3ATM');
INSERT INTO water_resistances
    (description)
VALUES
    ('5ATM');
INSERT INTO water_resistances
    (description)
VALUES
    ('10ATM');
INSERT INTO water_resistances
    (description)
VALUES
    ('20ATM');
INSERT INTO water_resistances
    (description)
VALUES
    ('50ATM');
INSERT INTO water_resistances
    (description)
VALUES
    ('100ATM');
INSERT INTO water_resistances
    (description)
VALUES
    ('30m');
INSERT INTO water_resistances
    (description)
VALUES
    ('50m');
INSERT INTO water_resistances
    (description)
VALUES
    ('100m');
INSERT INTO water_resistances
    (description)
VALUES
    ('200m');
INSERT INTO water_resistances
    (description)
VALUES
    ('300m');
INSERT INTO water_resistances
    (description)
VALUES
    ('500m');
INSERT INTO water_resistances
    (description)
VALUES
    ('1000m');
INSERT INTO water_resistances
    (description)
VALUES
    ('2000m');


CREATE TABLE movement_countries
(
    movement_country_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
INSERT INTO movement_countries
    (description)
VALUES
    ('Switzerland');
INSERT INTO movement_countries
    (description)
VALUES
    ('Japan');
INSERT INTO movement_countries
    (description)
VALUES
    ('China');
INSERT INTO movement_countries
    (description)
VALUES
    ('USA');
INSERT INTO movement_countries
    (description)
VALUES
    ('Russia');
INSERT INTO movement_countries
    (description)
VALUES
    ('Germany');
INSERT INTO movement_countries
    (description)
VALUES
    ('Others');

CREATE TABLE report_subjects
(
    subject_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
INSERT INTO report_subjects
    (description)
VALUES
    ('Suspicious vendor');
INSERT INTO report_subjects
    (description)
VALUES
    ('Counterfeit watch');
INSERT INTO report_subjects
    (description)
VALUES
    ('Inaccurate listing');
INSERT INTO report_subjects
    (description)
VALUES
    ('This watch is unavailable');
INSERT INTO report_subjects
    (description)
VALUES
    ('The dealer hasn''t responded');
INSERT INTO report_subjects
    (description)
VALUES
    ('Other');


CREATE TABLE seller_reports
(
    report_id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(user_id),
    seller_id INT REFERENCES users(user_id),
    subject_id INT REFERENCES report_subjects(subject_id),
    message TEXT DEFAULT '',
    phone VARCHAR(15) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE payment_types
(
    payment_type_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
insert into payment_types
    (description)
values
    ('Cash on Delivery');
insert into payment_types
    (description)
values
    ('Half Prepaid');
insert into payment_types
    (description)
values
    ('Full Prepaid');


CREATE TABLE reason_types
(
    reason_type_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

INSERT INTO reason_types
    (description)
VALUES
    ('Incorrect Product Specifications');
INSERT INTO reason_types
    (description)
VALUES
    ('Product is Defective or Damaged');
INSERT INTO reason_types
    (description)
VALUES
    ('The Watch is Not Working');
INSERT INTO reason_types
    (description)
VALUES
    ('Watch Not Received');
INSERT INTO reason_types
    (description)
VALUES
    ('Quality Not as Described in App');


CREATE TABLE refund_reasons
(
    refund_reason_id SERIAL PRIMARY KEY,
    order_id INT REFERENCES orders(order_id),
    user_id INT REFERENCES users(user_id),
    reason_type_id INT REFERENCES reason_types(reason_type_id),
    comment TEXT DEFAULT '',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE counters
(
    counter_id SERIAL PRIMARY KEY,
    label VARCHAR(255) NOT NULL,
    n INT default 1
);
insert into counters
    (label)
values
    ('777');

CREATE TABLE discount_types
(
    discount_type_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

INSERT INTO discount_types
    (description)
VALUES
    ('No Discount');

INSERT INTO discount_types
    (description)
VALUES
    ('Discount by Specific Percentage');

INSERT INTO discount_types
    (description)
VALUES
    ('Discount by Specific Amount');


CREATE TABLE seller_registration_fees
(
    fee_id SERIAL PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    amount DECIMAL(18, 2) DEFAULT 0.0,
    is_percent BOOLEAN DEFAULT FALSE,
    currency_id INT REFERENCES currencies(currency_id) DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into seller_registration_fees
    (description, amount, is_percent)
values
    ('Monthly Fee', '50000.00', false);
insert into seller_registration_fees
    (description, amount, is_percent)
values
    ('Commission Fee', '5', true);

CREATE TABLE seller_agreement_contract
(
    aggrement_id SERIAL PRIMARY KEY,
    file_path VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE discount_rules
(
    rule_id SERIAL PRIMARY KEY,
    discount_for VARCHAR(255) DEFAULT 'all',
    discount_for_id INT DEFAULT 0,
    discount_percent DECIMAL(10, 2) DEFAULT 0,
    discount_expiration TIMESTAMP DEFAULT null,
    discount_reason TEXT DEFAULT '',
    discounted_price DECIMAL(18, 2) DEFAULT 0.0,
    discount_type VARCHAR(255) DEFAULT 'Discount by Specific Percentage',
    coupon_code VARCHAR(255) DEFAULT NULL,
    creator_id INT REFERENCES users(user_id),
    shop_id INT REFERENCES shops(shop_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

-- discount_for: all, product, brand, category

CREATE TABLE advertisements
(
    advertisement_id SERIAL PRIMARY KEY,
    media_type VARCHAR(50) CHECK (media_type IN ('image', 'video')),
    media_url VARCHAR(255) NOT NULL,
    level INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE auctions
(
    auction_id SERIAL PRIMARY KEY,
    product_id INTEGER REFERENCES products(product_id),
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP NOT NULL,
    start_bid DECIMAL(10, 2) DEFAULT 0.0,
    current_bid DECIMAL(10, 2) DEFAULT 0.0,
    reserve_price DECIMAL(10, 2) DEFAULT 0.0,
    buy_it_now_available BOOLEAN DEFAULT FALSE,
    status VARCHAR(50) DEFAULT 'active',
    -- e.g., active, completed, canceled
    winner_id INTEGER REFERENCES users(user_id),
    sold_out_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE bids
(
    bid_id SERIAL PRIMARY KEY,
    auction_id INTEGER REFERENCES auctions(auction_id),
    bidder_id INTEGER REFERENCES users(user_id),
    bid_amount DECIMAL(10, 2) NOT NULL,
    bid_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    payment_status VARCHAR(50) DEFAULT 'save',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);