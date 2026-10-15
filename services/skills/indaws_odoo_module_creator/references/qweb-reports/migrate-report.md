# Example: Migrate a Custom Invoice Report from v16 to v19

## Scenario

A custom module inherits `account.report_invoice_document` to add a discount column and custom title. It works in v16 but breaks in v19.

## Original v16 code (BROKEN in v19)

```xml
<template id="custom_invoice" inherit_id="account.report_invoice_document">
    <!-- 1. Custom title (BREAKS in v18+: h2 no longer exists) -->
    <xpath expr="//div[hasclass('page')]/h2" position="replace">
        <h2>CUSTOM INVOICE <span t-field="doc.name"/></h2>
    </xpath>

    <!-- 2. Discount column header (OK — o_main_table is stable) -->
    <xpath expr="//table[hasclass('o_main_table')]//thead//tr/th[last()]" position="after">
        <th class="text-right">Discount</th>  <!-- BREAKS: text-right → text-end in BS5 -->
    </xpath>

    <!-- 3. Discount column cell -->
    <xpath expr="//table[hasclass('o_main_table')]//tbody//tr/td[last()]" position="after">
        <td class="text-right">  <!-- BREAKS: text-right -->
            <span t-field="line.discount"/>%
        </td>
    </xpath>

    <!-- 4. Tax totals (BREAKS in v18: template changed) -->
    <xpath expr="//t[@t-call='account.document_tax_totals']" position="replace">
        <t t-call="account.document_tax_totals">
            <t t-set="tax_totals" t-value="doc.tax_totals"/>
            <!-- Missing currency in v18+ -->
        </t>
    </xpath>
</template>
```

## Step 1: Run detection script

```bash
bash detect_qweb_issues.sh /path/to/my_module 19
```

Output:
```
--- [1] Bootstrap 4→5 classes ---
  report_templates.xml:5: text-right
  report_templates.xml:11: text-right
--- [4] layout_document_title ---
  report_templates.xml:2: expr.*//h2
--- [3] Tax totals ---
  report_templates.xml:16: WARNING: Missing currency param in v18
```

## Step 2: Fix each issue

### Fix 1: Title (v18+ uses layout_document_title)

```xml
<!-- BEFORE (v16) -->
<xpath expr="//div[hasclass('page')]/h2" position="replace">
    <h2>CUSTOM INVOICE <span t-field="doc.name"/></h2>
</xpath>

<!-- AFTER (v19) -->
<xpath expr="//t[@t-set='layout_document_title']" position="replace">
    <t t-set="layout_document_title">
        CUSTOM INVOICE <span t-field="doc.name"/>
    </t>
</xpath>
```

### Fix 2: Bootstrap 5 classes

```xml
<!-- BEFORE -->
<th class="text-right">Discount</th>
<td class="text-right">

<!-- AFTER -->
<th class="text-end">Discount</th>
<td class="text-end">
```

### Fix 3: Tax totals (v18+ needs currency)

```xml
<!-- BEFORE (v16) -->
<t t-call="account.document_tax_totals">
    <t t-set="tax_totals" t-value="doc.tax_totals"/>
</t>

<!-- AFTER (v19) -->
<t t-call="account.document_tax_totals">
    <t t-set="tax_totals" t-value="doc.tax_totals"/>
    <t t-set="currency" t-value="doc.currency_id"/>
</t>
```

## Final migrated code (v19)

```xml
<template id="custom_invoice" inherit_id="account.report_invoice_document">
    <!-- 1. Custom title (v18+ style) -->
    <xpath expr="//t[@t-set='layout_document_title']" position="replace">
        <t t-set="layout_document_title">
            CUSTOM INVOICE <span t-field="doc.name"/>
        </t>
    </xpath>

    <!-- 2. Discount column header -->
    <xpath expr="//table[hasclass('o_main_table')]//thead//tr/th[last()]" position="after">
        <th class="text-end">Discount</th>
    </xpath>

    <!-- 3. Discount column cell -->
    <xpath expr="//table[hasclass('o_main_table')]//tbody//t[@t-else='']//tr/td[last()]" position="after">
        <td class="text-end">
            <span t-field="line.discount"/>%
        </td>
    </xpath>

    <!-- 4. Tax totals with currency (v18+ requirement) -->
    <xpath expr="//t[@t-call='account.document_tax_totals']" position="replace">
        <t t-call="account.document_tax_totals">
            <t t-set="tax_totals" t-value="doc.tax_totals"/>
            <t t-set="currency" t-value="doc.currency_id"/>
        </t>
    </xpath>
</template>
```

## Changes summary

| Issue | v16 | v19 | Severity |
|-------|-----|-----|----------|
| Title XPath | `//div[hasclass('page')]/h2` | `//t[@t-set='layout_document_title']` | CRITICAL |
| Bootstrap class | `text-right` | `text-end` | HIGH |
| Tax totals | No currency param | `currency` required | HIGH |
| XPath for cells | `//tbody//tr/td` | `//tbody//t[@t-else='']//tr/td` | MEDIUM |
