DROP TABLE IF EXISTS verifications;

ALTER TABLE users DROP COLUMN verified_at;
ALTER TABLE customers DROP COLUMN verified_at;