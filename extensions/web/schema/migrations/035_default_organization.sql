-- The organization a user joins when nothing else claims them.
--
-- Before this column an SSO arrival whose email domain matched no organization
-- landed unattached: no `organization_members` row, and therefore no plan
-- grants, no seat, and — because every analytics query scopes through that
-- table — a dashboard of zeros. "Unattached" was never a state anyone chose;
-- it was the absence of a decision. This names the fallback so the decision is
-- always made.
--
-- Partial unique index rather than a CHECK: exactly one default, enforced the
-- same way `idx_organizations_platform` enforces exactly one platform tenant.
ALTER TABLE organizations
    ADD COLUMN IF NOT EXISTS is_default BOOLEAN NOT NULL DEFAULT FALSE;

CREATE UNIQUE INDEX IF NOT EXISTS idx_organizations_default
    ON organizations (is_default) WHERE is_default;
