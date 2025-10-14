# CORS Configuration Guide

## Overview

The Agent Platform includes flexible CORS (Cross-Origin Resource Sharing) configuration to control which origins can access the API. This is essential for security when your frontend and backend are hosted on different domains or ports.

## Default Configuration

By default, the platform allows requests from all localhost ports, which is ideal for local development:

- `http://localhost:3000` (React default)
- `http://localhost:3001`
- `http://localhost:5173` (Vite default)
- `http://localhost:8080`
- `http://localhost:8081`
- `http://127.0.0.1:*` (same ports)

## Environment Variables

### CORS_ALLOW_ALL_LOCALHOST

Controls whether all localhost ports are automatically allowed.

```bash
# Enable (default)
CORS_ALLOW_ALL_LOCALHOST=true

# Disable
CORS_ALLOW_ALL_LOCALHOST=false
```

**When to use**:
- Set to `true` for local development
- Set to `false` for production environments

### CORS_ALLOWED_ORIGINS

Comma-separated list of additional allowed origins.

```bash
# Single origin
CORS_ALLOWED_ORIGINS=https://example.com

# Multiple origins
CORS_ALLOWED_ORIGINS=https://example.com,https://app.example.com,https://admin.example.com

# With ports
CORS_ALLOWED_ORIGINS=https://example.com:3000,https://api.example.com:8080
```

**When to use**:
- Production deployments
- Specific domain restrictions
- Multiple frontend applications

## Configuration Examples

### Local Development

```bash
# .env
CORS_ALLOW_ALL_LOCALHOST=true
CORS_ALLOWED_ORIGINS=
```

This allows all localhost requests on common ports.

### Production (Single Domain)

```bash
# .env
CORS_ALLOW_ALL_LOCALHOST=false
CORS_ALLOWED_ORIGINS=https://app.example.com
```

Only allows requests from your production frontend.

### Production (Multiple Domains)

```bash
# .env
CORS_ALLOW_ALL_LOCALHOST=false
CORS_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com,https://mobile.example.com
```

Allows requests from multiple frontend applications.

### Staging + Production

```bash
# .env
CORS_ALLOW_ALL_LOCALHOST=false
CORS_ALLOWED_ORIGINS=https://staging.example.com,https://app.example.com
```

Allows both staging and production environments.

### Development + Production

```bash
# .env
CORS_ALLOW_ALL_LOCALHOST=true
CORS_ALLOWED_ORIGINS=https://app.example.com
```

Allows both localhost (for development) and production domain.

## Allowed Methods

The following HTTP methods are allowed by default:

- `GET`
- `POST`
- `PUT`
- `DELETE`
- `PATCH`
- `OPTIONS` (for preflight requests)

## Allowed Headers

All headers are allowed by default, including:

- `Content-Type`
- `Authorization`
- `X-Requested-With`
- Custom headers

## Credentials

Credentials (cookies, authorization headers) are allowed by default. This means:

- JWT tokens in `Authorization` headers work
- Cookies are sent with requests
- Authentication is fully supported

## Testing CORS Configuration

### Using curl

```bash
# Test CORS preflight request
curl -X OPTIONS http://localhost:8080/api/health \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: GET" \
  -v

# Test actual request
curl http://localhost:8080/api/health \
  -H "Origin: http://localhost:3000" \
  -v
```

### Using Browser Console

```javascript
// Test from browser console
fetch('http://localhost:8080/api/health', {
  method: 'GET',
  headers: {
    'Content-Type': 'application/json',
  },
  credentials: 'include',
})
  .then(response => response.json())
  .then(data => console.log('Success:', data))
  .catch(error => console.error('Error:', error));
```

### Expected Response Headers

When CORS is configured correctly, you should see these headers in the response:

```
Access-Control-Allow-Origin: http://localhost:3000
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS
Access-Control-Allow-Headers: *
Access-Control-Allow-Credentials: true
```

## Troubleshooting

### CORS Error in Browser

**Error**: `Access to fetch at 'http://localhost:8080/api/...' from origin 'http://localhost:3000' has been blocked by CORS policy`

**Solutions**:

1. **Check CORS_ALLOW_ALL_LOCALHOST**:
   ```bash
   # Make sure it's enabled for local development
   CORS_ALLOW_ALL_LOCALHOST=true
   ```

