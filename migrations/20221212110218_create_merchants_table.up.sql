CREATE TABLE merchants (
    id uuid DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    user_id uuid NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);