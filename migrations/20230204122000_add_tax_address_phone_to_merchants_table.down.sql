-- Add down migration script here
ALTER TABLE merchants DROP COLUMN address;
ALTER TABLE merchants DROP COLUMN phone_country_code;
ALTER TABLE merchants DROP COLUMN phone_number;
ALTER TABLE merchants DROP COLUMN tax;