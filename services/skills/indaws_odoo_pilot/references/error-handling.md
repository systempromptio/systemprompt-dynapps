# Error Handling & Troubleshooting

## Error Handling

All scripts follow consistent error handling:

1. **Exit Codes**:
   - `0` = Success
   - `1` = Error (check stderr for message)

2. **Output Streams**:
   - `stdout` = Data output (JSON, IDs, results)
   - `stderr` = Status messages and errors

3. **Error Messages**: Always include context and suggested solutions

```bash
# Capture both output and errors
RESULT=$(./scripts/search_records.sh res.partner '[]' 2>&1)
EXIT_CODE=$?

if [[ $EXIT_CODE -ne 0 ]]; then
    echo "Error occurred: $RESULT"
fi
```

## Troubleshooting

### Authentication Fails
```
Error: Authentication failed: Access Denied
```
**Solutions**:
- Verify credentials are correct
- Check if API keys are enabled (Odoo >= 13.0)
- Ensure user has API access permissions

### Module Not Found
```
Error: Module 'xyz' not found
```
**Solutions**:
- Check module name spelling
- Update module list: Apps > Update Apps List
- Verify module is available in your Odoo version

### Permission Denied
```
Error: You do not have permission to...
```
**Solutions**:
- Grant appropriate access rights to user
- Check user groups and permissions
- Use admin user for testing

### Connection Timeout
```
Error: Could not connect to Odoo instance
```
**Solutions**:
- Verify ODOO_URL is correct
- Check network connectivity
- Ensure Odoo instance is running

## Common Odoo Models

| Model | Description | Common Methods |
|-------|-------------|----------------|
| `res.partner` | Contacts/Companies | search_read, create, write |
| `res.users` | Users | create, write, has_group |
| `sale.order` | Sales Orders | action_confirm, action_cancel |
| `purchase.order` | Purchase Orders | button_confirm, button_cancel |
| `product.product` | Products | search_read, create, write |
| `product.template` | Product Templates | create, write |
| `account.move` | Invoices/Journal Entries | action_post, button_draft |
| `stock.picking` | Inventory Transfers | action_confirm, button_validate |
| `ir.module.module` | Modules | button_immediate_install |
| `ir.config_parameter` | System Parameters | get_param, set_param |

## Security Best Practices

1. **API Keys**: Use API keys instead of passwords (Odoo >= 13.0)
   - Navigate to: Settings > Users & Companies > Users > API Keys

2. **Dedicated Users**: Create integration users with minimal permissions

3. **Environment Variables**: Never commit credentials to version control

4. **HTTPS Only**: Always use HTTPS in production

5. **Audit Logging**: Monitor API access through Odoo audit logs

6. **Rate Limiting**: Implement delays between bulk operations
