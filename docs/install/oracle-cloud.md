# Install on Oracle Cloud (three regions, one database)

The complete runbook is `deploy/self-host/README.md`; this page is only the
OCI mapping for it.

| runbook item | OCI resource |
|---|---|
| node | a Compute instance per region (India, US, EU), Oracle Linux or Ubuntu, Docker Engine + Compose v2. `VM.Standard.E4.Flex` 2 OCPU / 8 GB is ample per node |
| load balancer | one Load Balancer per region with a TLS listener on 443, backend set → the node on 8080, health check `GET /api/v1/health` with a **body match on `healthy`** (return code 200 alone admits a booting node), `X-Forwarded-*` headers on, idle timeout raised to 300 s for SSE |
| database | OCI Database with PostgreSQL, **version 16**, primary in the region closest to most users, read replicas in the other two regions for DR. Enable the `vector`, `uuid-ossp` and `pgcrypto` extensions on the DB system (Configuration → Extensions) before running `provision.sql`; extensions not enabled at the system level cannot be created inside the database |
| database network | private endpoint; security list allows 5432 from the three node subnets over the DRG/VCN peering. Connect with `?sslmode=require`; the OCI CA is publicly chained so no `?sslrootcert=` is needed |
| secrets | OCI Vault holds `secrets.json` as one secret (base64 of the file). `render-profile.sh` runs once on an operator machine; the result goes into the Vault, and a cloud-init / Ansible step writes it to `/etc/astound/profile` on each node (`oci secrets secret-bundle get ... | base64 -d`) |
| image pull | `docker login ghcr.io` with the Astound-issued token, or mirror once into OCIR (`docker pull` then `docker tag` + `docker push` to `<region>.ocir.io/<tenancy>/systemprompt-astound:0.63.0`) and point `SYSTEMPROMPT_TAG`'s image at OCIR in the compose file |
| egress | NAT gateway per VCN for `api.anthropic.com`, `bedrock-runtime.eu-central-1.amazonaws.com`, `ghcr.io` |

Only the region-local node sits behind each region's balancer; every node
writes to the same primary, so the write latency from the far regions is the
round trip to the primary's region (typically 150–250 ms India ↔ EU). That
is the accepted launch trade-off; per-region replica reads are the phase-2
option described in `deploy/self-host/SECRETS.md` under `database_write_url`.

What Astound has verified: `scripts/e2e-published-release.sh` reproduces this
topology (PostgreSQL 16 primary with a streaming replica, three
nodes sharing one rendered profile, a TLS load balancer, the Linux bridge
installer from a config-free client) in Docker for each release; the report
is attached to the hand-over.
