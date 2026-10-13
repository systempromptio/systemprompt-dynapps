# Handoffs from agent S

1. services/ai/gateway.yaml ~lines 47-66 (`by_scope.chains`): the projects
   `commerce`, `core`, `storefront`, `business-implementation` no longer exist in
   services/web/config/groups.yaml. The only project is now `odoo-delivery`.
   Replace the four chain entries with one `odoo-delivery:` chain (same
   provider/fallbacks), or drop `by_scope` entirely.
2. tests/contract/admin/src/adfs_auth_contract.rs:30-35 reads
   services/web/config/adfs-federation-metadata.xml, which is not in the tree.
   That test needs its own fixture XML (e.g. under tests/fixtures/) instead of
   the services file.
3. extensions/web/shared/admin-core/src/repositories/sync/inventory/kinds.rs:170
   lists "web/config/adfs-federation-metadata.xml"; rename to
   "web/config/entra-federation-metadata.xml" to match adfs.yaml.
4. services/content/documentation/adfs-sso-setup.md (W): metadata filename is
   now entra-federation-metadata.xml; IdP is Microsoft Entra ID.
5. tests/unit/web/src/salesforce_orgs.rs:22 uses astound.my.salesforce.com as
   fixture data; swap to example.my.salesforce.com if the zero-astound sweep
   covers tests.
