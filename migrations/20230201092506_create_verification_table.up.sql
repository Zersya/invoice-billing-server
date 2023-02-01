CREATE TABLE verifications (
    id uuid DEFAULT uuid_generate_v4(),
    user_id uuid,
    customer_id uuid,
    code VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

-- Add up migration script here
ALTER TABLE users ADD COLUMN verified_at TIMESTAMP;
ALTER TABLE customers ADD COLUMN verified_at TIMESTAMP;