---
title: "Deploy on Oracle Cloud"
description: "Single node on an OCI Compute instance with OCI Database with PostgreSQL: the resources to create, the settings that matter, and the exact commands, step by step."
author: "Astound Digital"
slug: "deploy-oracle-cloud"
keywords: "oracle cloud, oci, compute, oci database with postgresql, load balancer, vault, nat gateway, deploy, self-host, docker"
kind: "guide"
public: true
tags: ["documentation", "deployment", "configuration"]
published_at: "2026-09-02"
updated_at: "2026-09-29"
after_reading_this:
  - "Create the OCI Compute, PostgreSQL and Load Balancer resources with the right settings"
  - "Enable the PostgreSQL extensions at the DB-system level before provisioning"
  - "Store the rendered secrets in OCI Vault and place them on the node"
  - "Boot, verify and sign in"
related_playbooks:
  - title: "Deploy: Validate Every Stage"
    url: "/documentation/deploy-troubleshooting"
  - title: "Deploy: One Node and PostgreSQL"
    url: "/documentation/deploy"
  - title: "Expose Your Instance Remotely"
    url: "/documentation/remote-access"
  - title: "Downloads"
    url: "/documentation/downloads"
  - title: "Proposed: Target Deployment Architecture"
    url: "/documentation/deploy-target-architecture"
---

# Deploy on Oracle Cloud

This is the [one-node deployment](/documentation/deploy) on Oracle Cloud
Infrastructure: one Compute instance running the image, one OCI Database with
PostgreSQL system, and one Load Balancer terminating TLS. **Read the one-node
page first.** This page gives only what differs on OCI: the resources, where
each step runs, and the OCI commands. Everything else, including the render
flags, the secrets and the verification checklist, is the same and is not
repeated here. The three-region layout later adds nodes and balancers to this
same base.

## Resources

| OCI resource | Settings |
|--------------|----------|
| VCN | a public subnet (load balancer) and a private subnet (compute and database), with a NAT gateway on the private subnet for outbound HTTPS |
| Compute instance | `VM.Standard.E4.Flex`, 2 OCPU / 8 GB, Oracle Linux 9 or Ubuntu 22.04, private subnet, Docker Engine and Compose v2 installed |
| Database with PostgreSQL | **version 16**, one primary node (add read replicas later, for DR only), private endpoint in the private subnet, `uuid-ossp` and `pgcrypto` enabled on the DB system |
| Load Balancer | flexible shape, public subnet, TLS listener on 443 with your certificate, backend set → compute instance port 8080 |
| Vault | one secret holding `secrets.json` (base64 of the file) |
| Security lists | LB → compute 8080; compute → database 5432; nothing else inbound. Ports 5000–5999 and 9000–9999 on the compute instance stay closed |

Outbound through the NAT gateway, all on 443: `ghcr.io`,
`aiplatform.googleapis.com` and `oauth2.googleapis.com` (Vertex AI serves
every Claude request), and `api.anthropic.com` (the failover).

## 1. Database

Create the DB system with PostgreSQL 16. Before anything else, open
**Configuration → Extensions** on the DB system and enable `uuid-ossp` and
`pgcrypto`. An extension that is not enabled at the system level cannot be
created inside the database, and `provision.sql` then fails.

The endpoint is private, so run `provision.sql` from the compute instance as
the DB-system admin user, exactly as in step 2 of the one-node page, against
`<db-private-fqdn>`. The OCI certificate chains to a public root, so
`sslmode=require` works without an extra CA file.

## 2. Compute instance

```bash
sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin   # Oracle Linux 9
sudo systemctl enable --now docker && sudo usermod -aG docker "$USER"
```

Then clone the bundle and log in to the registry as in step 1 of the one-node
page.

## 3. Secrets and profile, in Vault

Run `render-profile.sh` once, on the compute instance or an operator machine,
with the flags from step 3 of the one-node page. One value differs:
`--trusted-proxies` is the **CIDR of the load balancer's subnet** (for
example `10.0.0.0/24`), not `127.0.0.0/8`.

