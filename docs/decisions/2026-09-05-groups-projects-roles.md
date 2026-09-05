# 2026-09-05 — Groups, projects, flat roles, and one marketplace per group

**Status:** decided (Ed, 2026-09-05). Supersedes the "two projects from two AD
groups" spine in `2026-09-01-launch-scope-identity-and-deploy.md` §2.

## Decisions

1. **Two entities: Group and Project.** A *group* carries people and marketplace
   entitlement (Commerce devs, Europe devs, Business users). A *project* is
   work attribution; a user belongs to zero or more, and a project is also an
   ACL subject so a marketplace, plugin, skill or MCP server can be gated to
   it. Both are admin-managed rows (`groups`, `projects`, migration
   `041_groups_projects.sql`), not a hardcoded column. The
   `user_profile_ext.project` column, the `Project` enum and `ProjectScope`
   are deleted outright; nothing was in production with them.

2. **AD groups map into groups and projects through tables**
   (`group_ad_mappings`, `project_ad_mappings`), bootstrapped from
   `services/web/config/groups.yaml` and editable in the dashboard.
   Directory-derived membership (`source = 'adfs'`) is replaced on every
   sign-in; dashboard membership (`source = 'manual'`) survives it. A sign-in
   whose groups map to nothing lands in the system group **`unassigned`**,
   whose membership is derived (view `user_groups`): the user keeps everything
   the `user` role grants and fails only group- or project-gated guards until
   an admin assigns them. The `adfs_group` subject dimension is gone; `group`
   (precedence 150) and `project` (140) replace it.

3. **Flat roles, no hierarchy.** `platform_admin`, `admin`, `developer`,
   `user`, `project_manager`. Every route names the roles it accepts:
   console reads = platform_admin | admin | project_manager; writes =
   platform_admin | admin; AD mappings and granting or revoking
   `platform_admin` = platform_admin only, and the last platform_admin cannot
   be removed. `developer` is an entitlement role (it selects developer
   marketplaces) with no admin console. `users.roles` is the union of the
   directory roles and dashboard-granted `user_manual_roles`.

4. **One marketplace per audience.** Core's manifest now unions every enabled
   marketplace the caller is granted instead of resolving a single active one,
   and a marketplace's `access` block can carry `rules` against extension
   dimensions (`group`, `project`) beside `roles`. This instance ships
   `enterprise-demo` (baseline for every `user`), `astound-europe-dev`,
   `astound-india-dev` and the role-gated `astound-cowork` (relabelled on
   2026-09-05 from `astound-commerce-dev` / `astound-europe-dev` /
   `astound-business`; the `commerce-devs` group became `europe-devs`, the old
   `europe-devs` became `india-devs`, and `knowledge_worker` joined the role
   set as the Cowork entitlement).

## Why

Governance keyed on two fixed groups had to be redesigned the moment a third
audience appeared; a group table grows by inserting a row. Keeping people
(groups) apart from work (projects) lets a marketplace be handed to an
audience while cost and skill usage are still attributed to the project the
work was for. Flat roles keep every route's guard readable as a list, which is
what an auditor asks for. Deriving `unassigned` instead of storing it means
assignment is one write and the directory never has to know about the bucket.

## Consequences

- Admin pages: Groups (list, detail with overview / members / access /
  projects / AD mappings), Unassigned, Projects, Marketplaces with an audience
  matrix, Access control, and groups/projects/roles on the user pages.
- Every scoped listing binds a set of user ids (`SubjectScope`) resolved from
  the caller's visibility plus optional `?group=` / `?project=` filters.
- Contract baseline gains `platform-admin` and `developer` principals.
- Core `next` carries the multi-marketplace change; `bridge/CORE_REF` points at
  it until the release lands on `main`.
