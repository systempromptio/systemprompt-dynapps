# Fallback image for the knowledge Postgres app — used ONLY if
# `CREATE EXTENSION vector` fails on the stock flyio/postgres-flex:17 image
# (see README.md, step 2). Same base image, plus the pgvector package, so the
# app keeps Fly's replication, backup and health-check tooling untouched.
FROM flyio/postgres-flex:17

RUN apt-get update \
    && apt-get install -y --no-install-recommends postgresql-17-pgvector \
    && rm -rf /var/lib/apt/lists/*
