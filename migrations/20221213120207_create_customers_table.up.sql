CREATE TABLE customers (
    id uuid DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    merchant_id uuid NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (merchant_id) REFERENCES merchants(id) ON DELETE CASCADE
);