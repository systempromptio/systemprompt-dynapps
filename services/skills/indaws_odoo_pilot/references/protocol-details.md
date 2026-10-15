# Protocol Details

## Version-Based Protocol Selection

- **Odoo >= 19.0**: MUST use JSON2
- **Odoo <= 18.0**: MUST use JSON-RPC

### JSON-RPC (Odoo <= 18.0)
- **Use for**: Odoo versions 8.0 through 18.0
- **Endpoint**: `/jsonrpc` and `/web/session/authenticate`
- **Session Management**: Required (cookie-based)
- **Authentication**: Must authenticate once, then use session cookie
- **Structure**: Uses `execute_kw` wrapper

```bash
# For Odoo <= 18.0 only
eval $(./scripts/auth_jsonrpc.sh)

# Session persists via ODOO_SESSION_ID
./scripts/search_records.sh res.partner '[]'
```

### JSON2 (Odoo >= 19.0)
- **Use for**: Odoo versions 19.0 and later
- **Endpoint**: `/json/2/{model}/{method}` (NOT `/json/2/call`)
- **Session Management**: Not required (stateless)
- **Authentication**: Bearer token in `Authorization` header
- **Structure**: Direct parameters in body, no JSON-RPC wrapper
- **Headers**: `Authorization: bearer {API_KEY}`, `X-Odoo-Database: {DB_NAME}`

```bash
# For Odoo >= 19.0 only
eval $(./scripts/auth_json2.sh)

# API key automatically included in each request via Bearer token
./scripts/search_records.sh res.partner '[]'
```

**Recommended**: Use `auth.sh` for automatic protocol selection based on Odoo version.

**Important**: For detailed JSON2 format documentation, see:
- `JSON2_ENDPOINT_FORMAT.md` - Complete endpoint reference with examples
- `PROTOCOL_GUIDE.md` - Protocol comparison and migration guide
- `EXAMPLES_JSON2.md` - Practical usage examples
- `CHANGELOG_JSON2.md` - Implementation details and fixes

## API Documentation References

### Official Odoo Documentation
- **JSON2 API (Odoo 19.0+)**: https://www.odoo.com/documentation/19.0/developer/reference/external_api.html
- **JSON-RPC (Legacy)**: https://www.odoo.com/documentation/18.0/developer/howtos/web_services.html
- **Model Reference**: https://www.odoo.com/documentation/18.0/developer/reference/backend/orm.html

### Skill Documentation Files
- **`JSON2_ENDPOINT_FORMAT.md`**: Comprehensive JSON2 endpoint format reference with curl examples
- **`PROTOCOL_GUIDE.md`**: Complete protocol comparison, migration guide, and troubleshooting
- **`EXAMPLES_JSON2.md`**: Practical examples for CRUD operations, module management, and workflows
- **`CHANGELOG_JSON2.md`**: Technical details of JSON2 implementation and fixes applied
