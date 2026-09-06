---
title: "BA writing standards"
space: EAICOMM
page_id: "4718002"
date: 2026-05-27
status: Current
owner: Priya Raman (BA)
---

# BA writing standards

## Why the standards exist

Every story, bug and specification on this programme is read by people who did not write it: Northwind stakeholders who approve scope, developers who estimate and build, QA who derives tests, and, months later, whoever is asked why something behaves the way it does. Consistent structure lets each of those readers find what they need without reading everything, and it lets the knowledge tools that index this space return a useful answer. These standards apply to Jira tickets in EAICOMM and to every page in this Confluence space that carries a status. They are short on purpose; a standard nobody remembers is not a standard.

## Story titles

A story title has the form `[Layer] | [Module] | [Action]`. Layer is one of Storefront, Integration, Salesforce Core, Data, or Ops. Module is the functional area in customer language: Checkout, Account, PDP, Basket, Search, Content, Orders, Reporting. Action is a verb phrase describing what the user or system can do when the story is done, written in the present tense and without the word "should". Examples: `Storefront | Checkout | Place an express order from the cart page`, `Integration | Orders | Export express orders to OMS with the checkout mode attribute`, `Salesforce Core | Reporting | Record engagement health weekly`. A bug title uses the same three parts with the action replaced by the observed failure: `Storefront | Account | Saved card list shows an expired card as active`.

## Story body

The body has four headings in this order: Context, which says why the story exists in two or three sentences and links the specification page by title; Acceptance criteria, which is a numbered list of observable outcomes, each one testable on its own and written in plain prose; Out of scope, which lists what a reasonable reader might assume is included but is not; and Technical notes, which the BA leaves blank. Technical notes are filled by the developer who picks up the story, with the approach, the cartridges or components touched, and any data change, and they are updated if the approach changes during the sprint. The BA does not write technical notes because the BA does not decide the approach; leaving the heading in place and empty is the signal that the section belongs to the developer.

## No Gherkin

Acceptance criteria are written as prose sentences, not as Given/When/Then scenarios. Gherkin was tried in the first two sprints and produced criteria that were long, repetitive, and read by nobody at Northwind; the QA lead derives test cases from prose at least as well, and stakeholders read prose. A criterion states a condition and an outcome in one sentence: "An authenticated customer with an active saved card and a default address sees the express panel on the cart page." Where a criterion needs data, the data is named in the sentence or attached as a table, never embedded as a scenario outline.

## Specification pages

A specification page carries a frontmatter block with title, the space key, the page ID, a date, a status of Draft, In review, Approved or Superseded, and an owner. It uses second-level headings for its sections, each section is a self-contained paragraph or set of paragraphs that makes sense when read alone, and it links other pages by their exact title so that a search for the title finds the reference. A page that is superseded says by which page, and is not deleted, because tickets link to it. Open questions are a section, not scattered comments, and each open question names who will answer it and, if known, when.

## Comments and status on tickets

A comment on a ticket records a decision, a question, or a finding, and starts with which of the three it is. A decision names who made it. Status transitions carry a comment when they are not the obvious next step: a story moved back from In review to In progress says what was found. Sprint and assignee are always set; an unassigned story in an active sprint is a grooming failure, not a developer's problem.
