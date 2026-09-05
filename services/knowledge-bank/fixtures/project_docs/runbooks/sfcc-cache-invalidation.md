---
title: "Runbook - SFCC cache invalidation"
space: EAICOMM
page_id: "4719140"
date: 2026-07-28
status: Current
owner: Kofi Mensah (Integration lead)
---

# Runbook - SFCC cache invalidation

## Why the cache matters

The Northwind Trading storefront serves category pages, product detail pages and content pages from the platform page cache, with a time-to-live per page type set in the caching rules of the site. Origin traffic is a small fraction of total traffic while the cache is warm, and the storefront's capacity plan assumes it. A full invalidation on a trading day sends every page to origin at once and has, on two occasions, produced an incident that looked like an application failure. This runbook therefore treats invalidation as a change, not a reflex: it says when to invalidate, how narrowly, and how to warm afterwards.

## When to invalidate

Invalidate a single page or a category subtree when the content of that page is wrong and its natural time-to-live is longer than the business can wait, for example a price correction on a product that is being promoted, a legal text change, or a broken image on a landing page after a content deploy. Invalidate the whole site only after a deploy whose release notes say so explicitly, which is the case when the page templates changed or a shared component changed its markup, and only inside the deploy window. Do not invalidate to "make sure": a page that is correct in the cache does not become more correct at origin, and the load cost is real. Content editors invalidate their own pages through the content tool; developers invalidate through the administration interface or the CLI and record the action in the deploy or incident channel.

## How to invalidate narrowly

For a product, invalidate by product ID, which clears the PDP and every category page listing that product. For a category, invalidate the category with subcategories only if the subtree is small; a top-level category on this site is hundreds of pages and is closer to a full invalidation than it looks. For a content page, invalidate the content asset by ID. For a change in a shared header or footer, there is no narrow option: the page templates embed them, and the site must be invalidated. Static assets are versioned by the build and never need invalidation; if an old bundle is being served after a deploy, the cause is the code version, not the cache.

## Warming after a full invalidation

After a full invalidation, warm the cache before customers do: run the warming script against the top two hundred pages from the previous week's analytics export, at a concurrency of four, and watch origin response times until they return to the normal band. The script lives in the tools repository and takes the site host and the page list; it logs any page that returns a non-200 status, which is worth reading because a broken page after a deploy shows up here before it shows up in support tickets. Warming takes about ten minutes for the standard list and is part of the deploy checklist whenever a full invalidation is performed.

## Recording

Every invalidation beyond a single page is posted in the release channel with what was invalidated, why, by whom and when, so that anyone reading the incident diagnosis checklist can correlate a load spike with a cache action. The steering committee receives a monthly count of full invalidations; the target is zero outside deploy windows.
