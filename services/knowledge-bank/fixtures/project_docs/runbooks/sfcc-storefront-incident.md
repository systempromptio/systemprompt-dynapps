---
title: "Runbook - SFCC storefront incident"
space: EAICOMM
page_id: "4719133"
date: 2026-08-12
status: Current
owner: Ben Carver (Dev lead)
---

# Runbook - SFCC storefront incident

## Declaring an incident

An incident is declared when the production storefront is unavailable, when checkout fails for a meaningful share of customers, when the order feed to OMS stops, when customer data is exposed, or when a synthetic monitor is red for more than five minutes. Anyone on the delivery team or at Northwind may declare one by posting in the incident channel with the word INCIDENT, a one-line description and the time they noticed it. The first Astound Digital engineer to respond becomes the incident commander until they hand over explicitly. Severity is set at declaration: S1 for revenue-stopping, S2 for degraded checkout or a broken feed, S3 for a visible defect with a workaround. S1 and S2 page the on-call developer and inform Marcus Feld immediately.

## Roles during the incident

The incident commander owns the timeline, decides on mitigation, and is the only person who speaks to Northwind during the incident; everyone else reports to them in the channel. A scribe, usually the BA on duty, keeps the timeline in the channel with timestamps for every observation, decision and action. Engineers work the diagnosis and report findings rather than acting on their own. Miriam Okafor is informed of every S1 and S2 within fifteen minutes and takes over client communication for anything longer than an hour. Nobody deploys, changes a preference or clears a cache without the commander recording the decision first.

## Diagnosis checklist

Check the platform status page for a realm-wide event before looking at our code. Check the most recent deploy and the most recent preference change; the majority of storefront incidents in the past year followed one of the two within an hour. Read the error log grouped by error class for the last thirty minutes and compare with the previous day at the same hour. Check the integration job history for the OMS order export and the ERP inventory import; a stalled import shows up as sold-out products, a stalled export as orders without confirmation emails. Check the PSP status page if payment authorisations are failing. If the incident coincides with a marketing send, check the cache hit rate: a mis-cached page under load is the classic cause of a sudden spike in origin traffic.

## Mitigation options

In order of preference: turn off the feature behind a flag if the incident is confined to one feature; activate the previous code version if the incident followed a deploy; invalidate a page cache entry if a stale or broken page is being served; disable a promotion if pricing is wrong; fail over the PSP to its secondary endpoint if authorisations fail. A code hotfix is the last resort and is deployed by the commander with the reduced checklist: build from a hotfix branch off the production tag, run the unit tests, upload, activate, smoke-test checkout. A hotfix is followed within a day by the full deploy runbook so that the release branch and production agree again.

## Closing and follow-up

The incident is closed when the commander confirms the trigger is resolved and the monitors have been green for thirty minutes. The scribe turns the timeline into a post-incident review page within two working days, with a factual timeline, the root cause, the customer impact expressed in orders and minutes, and actions with owners and dates. The review is blameless and is shared with Northwind. Actions become Jira tickets in the next sprint and are tagged incident-follow-up so the steering committee can see them. Recurring causes are added to the diagnosis checklist above.