2. **Add your origin to CORS_ALLOWED_ORIGINS**:
   ```bash
   CORS_ALLOWED_ORIGINS=http://localhost:3000
   ```

3. **Restart the server** after changing environment variables

4. **Check the origin matches exactly**:
   - `http://localhost:3000` ≠ `http://127.0.0.1:3000`
   - `http://localhost:3000` ≠ `https://localhost:3000`
   - Port numbers must match

### Preflight Request Fails

**Error**: `Response to preflight request doesn't pass access control check`

**Solutions**:

1. Ensure `OPTIONS` method is allowed (it is by default)
2. Check that all required headers are allowed
3. Verify credentials setting matches your request

### Credentials Not Sent

**Error**: Cookies or Authorization headers not being sent

**Solutions**:

1. Ensure `credentials: 'include'` in fetch requests:
   ```javascript
   fetch(url, {
     credentials: 'include',
     // ...
   });
   ```

2. For Axios:
   ```javascript
   axios.defaults.withCredentials = true;
   ```

3. Verify CORS allows credentials (it does by default)

## Security Best Practices

### Development

✅ **Do**:
- Use `CORS_ALLOW_ALL_LOCALHOST=true`
- Test with actual frontend application
- Use environment-specific configurations

❌ **Don't**:
- Use production credentials in development
- Commit `.env` files to version control
- Allow all origins (`*`) even in development

### Production

✅ **Do**:
- Set `CORS_ALLOW_ALL_LOCALHOST=false`
- Specify exact allowed origins
- Use HTTPS for all origins
- Regularly review allowed origins
- Use environment variables for configuration

❌ **Don't**:
- Allow all origins (`*`)
- Allow `http://` origins in production
- Include development origins
- Use wildcard subdomains unless necessary

### Example Production Configuration

```bash
# Production .env
CORS_ALLOW_ALL_LOCALHOST=false
CORS_ALLOWED_ORIGINS=https://app.example.com

# Use HTTPS
# Specific domain only
# No wildcards
# No localhost
```

## Docker Configuration

When running in Docker, ensure your frontend origin is accessible:

```yaml
# docker-compose.yml
services:
  backend:
    environment:
      CORS_ALLOW_ALL_LOCALHOST: "true"
      CORS_ALLOWED_ORIGINS: "http://frontend:3000"
```

## Kubernetes Configuration

```yaml
# k8s/deployment.yaml
env:
  - name: CORS_ALLOW_ALL_LOCALHOST
    value: "false"
  - name: CORS_ALLOWED_ORIGINS
    value: "https://app.example.com,https://admin.example.com"
```

## Advanced Configuration

### Dynamic Origins

If you need to allow origins dynamically (e.g., customer subdomains), you'll need to modify the code in `src/presentation/server.rs`:

```rust
// Example: Allow all subdomains of example.com
let cors = CorsLayer::new()
    .allow_origin(tower_http::cors::AllowOrigin::predicate(
        |origin: &HeaderValue, _request_parts: &_| {
            origin
                .to_str()
                .map(|s| s.ends_with(".example.com"))
                .unwrap_or(false)
        }
    ))
    // ... rest of configuration
```

### Per-Route CORS

For different CORS policies on different routes, apply CORS at the route level:

```rust
// In routes configuration
Router::new()
    .route("/public", get(handler))
    .layer(public_cors)
    .route("/admin", get(admin_handler))
    .layer(admin_cors)
```

## Monitoring

Check CORS configuration in logs:

```bash
# Server startup logs will show CORS configuration
docker-compose logs backend | grep -i cors
```

## References

- [MDN CORS Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
- [Tower HTTP CORS](https://docs.rs/tower-http/latest/tower_http/cors/)
- [Axum CORS Example](https://github.com/tokio-rs/axum/tree/main/examples/cors)

## Support

If you encounter CORS issues:

1. Check this documentation
2. Review server logs
3. Test with curl
4. Check browser console for specific errors
5. Open an issue with:
   - Your CORS configuration
   - The error message
   - Browser console output
   - Server logs

---

**Last Updated**: 2024-01-01  
**Version**: 1.0.0
