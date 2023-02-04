-- Add up migration script here
ALTER TABLE merchants ADD COLUMN address VARCHAR(255);
ALTER TABLE merchants ADD COLUMN phone_country_code VARCHAR(100);
ALTER TABLE merchants ADD COLUMN phone_number VARCHAR(255);
ALTER TABLE merchants ADD COLUMN tax DECIMAL;