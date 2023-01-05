-- Add up migration script here
ALTER TABLE customers ADD COLUMN tags TEXT[] NOT NULL DEFAULT '{}';