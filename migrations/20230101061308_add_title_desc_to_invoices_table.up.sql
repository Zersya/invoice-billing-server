-- sql add column title and desc to invoices table
ALTER TABLE invoices ADD COLUMN title VARCHAR(255);
ALTER TABLE invoices ADD COLUMN description VARCHAR(255);