---
title: "Runbook - SFCC storefront deploy"
space: EAICOMM
page_id: "4719120"
date: 2026-08-12
status: Current
owner: Ben Carver (Dev lead)
---

# Runbook - SFCC storefront deploy

## When this runbook applies

This runbook is followed for every deployment of the Northwind Trading storefront code to the staging and production instances of the Salesforce Commerce Cloud realm. It covers the code version upload, the Storefront Next bundle, the metadata import, the activation, and the verification. It does not cover data migrations that touch customer records, which need a change ticket from Marcus Feld and their own plan, nor hotfixes during an incident, which follow "Runbook - SFCC storefront incident" and are deployed by the incident commander with a reduced checklist. A deploy to a developer sandbox needs none of this; use the CLI directly.

## Preconditions

The release branch is cut, tagged, and green on the CI pipeline including the unit tests, the storefront integration tests and the accessibility scan. The release readiness meeting has recorded a go. The metadata export for the release has been reviewed by the BA and compared against the sandbox where the stories were accepted. The code version name follows `nwt_YYYYMMDD_NN`. The person deploying holds the "Deploy" role in Account Manager for the realm and has an active session in the CLI with the realm's client credentials from the team vault, never a personal token. The on-call developer and Lena Sørensen from QA are available for the verification window. For production, the change ticket number from Marcus is in hand and the deploy window is inside the agreed hours of 06:00 to 08:00 CET on a weekday that is not a trading peak.

## Steps

Build the release artefact with the release recipe, which produces the cartridge archive and the Storefront Next bundle with the git tag stamped into both. Upload the code version to the target instance and confirm it lists with the expected name. Import the metadata archive for the release into the target instance and check the import log for warnings; a warning about an unknown attribute means the archive was built against the wrong branch and the deploy stops here. Activate the code version. Run the smoke pack: home page, a category page, a PDP for a simple product and for a configurable product, guest checkout to the payment stage, and a signed-in checkout using a test account with a saved card. Invalidate the page cache only if the release notes say so; see "Runbook - SFCC cache invalidation" for how and why. Announce completion in the release channel with the code version and the smoke pack result.

## Verification

Within fifteen minutes of activation, confirm that order placement is working by checking that the order count in the last fifteen minutes on production is within the normal range for the hour, that the error log shows no new error class, and that the two synthetic monitors (home page and checkout) are green. Confirm the feature flags for the release match the release notes; a flag that should be off but is on is a rollback trigger. Lena records the verification in the release ticket. If any check fails, roll back by activating the previous code version, which is always kept on the instance, and re-run the smoke pack before declaring the rollback complete.

## Rollback

Activating the previous code version reverts the code within a minute. Metadata cannot be rolled back by activation: if the release imported metadata that the previous code cannot tolerate, the reverse metadata archive prepared at release readiness is imported first. Site preferences changed during the release are reverted by hand from the list in the release notes. After a rollback the incident process applies: an incident is opened, Marcus is informed, and the failed release is analysed before it is rescheduled. The previous code version is never deleted from an instance while it is the rollback target.
