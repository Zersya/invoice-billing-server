-- Add up migration script here
ALTER TABLE merchants ADD COLUMN merchant_code VARCHAR(255) UNIQUE; 
