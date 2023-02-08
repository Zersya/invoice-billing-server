-- Add up migration script here
ALTER TABLE customer_contact_channels ADD COLUMN additional_value VARCHAR(255);
