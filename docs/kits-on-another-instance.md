# Composing the Astound kits on another instance

The kits are published once, by their repositories' CI, as signed services
bundles on GHCR. Any systemprompt instance built from this repository can
compose them; nothing in the kit is instance-specific. This is what an
operator of a separate deployment — `sp-dev.astound.digital` today — does to
follow the same kits as `astound.systemprompt.io`. The model is
`/documentation/services-sync`; the kit contract is
`deploy/kit/README-INTEGRATION.md`.

## 1. A pull token for the private packages

The two Astound kits are private GHCR packages. The instance pulls them with
one secret:

| secret | value |
|---|---|
| `ghcr_pull_token` | `<github-user>:<PAT>` — a GitHub personal access token (classic) with the single scope `read:packages`, on an account that can read the `Astound-Digital` organisation's packages |

Put it in the profile's `secrets.json` (or the equivalent secret store the
deployment uses; `secrets.source` in the profile says which). The public
`astound-ba` kit needs no token.

## 2. The sources block

Add to the instance's profile (`.systemprompt/profiles/<name>/profile.yaml`):

```yaml
services:
  sources:
    - name: astound-ba
      oci:
        reference: ghcr.io/systempromptio/astound-ba-skills:stable
        verify:
          ed25519_public_keys: ["otpJ3aM8qDX1P60cOMRURDCxzwB94PO+uMnh5rUJ/b0="]
    - name: sfnext
      oci:
        reference: ghcr.io/astound-digital/sfnext-claude-dev-kit:stable
        auth_secret: ghcr_pull_token
        verify:
          ed25519_public_keys: ["CyTVcLiluJ5zeCfmTt0QlOxJvZf+f1+UkubOjZP8K2Q="]
    - name: india
      oci:
        reference: ghcr.io/astound-digital/sfscore-claude-india-kit:stable
        auth_secret: ghcr_pull_token
        verify:
          ed25519_public_keys: ["a8Xa0VL5BFVauW8NsUmVktmzj8G+9+qt6//UBZH4X0o="]
  cache_dir: /app/services-cache        # any writable directory
  on_fetch_failure: use_last_good       # use_bundled on the very first boot
```

The public keys are the ones recorded in `deploy/kit/known-kits.json`; a
bundle signed by any other key is refused. `just services-pin <kit> stable
<profile>` writes exactly these entries.

## 3. Entitlement stays in this repository

Who reaches a kit's marketplace is declared in
`services/access-control/rules.yaml` (`marketplace/astound-india-dev` →
`group: [india-devs]`, `marketplace/astound-europe-dev` → `group:
[europe-devs]`, `project: [storefront]`), and the groups are mapped from the
directory in `services/web/config/groups.yaml` (`Systemprompt-Core`,
`Systemprompt-Commerce`). An instance deployed from this repository carries
both; a person in neither group receives the commons only.

## 4. Import after every kit release

A kit release moves its `stable` tag. The instance serves it after one
**Import sources** on `/admin/sync`, or:

```bash
curl -fsS -X POST -H "Authorization: Bearer <admin PAT>" \
  https://<instance>/api/v1/admin/services/refresh
# {"changed":true,"reconciled":true,"restart_recommended":false,...}
```

Nothing restarts. The kit CI can make this call itself for **one** instance
(its `SYSTEMPROMPT_API_URL` / `SYSTEMPROMPT_ADMIN_TOKEN` secrets); every other
instance imports on its own schedule. `GET /api/v1/admin/services/status`
shows the active digest per source.

## 5. Rollback

`just services-pin <kit> sha256:<previous digest> <profile>` and Import; the
cache keeps the last two trees per source. The digests are in each kit's
release run summary and on the GHCR package page.
