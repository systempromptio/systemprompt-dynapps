---
title: "Client brief - Meridian Pharma Supply"
account: Meridian Pharma Supply B.V.
industry: Pharmaceutical wholesale and B2B distribution
status: Prospect - proposal
owner: Miriam Okafor
date: 2026-08-14
last_touch: 2026-08-07
---

# Client brief - Meridian Pharma Supply

## Company and industry

Meridian Pharma Supply is a Rotterdam-based wholesaler distributing pharmaceuticals, medical consumables and cold-chain products to pharmacies, hospitals and clinics in the Benelux. The business is regulated at every step: customer eligibility depends on licence type, some products may only be sold to certain licence classes, and every order must be traceable. Their current ordering portal is a decade-old custom application that their pharmacists tolerate and their IT department fears. The commercial driver is a B2B storefront that respects the licence rules automatically and integrates with their serialisation and cold-chain systems.

## Platforms

Today a custom Java portal with an Oracle back end, an ERP that owns pricing per customer contract, and a serialisation system that must be queried before shipment. The proposal is Salesforce Commerce Cloud B2B with a Storefront Next front end, customer groups derived from licence classes, contract pricing from the ERP, and Salesforce Core for the customer master and the licence records. Nothing has been built; this is a proposal-stage account.

## Key contacts

Dr Annelies de Vries, Managing Director, decision maker and the person who must be convinced that a platform can encode regulatory rules. Pieter Jansen, Head of IT, sceptical of packaged platforms after a failed ERP project and the person the proposal must win over. Sofia Marín, Head of Sales Operations, owner of the customer contracts and the pricing rules and the most enthusiastic contact. Astound Digital's lead is Miriam Okafor with Ana Lucia Ferreira on the solution design and Jonas Kraft on the Salesforce Core licence model.

## Engagement scope

A proposal for a discovery phase of eight weeks producing a solution design, a licence-rule model on Salesforce Core, a prototype storefront showing licence-aware catalogue visibility, and a costed roadmap for the build. The discovery is fixed price; the build would be a separate SOW. The proposal was sent on 2026-08-07 and a decision is expected at Meridian's board meeting on 2026-09-03.

## Open opportunities

The build itself, estimated at fourteen months and the largest opportunity in the pipeline. A managed service for the storefront after go-live, which Pieter would prefer to owning it. A Service Cloud implementation for their pharmacist support desk, mentioned by Sofia as a separate budget.

## Risks

Pieter's scepticism could stall the board decision; the prototype in the discovery must be real, not slides. The licence rules are more complex than the RFP describes, with exceptions per province, and discovery must surface them all. Cold-chain shipping rules affect which shipping methods a customer sees and the reference application has no notion of product-level shipping constraints, so the design will need an extension pattern that is documented from day one.
