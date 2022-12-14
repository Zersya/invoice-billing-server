CREATE TABLE invoices (
    id uuid DEFAULT uuid_generate_v4(),
    merchant_id uuid NOT NULL,
    customer_id uuid NOT NULL,
    amount INTEGER NOT NULL,
    total_amount INTEGER NOT NULL,
    tax_amount INTEGER NOT NULL,
    tax_rate INTEGER NOT NULL,
    invoice_date TIMESTAMP NOT NULL,
    created_by uuid NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (merchant_id) REFERENCES merchants(id) ON DELETE CASCADE,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);