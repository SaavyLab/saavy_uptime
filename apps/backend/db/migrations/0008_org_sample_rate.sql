-- Migration number: 0008 	 2025-11-19T21:44:14.885Z

PRAGMA defer_foreign_keys = true;

ALTER TABLE organizations
ADD COLUMN ae_sample_rate REAL NOT NULL DEFAULT 1.0;

UPDATE organizations SET ae_sample_rate = 1.0 WHERE ae_sample_rate IS NULL;
