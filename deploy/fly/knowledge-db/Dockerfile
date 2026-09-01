# Image for the knowledge Postgres app. The stock `flyio/postgres-flex` image
# carries no pgvector -- confirmed on 2026-09-01 against 18.1, where
# `CREATE EXTENSION vector` answers `extension "vector" is not available` -- so
# the app runs this instead: the same base image plus the pgvector package, and
# Fly's replication, backup and health-check tooling untouched.
#
# The tag MUST track the major version the cluster already runs. `flyctl
# postgres create` provisioned 18.1, and pointing this at an older major would
# start a downgraded server against an existing data directory rather than
# adding an extension. Read the running major from
# `flyctl image show -a sp-a2f658d8bc5f-knowledge` before changing it.
FROM flyio/postgres-flex:18.1

RUN apt-get update \
    && apt-get install -y --no-install-recommends postgresql-18-pgvector \
    && rm -rf /var/lib/apt/lists/*
