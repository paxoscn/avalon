# CORS Configuration - Quick Setup Summary

## What Was Added

CORS (Cross-Origin Resource Sharing) configuration has been added to allow frontend applications to communicate with the backend API from different origins.

## Quick Start

### For Local Development

No configuration needed! By default, all localhost requests are allowed:

```bash
# .env (or leave these out to use defaults)
CORS_ALLOW_ALL_LOCALHOST=true
```

This automatically allows:
- `http://localhost:3000` (React)
- `http://localhost:5173` (Vite)
- `http://localhost:8080`
- And other common ports

### For Production

Set specific allowed origins:

```bash
# .env
CORS_ALLOW_ALL_LOCALHOST=false
CORS_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com
```

## Testing

### Test from Browser Console

```javascript
fetch('http://localhost:8080/api/health')
  .then(r => r.json())
  .then(data => console.log('Success:', data))
  .catch(err => console.error('CORS Error:', err));
```

### Test with curl

```bash
curl -X OPTIONS http://localhost:8080/api/health \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: GET" \
  -v
```

## Common Issues

### "CORS policy blocked" Error

**Solution 1**: Enable localhost (for development)
```bash
CORS_ALLOW_ALL_LOCALHOST=true
```

**Solution 2**: Add your origin explicitly
```bash
CORS_ALLOWED_ORIGINS=http://localhost:3000
```

**Solution 3**: Restart the server after changing .env

### Different Port

If your frontend runs on a different port (e.g., 3001):

```bash
# Either enable all localhost
CORS_ALLOW_ALL_LOCALHOST=true

# Or add specific port
CORS_ALLOWED_ORIGINS=http://localhost:3001
```

## Files Modified

1. **src/config/mod.rs** - Added CORS configuration structure
2. **src/presentation/server.rs** - Added CORS middleware
3. **.env.example** - Added CORS environment variables
4. **docs/cors_configuration.md** - Complete CORS documentation

## Configuration Options

| Variable | Default | Description |
|----------|---------|-------------|
| `CORS_ALLOW_ALL_LOCALHOST` | `true` | Allow all localhost ports |
| `CORS_ALLOWED_ORIGINS` | - | Comma-separated list of allowed origins |

## Examples

### React App (localhost:3000)
```bash
# No configuration needed - works by default!
```

### Vite App (localhost:5173)
```bash
# No configuration needed - works by default!
```

### Production Deployment
```bash
CORS_ALLOW_ALL_LOCALHOST=false
CORS_ALLOWED_ORIGINS=https://myapp.com
```

### Multiple Frontends
```bash
CORS_ALLOW_ALL_LOCALHOST=false
CORS_ALLOWED_ORIGINS=https://app.com,https://admin.app.com,https://mobile.app.com
```

## Next Steps

1. ✅ CORS is configured and working
2. 📝 See [docs/cors_configuration.md](cors_configuration.md) for detailed documentation
3. 🔧 Adjust `.env` for your specific needs
4. 🚀 Deploy with confidence!

## Support

For detailed information, see:
- [Complete CORS Documentation](cors_configuration.md)
- [Troubleshooting Guide](troubleshooting.md)
- [Deployment Guide](deployment_guide.md)
