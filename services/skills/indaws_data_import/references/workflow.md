# Workflow (Mandatory Sequence)

## STEP 1: Information Gathering

**NEVER proceed without ALL required information.**

Use `AskUserQuestion` to gather:

### Priority 1: Basic Info
1. **What needs to be imported?**
   - Model: products, contacts, invoices, etc.
   - Technical model name: `product.product`, `res.partner`, etc.

2. **Data source?**
   - CSV file
   - Excel file (XLS/XLSX)
   - JSON file
   - External API
   - SFTP/FTP server
   - Database connection

3. **Import frequency?**
   - One-time manual import
   - Scheduled (daily, weekly, monthly)
   - Triggered by external event

4. **Data volume?**
   - < 1,000 records (simple)
   - 1,000 - 100,000 records (medium)
   - > 100,000 records (large, needs chunking)

### Priority 2: Technical Details
5. **Odoo version?**
   - 14.0, 15.0, 16.0, 17.0, 18.0, 19.0

6. **Odoo instance details?**
   - URL, database, credentials
   - Or local development path

7. **File structure?** (if file-based)
   - Sample file or file structure
   - Column names
   - Data types

8. **Transformation needs?**
   - Field mappings (source -> Odoo field)
   - Data transformations needed?
   - Validations required?

### Priority 3: Advanced Features (if needed)
9. **Matching strategy?**
   - Match by external ID
   - Match by name
   - Match by code/reference
   - Custom matching logic

10. **Error handling?**
    - Continue on error vs stop on first error
    - Error notifications?

**STOP HERE** - Wait for user's response before continuing.

---

## STEP 2: Select Use Case

Based on gathered information, select the appropriate use case guide:

| User Needs | Recommended Use Case |
|------------|---------------------|
| "Import records from CSV/Excel" | [01-simple-csv-import.md](../use-cases/01-simple-csv-import.md) |
| "Daily import from API" | [04-api-data-collector.md](../use-cases/04-api-data-collector.md) |

**Announce the selected use case to the user and follow its guide.**

---

## STEP 3: Follow Use Case Guide

Each use case guide contains:
1. **Prerequisites check**
2. **Template creation steps**
3. **Field mapping configuration**
4. **Python code examples** (if needed)
5. **Testing instructions**
6. **Troubleshooting**

---

## STEP 4: Validation & Testing

After creating the import template:

1. **Verify template structure:**
   ```python
   template = env['indaws.import.template'].search([('name', '=', 'Template Name')])
   print(f"Model: {template.model_id.name}")
   print(f"Fields mapped: {len(template.line_ids)}")
   print(f"Match fields: {template.match_line_ids.mapped('field_id.name')}")
   ```

2. **Test with sample data:**
   - Create small test file (5-10 records)
   - Run import manually
   - Verify results

3. **Check for errors:**
   - Review import logs
   - Check error records
   - Validate data integrity

4. **Performance test** (if large volume):
   - Test with subset of real data
   - Monitor execution time
   - Adjust chunk size if needed
