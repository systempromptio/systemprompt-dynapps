# Platform data console

Platform administrators can open **Platform → Logs** and **Platform → SQL**.
The pages read PostgreSQL data on demand and never poll in the background.

Logs show persisted application/server events. The default window is one hour;
filters support up to seven days, level, module, context, session and trace.
Pages contain at most 50 rows and use timestamp/ID cursors. Refresh advances the
window; paging keeps its upper bound fixed. Container stdout and reverse-proxy
logs are not ingested by this page.

SQL lists all application tables, including identity, configuration and audit
records. Primary-key cursors support composite keys. Tables without a primary
key expose only a bounded first page. Values and row details have display
limits. The query editor accepts one SELECT or read-only WITH query, runs it in
a PostgreSQL read-only transaction, and caps results at 200 rows, 1 MiB and
three seconds.

Clearing supports matching rows or an entire selected table. The preview shows
foreign-key-related deletions and SET NULL/DEFAULT updates, plus active database
triggers. Confirmation requires the exact table name and explicit acknowledgement.
Tickets are administrator-bound, single-use and expire after ten minutes.
Confirmation locks affected tables and rechecks the preview before deleting.
Changed data, constraints, lock failures and timeouts prevent a commit. A
receipt is saved atomically with the deletion, even when clearing receipt data.

Database constraints and triggers remain enabled. Trigger effects beyond
foreign keys are disclosed but are not counted in the preview. Non-deferrable
dependency cycles and SET NULL/DEFAULT operations that modify another referenced
key are refused; dependent tables must be handled first. Large selections may
exceed the five-second preview or ten-second clearing limits and require smaller
filtered selections. The console does not bypass these protections.

The transcript reader parses stored tool arguments individually. Malformed
arguments preserve their raw text and parse diagnostic in the transcript and
exports, allowing the remaining conversation to be read.

Migration 112 records its index cost from an isolated PostgreSQL 18 benchmark
with one million synthetic log rows. Timestamp/ID indexing took 4.157 seconds;
level/timestamp/ID took 2.199 seconds. The reproducible fixture is
`tests/fixtures/schema/platform-data-console-cost.sql`. Actual deployment time
depends on retained rows and database storage.
