---
title: "FSD - Account Saved Payments"
space: EAICOMM
page_id: "4718644"
date: 2026-06-30
status: Approved
owner: Priya Raman (BA)
---

# FSD - Account Saved Payments

## Purpose and scope

Saved Payments lets an authenticated Northwind Trading customer store a card with the payment service provider at the end of a successful checkout, or from the account area, and reuse it later with only a CVV re-entry where the card scheme demands it. The storefront never sees or stores a card number: what is stored on the customer profile is a PSP token reference, the card brand, the last four digits, the expiry month and year, and the billing address identifier. This document covers the account-area screens, the consent rules, the token lifecycle, and the storefront-side validation. The PSP integration cartridge is configured, not written, for this feature; any change to it is a separate specification.

## Consent and storage rules

A card is saved only when the customer ticks "Save this card for next time" during checkout, or explicitly adds it from the account area; there is no default-on behaviour and the checkbox state is not remembered between sessions. The consent copy was signed off by legal on 2026-06-18 and must be rendered verbatim from the content asset `saved-payments-consent`, never hard-coded in a component. A profile may hold at most five saved payment methods; adding a sixth prompts the customer to remove one first. Removing a method calls the PSP to revoke the token before the profile record is deleted, and a PSP failure on revoke leaves the record in place with status REVOKE_PENDING so a scheduled job can retry it rather than orphaning a live token at the provider.

## Token lifecycle

Each saved method carries a status: ACTIVE, EXPIRED, REVOKE_PENDING, or REVOKED. A nightly job moves methods whose expiry month has passed to EXPIRED and sends the "card expiring" email thirty days before that happens, once. EXPIRED methods remain listed in the account area with an "Update card" action that runs the add-card flow and, on success, replaces the record in place so the customer's default selection survives. Only ACTIVE methods are offered at checkout and only ACTIVE methods make a customer eligible for Express Checkout, which is why "FSD - Express Checkout" reads this status rather than checking expiry itself. A token the PSP reports as invalid at authorisation time is moved to EXPIRED immediately and the customer is asked for a different method.

## Account area screens

The "Payment methods" page lists saved methods as cards showing brand, masked number, expiry and the billing address's first line, with the default method marked and a "Make default" action on the others. "Add a payment method" opens the PSP-hosted card form inside the account layout using the same iframe component as checkout, so the storefront's PCI scope does not widen. The page is reachable only over an authenticated session with a fresh login: a session older than fifteen minutes since authentication is asked to re-enter the password before the page renders, matching the existing behaviour of the "Change email" page. Every action shows a confirmation toast and writes an audit line to the customer's profile notes with the action, the masked number and a timestamp.

## Validation and errors

The storefront validates expiry as a future month, brand as one of the enabled brands for the site, and the billing address as a complete address that passed verification when it was saved. A PSP tokenisation failure surfaces the provider's customer-safe message and keeps the form open with entered non-card fields intact. Any attempt to use a method whose profile owner does not match the session's customer is refused server-side and logged as a security event; this is a defence in depth, since the UI never lists another customer's methods. The maximum-methods rule and the consent rule are both enforced on the server, not only in the form, so a scripted request cannot bypass them.

## Reporting

The number of customers with at least one ACTIVE saved method, the number of checkouts that used a saved method, and the count of REVOKE_PENDING records older than a day are exposed as three custom report metrics. The third is an operational alarm: a non-zero value for more than a day means the PSP revoke job is failing and someone must look at the job log before the next PCI review asks why live tokens exist without a customer record.
