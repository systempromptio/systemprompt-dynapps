---
title: "Deploy on Oracle Cloud"
description: "Single node on an OCI Compute instance with OCI Database with PostgreSQL: the resources to create, the settings that matter, and the exact commands, step by step."
author: "Astound Digital"
slug: "deploy-oracle-cloud"
keywords: "oracle cloud, oci, compute, oci database with postgresql, pgvector, load balancer, vault, deploy, self-host, docker"
kind: "guide"
public: true
tags: ["documentation", "deployment", "configuration"]
published_at: "2026-09-02"
updated_at: "2026-09-02"
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

This is the [one-node deployment](/documentation/deploy) mapped onto Oracle
Cloud Infrastructure: one Compute instance running the image, one OCI
Database with PostgreSQL system, one Load Balancer terminating TLS. Follow
the numbered steps in order; each names the OCI resource, the settings that
matter, and the command to run. The three-region layout adds nodes and
balancers to this exact base later.

## Resources

| OCI resource | Settings |
|--------------|----------|
| VCN | one VCN with a public subnet (load balancer) and a private subnet (compute + database), a NAT gateway on the private subnet for outbound HTTPS |
| Compute instance | `VM.Standard.E4.Flex`, 2 OCPU / 8 GB, Oracle Linux 9 or Ubuntu 22.04, private subnet, Docker Engine + Compose v2 installed |
| Database with PostgreSQL | **version 16**, one primary node (add read replicas later for DR only), private endpoint in the private subnet, `vector`, `uuid-ossp`, `pgcrypto` enabled on the DB system |
| Load Balancer | flexible shape, public subnet, TLS listener on 443 with your certificate, backend set → compute instance port 8080 |
| Vault | one secret holding `secrets.json` (base64 of the file) |
| Security lists | LB → compute 8080; compute → database 5432; nothing else inbound. Ports 5000–5999 and 9000–9999 on the compute instance stay closed |

## 1. Database

Create the DB system with PostgreSQL 16. Before anything else open
**Configuration → Extensions** on the DB system and enable `vector`,
`uuid-ossp` and `pgcrypto`; an extension not enabled at the system level
cannot be created inside the database, and the knowledge bank refuses to
start without `vector`.

From the compute instance (the endpoint is private), run the bundle's
provisioning script as the DB-system admin user:

```bash
psql "postgres://<admin>:<admin-pw>@<db-private-fqdn>:5432/postgres?sslmode=require" \
  -v pw="<application-db-password>" -f provision.sql
```

It creates the `systemprompt` role and database and installs the three
extensions. The OCI certificate chains to a public root, so `sslmode=require`
works with no extra CA file. The application connection string:

```
postgres://systemprompt:<application-db-password>@<db-private-fqdn>:5432/systemprompt?sslmode=require
```

## 2. Compute instance

```bash
sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin   # Oracle Linux 9
sudo systemctl enable --now docker && sudo usermod -aG docker "$USER"
git clone https://github.com/Astound-Digital/systemprompt-astound.git
cd systemprompt-astound/deploy/self-host
echo "$GHCR_TOKEN" | docker login ghcr.io -u <github-user> --password-stdin
docker pull ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@
```

Outbound access needed through the NAT gateway: `ghcr.io`,
`api.anthropic.com`, `bedrock-runtime.eu-central-1.amazonaws.com`, all on 443.

## 3. Secrets and profile

Render once, on the compute instance or an operator machine:

```bash
./render-profile.sh --out ./profile \
  --external-url https://ai.example.com \
  --admin-email you@example.com \
  --trusted-proxies 10.0.0.0/24 \
  --database-url 'postgres://systemprompt:<pw>@<db-private-fqdn>:5432/systemprompt?sslmode=require' \
  --anthropic-key sk-ant-... \
  --bedrock-key ... \
  --image ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@
```

`--trusted-proxies` is the CIDR of the load balancer's subnet. Store the
result in Vault and place it on the instance:

```bash
base64 -w0 profile/secrets.json | oci vault secret create-base64 --compartment-id ... --vault-id ... --key-id ... \
  --secret-name astound-secrets --secret-content-content file:///dev/stdin
sudo install -d -m 0750 -o 1000 -g 1000 /etc/astound/profile
sudo install -m 0644 -o 1000 -g 1000 profile/profile.yaml /etc/astound/profile/profile.yaml
oci secrets secret-bundle get --secret-id <ocid> --query 'data."secret-bundle-content".content' --raw-output \
  | base64 -d | sudo install -m 0600 -o 1000 -g 1000 /dev/stdin /etc/astound/profile/secrets.json
```

The container runs as uid 1000 and mounts `/etc/astound/profile` read-only.

## 4. Boot

```bash
export SYSTEMPROMPT_TAG=@RELEASE_VERSION@ ASTOUND_PROFILE_DIR=/etc/astound/profile
export DATABASE_URL='postgres://systemprompt:<pw>@<db-private-fqdn>:5432/systemprompt?sslmode=require'
docker compose -f docker-compose.node.yml pull
docker compose -f docker-compose.node.yml up -d
docker compose -f docker-compose.node.yml ps            # wait for: healthy
```

`docker-compose.node.yml` is the app-only file: it needs `DATABASE_URL` for the
readiness probe and the same value inside `secrets.json`. If `verify-node.sh`
later reports the public site missing (older entrypoints skipped the render
on this node), run it once after `healthy`:

```bash
docker compose -f docker-compose.node.yml exec \
  -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml \
  app systemprompt infra jobs run publish_pipeline
```

## 5. Load balancer

| Setting | Value |
|---------|-------|
| Listener | HTTPS 443, your certificate |
| Backend set | the compute instance, port 8080, policy round robin |
| Health check | HTTP, path `/api/v1/health`, status 200, **response body regex** `healthy`. Status alone admits a node that is still migrating |
| Headers | forward `X-Forwarded-For` and `X-Forwarded-Proto`; keep the `Host` header |
| Timeouts | idle timeout 300 s (streamed responses) |
| Rules | return 403 for `/metrics` and `/api/v1/health/detail` |
| Drain | when stopping the node, drain it in the backend set and wait 30 s before `docker compose stop` |

Point the DNS name in `--external-url` at the load balancer.

## 6. Verify and sign in

```bash
./verify-node.sh --url http://localhost:8080 \
  --container "$(docker compose -f docker-compose.node.yml ps -q app)" --version @RELEASE_VERSION@
curl -fsS https://ai.example.com/api/v1/health
```

Every line of the checklist must pass. Then sign in at
`https://ai.example.com/admin/login` with the admin e-mail, register a passkey,
and send users to [Downloads](/documentation/downloads), served by this node.

## Upgrades, backups, growth

- Upgrade: drain at the load balancer, `SYSTEMPROMPT_TAG=<new>` then `pull` and
  `up -d`, run `verify-node.sh`, re-enable. Migrations are forward-only; take an
  OCI PostgreSQL backup first.
- Backups: enable automatic backups on the DB system and test a restore once
  before launch.
- More nodes: create a compute instance and load balancer per region, copy
  `/etc/astound/profile` unchanged, add each region's balancer subnet to
  `--trusted-proxies` and re-render, and boot nodes one at a time. All nodes use
  the same primary; read replicas are for disaster recovery only.
