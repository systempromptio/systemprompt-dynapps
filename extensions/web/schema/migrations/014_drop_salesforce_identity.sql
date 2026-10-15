-- Drop the Salesforce identity table.
--
-- salesforce_user_identities mapped a user to the Salesforce Username the
-- JWT-bearer grant signed for. The Salesforce connector, its org registry and
-- the linked-account authorization band are gone, so nothing reads or writes
-- the table. schema/21_salesforce_identity.sql is deleted with this migration.
DROP TABLE IF EXISTS salesforce_user_identities CASCADE;
