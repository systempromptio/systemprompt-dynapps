---
title: "Client brief - Northwind Trading"
account: Northwind Trading GmbH
industry: Industrial tools and workshop equipment retail
status: Active
owner: Miriam Okafor
date: 2026-08-14
last_touch: 2026-08-13
---

# Client brief - Northwind Trading

## Company and industry

Northwind Trading is a Hamburg-based retailer of professional tools, workbenches, storage systems and workshop consumables, selling to trade customers and serious hobbyists across Germany, Austria and Switzerland through a storefront, a printed catalogue and three showrooms. Revenue is roughly two thirds online. The category is competitive on price and delivery speed, and Northwind competes on range depth and on configurable workshop furniture that the marketplaces do not carry well. The business is family owned, decides quickly, and expects its partners to do the same.

## Platforms

The storefront runs on Salesforce Commerce Cloud with the Storefront Next React Router front end, replaced in 2025 after the previous SFRA site became unupgradeable. Orders flow to a third-party OMS and inventory comes from an on-premise ERP through nightly and intraday jobs owned by Marcus Feld's team. Payments are through a PSP with hosted card fields and network tokenisation. Marketing automation is a separate platform that reads basket and order events from the storefront. Astound Digital's own engagement record lives in the Salesforce Core org under the objects described in "Salesforce Core - engagement reporting schema".

## Key contacts

Helen Marsh, Chief Digital Officer, executive sponsor and steering committee chair. Daniel Reyes, Head of E-commerce, product owner and scope tie-breaker. Sunita Bhatt, Head of Customer Service, sign-off on anything touching orders, returns and saved payments. Marcus Feld, IT lead, owner of the integrations, the change calendar and production access. Contact rules and decision rights are in "Stakeholders - Northwind Trading storefront programme".

## Engagement scope

A twelve-month storefront programme on a time-and-materials SOW with two fixed-price change requests to date, delivering in two-week sprints: the Express Checkout pilot (epic EAICOMM-101), the Account Saved Payments feature, the PDP Configurator for the workshop furniture range, and the operational runbooks for deploy, incident and cache management. The team is a BA lead, a second BA, an architect, four storefront developers, two integration developers and a QA lead, with Salesforce Core support for the engagement reporting.

## Open opportunities

A second language and a French-speaking Swiss storefront in the next financial year, for which the localisation groundwork is already required by the storefront conventions. A B2B account structure with buyer roles and approval limits for the trade customers, which Daniel raised at the August steering committee. A replatforming of the marketing automation triggers onto the storefront's event stream, which would remove the abandoned-checkout ambiguity tracked in EAICOMM-110.

## Risks

The production pilot of Express Checkout depends on EAICOMM-107 closing, and a second slip would cost credibility with Helen before the B2B conversation. The configurator's rendering service has not been load tested for a sale day. Marcus's team is a single point of knowledge for the ERP integration and holidays in late August thin it further. The RAID log on the Salesforce record is current as of 2026-08-13.
