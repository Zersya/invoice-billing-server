CREATE TABLE customer_contact_channels (
    id uuid DEFAULT uuid_generate_v4(),
    customer_id uuid NOT NULL,
    contact_channel_id uuid NOT NULL,
    value VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_channel_id) REFERENCES contact_channels(id) ON DELETE CASCADE
);