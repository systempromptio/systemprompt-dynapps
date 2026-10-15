# Odoo Queries — Case Study Generator

All queries use JSON2 REST: `POST /json/2/<model>/<method>` with `Authorization: Bearer <key>`.
See `references/shared/odoo-connection.md` for auth setup.

## Partner + contacts

```
/json/2/res.partner/read
ids=[<id>]
fields=[name,street,city,state_id,country_id,phone,email,website,category_id,comment,user_id]

/json/2/res.partner/search_read
domain=[["commercial_partner_id","=",<id>],["id","!=",<id>]]
fields=[id,name,email,function,phone]
```

## Sale orders + line items

```
/json/2/sale.order/search_read
domain=[["partner_id.commercial_partner_id","=",<id>]]
fields=[id,name,date_order,amount_total,state,order_line]
order="date_order asc"  limit=100

/json/2/sale.order.line/search_read
domain=[["order_partner_id.commercial_partner_id","=",<id>],["product_id","!=",false]]
fields=[id,order_id,product_id,name,product_uom_qty,price_subtotal]
limit=300
```

## Projects + tasks

```
/json/2/project.project/search_read
domain=[["partner_id.commercial_partner_id","=",<id>]]
fields=[id,name,date_start,date,stage_id,task_count]

/json/2/project.task/search_read
domain=[["partner_id.commercial_partner_id","=",<id>]]
fields=[id,name,project_id,stage_id,description]
limit=300
```

## Posted invoices

```
/json/2/account.move/search_read
domain=[["partner_id.commercial_partner_id","=",<id>],["move_type","in",["out_invoice","out_refund"]],["state","=","posted"]]
fields=[id,name,invoice_date,amount_total]
order="invoice_date asc"  limit=300
```

## Helpdesk tickets

```
/json/2/helpdesk.ticket/search_read
domain=[["partner_id.commercial_partner_id","=",<id>]]
fields=[id,name,create_date,stage_id]
order="create_date asc"  limit=300
```

## Indaws Core modules (when available)

```
# Find env record
/json/2/database.env/search_read
domain=[["partner_id","=",<id>]]
fields=[id,name,database,odoo_version,odoo_edition,module_count]

# If found, get Indaws modules
/json/2/database.env.module/search_read
domain=[["env_id","=",<env_id>],["technical_name","=like","indaws_%"]]
fields=[id,technical_name,name,version]
limit=300
```

Note: customers on odoo.com SaaS will not appear in `database.env`. This is expected — proceed using task/ticket inference instead.

## Customer resolution (Step 1)

If a name is given (not an ID):
```
POST /json/2/res.partner/search_read
domain=[["name","ilike","<name>"],["is_company","=",true]]
fields=["id","name","street","city","state_id","website"]
limit=10
```

If multiple matches, list them and ask the user to confirm before continuing.

Once resolved, note both `partner_id` and `commercial_partner_id` — always use `commercial_partner_id` as the filter anchor (catches child contacts, invoices, tickets for multi-contact companies).