Store the rendered `secrets.json` in Vault and place both files on the
instance:

```bash
oci vault secret create-base64 --region <region> --compartment-id ... --vault-id ... --key-id ... \
  --secret-name astound-secrets --secret-content-content "$(base64 -w0 profile/secrets.json)"
sudo install -d -m 0750 -o 1000 -g 1000 /etc/astound/profile
sudo install -m 0644 -o 1000 -g 1000 profile/profile.yaml /etc/astound/profile/profile.yaml
oci secrets secret-bundle get --region <region> --secret-id <ocid> --query 'data."secret-bundle-content".content' --raw-output \
  | base64 -d | sudo install -m 0600 -o 1000 -g 1000 /dev/stdin /etc/astound/profile/secrets.json
```

Two OCI CLI traps:

- **Pass `--region` on every `oci` command** (for example `eu-frankfurt-1`).
  Without it the CLI uses the active profile's region, which resolves to a
  different tenancy endpoint and fails with a not-found or auth error even
  though the OCID is correct.
- **`--secret-content-content` takes the base64 value itself**, not a file
  reference. The OCI CLI rejects `file:///dev/stdin`. On macOS, use `base64`
  with no flags in place of `base64 -w0`.

The container runs as uid 1000 and mounts `/etc/astound/profile` read-only.

## 4. Boot

OCI uses the app-only compose file, because the database is external:

```bash
export SYSTEMPROMPT_TAG=@RELEASE_VERSION@ ASTOUND_PROFILE_DIR=/etc/astound/profile
export DATABASE_URL='postgres://systemprompt:<pw>@<db-private-fqdn>:5432/systemprompt?sslmode=require'
docker compose -f docker-compose.node.yml pull
docker compose -f docker-compose.node.yml up -d
docker compose -f docker-compose.node.yml ps            # wait for: healthy
```

`docker-compose.node.yml` needs `DATABASE_URL` in the environment for its
readiness probe, and the same value inside `secrets.json`.

## 5. Load balancer

| Setting | Value |
|---------|-------|
| Listener | HTTPS 443, your certificate |
| Backend set | the compute instance, port 8080, round-robin policy |
| Health check | HTTP, path `/api/v1/health`, status 200, **response body regex** `healthy`. A status check alone admits a node that is still migrating, because it answers `{"status":"starting"}` |
| Headers | forward `X-Forwarded-For` and `X-Forwarded-Proto`; keep the `Host` header |
| Timeouts | idle timeout 300 s (streamed responses) |
| Rules | return 403 for `/metrics` and `/api/v1/health/detail` |
| Drain | when stopping the node, drain it in the backend set and wait 30 s before `docker compose stop` |

Point the DNS name you passed as `--external-url` at the load balancer.

## 6. Verify and sign in

Run `verify-node.sh` as in step 6 of the one-node page, with
`docker-compose.node.yml` in place of the single-node file, then check the
public address:

```bash
./verify-node.sh --url http://localhost:8080 \
  --container "$(docker compose -f docker-compose.node.yml ps -q app)" --version @RELEASE_VERSION@
curl -fsS https://ai.example.com/api/v1/health
```

Every check must pass. Then configure [Astound SSO](/documentation/adfs-sso-setup)
for this hostname and sign in at `https://ai.example.com/admin/login`.

## Upgrades, backups, growth

- **Upgrade.** Drain the node at the load balancer, set `SYSTEMPROMPT_TAG=<new>`,
  then `pull` and `up -d`, run `verify-node.sh`, and re-enable the node.
  Migrations are forward-only, so take an OCI PostgreSQL backup first.
- **Backups.** Enable automatic backups on the DB system, and test a restore
  once before launch.
- **More nodes.** Create a compute instance and a load balancer per region and
  copy `/etc/astound/profile` to it unchanged. Add each region's balancer
  subnet to `--trusted-proxies`, re-render, and boot the nodes one at a time.
  Every node uses the same primary; read replicas are for disaster recovery
  only.
