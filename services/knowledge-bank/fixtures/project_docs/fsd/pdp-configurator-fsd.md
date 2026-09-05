---
title: "FSD - PDP Configurator"
space: EAICOMM
page_id: "4718702"
date: 2026-08-05
status: In review
owner: Tomasz Wieczorek (BA)
---

# FSD - PDP Configurator

## Purpose and scope

The PDP Configurator replaces the flat variation selector on Northwind Trading's configurable products (workbenches, storage systems and tool trolleys) with a guided, step-based configurator on the product detail page. A customer chooses a base unit, then dimensions, then material and finish, then optional accessories, and sees a live price and a rendered preview at each step. The configuration is stored on the basket line as a set of option values, priced by the existing option pricing, and passed to OMS unchanged. This document covers the storefront behaviour, the data model on the product, the pricing rules, and the constraints engine; it does not cover the rendering service, which is a third-party image API specified separately.

## Product data model

A configurable product is a standard master product with a custom attribute `c_configuratorTemplate` naming a template stored as a content asset in the library folder `configurator/templates`. The template is JSON describing the steps, the options in each step with their option IDs, the constraints between options, and the preview layer mapping. Each option value maps to an existing option product value so that pricing and inventory use the platform's option pricing and no new pricing tables are introduced. A product whose template fails to parse renders the flat variation selector and logs a warning; the configurator never blocks a sale on a data error, which was the principal lesson from the previous storefront's launch.

## Constraints engine

Constraints are declared in the template as rules of the form "if option A has value X then option B may only be one of {Y, Z}" or "option C is required when option D is present". The engine evaluates the rules on every change, disables invalid values with a tooltip naming the rule in customer language, and, if the current selection has become invalid, resets the affected step to its default and shows a message. Rules are evaluated on the client for responsiveness and re-evaluated on the server at add-to-basket, and the server is authoritative: a client that submits an invalid combination receives a structured error listing the violated rules and the PDP re-renders with the corrected state. Rule authors are the product data team, not developers; the template schema is validated on upload.

## Pricing and display

The displayed price is the base unit price plus the sum of the selected option prices, recomputed on the client from the option prices delivered with the page and confirmed by the server on add-to-basket; a mismatch of more than one minor currency unit fails the add and refreshes the prices. Promotions apply to the basket line as normal and are not previewed inside the configurator, which shows the list price and a "promotions applied in basket" note when the customer belongs to a group with an active promotion on the category. Prices are shown in the site currency with tax display following the site's existing rule. The live preview updates within 300 ms of a change; while a render is pending, the last good image stays with a subtle progress indicator rather than a blank frame.

## Saved configurations

An authenticated customer may save a configuration to a named list stored on the profile as a custom object `SavedConfiguration` holding the product ID, the template version, and the option values. A saved configuration is re-validated on open against the current template and current option availability: values that are no longer available are flagged and the customer is asked to choose again for those steps. Saved configurations are shareable by link; the link carries only the configuration ID and the recipient must reconstruct the price from current data, so a shared link never freezes a price. The account area lists saved configurations with a thumbnail from the last render.

## Open questions

Whether accessories should be offered as a final step inside the configurator or as a cross-sell on the basket page is unresolved and is the main point of the review scheduled for 2026-08-19. The rendering service's rate limit under a sale-day load has not been tested, which is tracked in EAICOMM-131. Analytics events follow the pattern set in "FSD - Express Checkout" and are listed in the appendix of the working draft.
