-- A session-owner binding exists only to pin ingestion rows to a live user;
-- once the user is deleted it protects nothing, and without CASCADE it made
-- `admin users delete` fail on every user who ever reported a hook.
ALTER TABLE ingestion_session_owners
    DROP CONSTRAINT IF EXISTS ingestion_session_owners_user_id_fkey,
    ADD CONSTRAINT ingestion_session_owners_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
