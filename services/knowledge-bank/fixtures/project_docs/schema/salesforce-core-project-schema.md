---
title: "Salesforce Core - engagement reporting schema"
space: EAICOMM
page_id: "4719260"
date: 2026-06-11
status: Current
owner: Jonas Kraft (Salesforce Core)
---

# Salesforce Core - engagement reporting schema

## Purpose

The Astound Digital Salesforce org carries a small set of custom objects that describe every client engagement, its statements of work, its staffing and its health, so that the account team, delivery leadership and finance read the same data. The storefront programme for Northwind Trading is one such engagement. This page lists the objects, their API names and the fields the delivery team is expected to keep current; everything else on the org belongs to the sales and finance teams and is documented in their own spaces.

## Engagement__c

`Engagement__c` is one record per client engagement, parented to the standard `Account` through the lookup `Client__c`. Key fields: `Name` (the engagement name as it appears on the SOW), `Engagement_Manager__c` (lookup to `User`), `Technical_Architect__c` (lookup to `User`), `Status__c` (picklist: Prospect, Active, On hold, Closed), `Start_Date__c` and `Planned_End_Date__c`, `Health__c` (picklist: Green, Amber, Red, set weekly by the engagement manager), `Health_Note__c` (long text, required when Health is not Green), `Platform__c` (multi-select picklist: SFCC, Storefront Next, Salesforce Core, Marketing Cloud, OMS, Other), and `Jira_Project_Key__c` (text, the key of the delivery project, EAICOMM for Northwind). The record type `Delivery` is the only one the delivery team creates.

## SOW_Line__c

`SOW_Line__c` is a child of `Engagement__c` via the master-detail `Engagement__c` and records one line of a signed statement of work: `Name` (the line description), `SOW_Reference__c` (the document number), `Line_Type__c` (picklist: Fixed price, Time and materials, Retainer, Change request), `Value__c` (currency), `Currency__c` (picklist matching the org's enabled currencies), `Signed_Date__c`, `Delivery_Start__c`, `Delivery_End__c`, `Percent_Complete__c` (percent, set at sprint review by the engagement manager), and `Invoiced_To_Date__c` (currency, rolled up from finance's invoice object and read-only to delivery). A change request is always a new `SOW_Line__c` with `Line_Type__c` = Change request, never an edit to the original line.

## Staffing_Assignment__c

`Staffing_Assignment__c` links a `User` to an `Engagement__c` for a period with a role: `Consultant__c` (lookup to `User`), `Engagement__c` (master-detail), `Role__c` (picklist: Engagement Manager, Business Analyst, Technical Architect, Developer, QA, Salesforce Core, Integration), `Allocation_Percent__c`, `Start_Date__c`, `End_Date__c`, and `Billable__c` (checkbox). Resourcing reads this object to plan capacity, and finance reads it to check that time booked against an engagement comes from someone assigned to it, so an assignment must exist before the first timesheet is entered.

## Engagement_Risk__c

`Engagement_Risk__c` is the RAID log: `Engagement__c` (master-detail), `Type__c` (picklist: Risk, Assumption, Issue, Dependency), `Title__c`, `Description__c`, `Probability__c` and `Impact__c` (picklists Low, Medium, High), `Owner__c` (lookup to `User`), `Status__c` (Open, Mitigating, Closed), `Raised_Date__c`, `Target_Date__c`, and `Mitigation__c`. The steering committee report is a report on this object filtered to Open and Mitigating, so a risk that is not on the record is not on the agenda. Closing a risk requires a `Closure_Note__c`.

## Conventions

API names end in `__c` and use PascalCase with underscores; picklist values are sentence case; every object has a `Description` explaining its purpose to the next administrator; and no formula field references a field on another object more than one hop away, to keep the reporting snapshot fast. Changes to these objects are made by the Salesforce Core team through the metadata pipeline, never by hand in production, and are announced in the delivery channel before deployment because the weekly health update depends on them.
