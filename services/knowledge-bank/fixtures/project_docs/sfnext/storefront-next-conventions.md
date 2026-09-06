---
title: "Storefront Next - conventions for this project"
space: EAICOMM
page_id: "4719301"
date: 2026-07-02
status: Current
owner: Ana Lucia Ferreira (Technical Architect)
---

# Storefront Next - conventions for this project

## Extend, do not fork

The Northwind Trading storefront is built on the Storefront Next reference application, which ships as a versioned package with a React Router application, a design system, and a commerce data layer. The reference application is a dependency, not a starting point to copy: project code lives in the `app/` overlay and composes or wraps the reference components, and the reference package is upgraded on its release cadence. A component is copied out of the reference package only when a written note in `docs/forks.md` records why extension was impossible, what was copied, and the version it was copied from, so that the next upgrade can reconcile it. A pull request that adds a fork without that note is not merged. This is the single most important rule on the project, because the previous storefront became unupgradeable within a year through exactly this drift.

## Routing

Routes are declared in `app/routes.ts` using React Router's file-based conventions with explicit route modules, one module per route, exporting `loader`, `action`, `meta` and the default component. Data loading happens in loaders, never in effects inside components, so that server rendering, prefetching and error boundaries all work the same way; a component that needs data it did not receive from its loader is a design error to fix at the route, not with a fetch in the component. Route modules stay thin: the loader calls a data function in `app/data/`, which is where the commerce client is used and where caching headers are decided. Nested routes are used for layouts (the account area, the checkout stages) and error boundaries are declared at the layout level so a failing stage never takes down the shell.

## Approved libraries

The project adds no runtime dependency without an architecture review. The approved set beyond the reference application is: `zod` for schema validation at every boundary that receives external data (form submissions, PSP callbacks, the configurator template), `date-fns` for date formatting with the site locale, and `@tanstack/react-table` for the account-area order and saved-configuration tables. State management is React Router's loaders and actions plus local component state; there is no global store and a proposal to add one must show a case the router cannot serve. Styling is the design system's tokens and components; project components use the design system's styling primitives rather than a separate CSS approach, so that theme changes from the design team apply everywhere.

## Components and naming

Project components live under `app/components/<domain>/` where the domain is one of `catalog`, `basket`, `checkout`, `account`, `content` or `shared`, are named in PascalCase after what they show rather than where they sit, and export one component per file with its props type beside it. A component that wraps a reference component keeps the reference name with a project prefix, for example `NwtProductTile` around `ProductTile`, so that a reader can see the relationship. Feature flags are read through the `useSitePreference` hook, which is the only place that knows the preference names; a component that reads a flag by string is refactored in review. Every component that renders customer-visible text uses the localisation hook, even for the site's single launch language, because the second language is already on the roadmap.

## Testing and accessibility

Every route module has a loader test and a rendering test using the reference test utilities with a mocked commerce client; components with logic have unit tests; and the checkout, account and configurator flows have Playwright tests that run against the staging instance nightly. The accessibility scan runs in CI on every pull request and fails the build on any violation at the AA level, without exceptions and without a suppression list, because a suppression list grows and is never emptied. Pull requests link the Jira ticket in the title, follow the ticket's acceptance criteria, and include a screenshot or recording for any visual change.
