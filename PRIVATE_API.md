# Lemmy Private API Documentation

## Overview

The Lemmy Private API provides secure endpoints for external membership systems to manage user registration programmatically. These endpoints bypass normal registration restrictions (captcha, application requirements, registration mode) and are protected by a shared secret.

## Security

- **Authentication**: All endpoints require a shared secret (API key) for authentication
- **Constant-time comparison**: Secret validation uses constant-time comparison to prevent timing attacks
- **HTTPS Required**: Always use HTTPS in production to protect the shared secret in transit
- **Rate Limiting**: Endpoints are rate-limited to prevent abuse

## Configuration

### Environment Variable (Recommended)
```bash
export LEMMY_PRIVATE_API_SECRET="your-secure-secret-here"
```

### Configuration File
Add to your Lemmy configuration file (e.g., `config/config.hjson`):
```hjson
{
  private_api_secret: "your-secure-secret-here"
}
```

### Security Best Practices
- Use a cryptographically strong secret (minimum 32 characters)
- Store the secret securely (environment variables, secrets manager)
- Never commit the secret to version control
- Rotate the secret periodically
- Use different secrets for different environments (dev/staging/prod)

## API Endpoints

### 1. Check Email Registration Status

Check if an email address is already registered in Lemmy.

**Endpoint**: `POST /api/v3/user/check_email`

**Request Headers**:
```
Content-Type: application/json
```

**Request Body**:
```json
{
  "email": "user@example.com",
  "api_secret": "your-secure-secret-here"
}
```

**Response (Email Not Registered)**:
```json
{
  "exists": false,
  "person_id": null,
  "username": null
}
```

**Response (Email Registered)**:
```json
{
  "exists": true,
  "person_id": 123,
  "username": "existing_user"
}
```

**Error Responses**:
- `401 Unauthorized`: Invalid or missing API secret
- `400 Bad Request`: Invalid request format
- `500 Internal Server Error`: Server error

**Example (curl)**:
```bash
curl -X POST https://your-lemmy-instance.com/api/v3/user/check_email \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "api_secret": "your-secure-secret-here"
  }'
```

**Example (Python)**:
```python
import requests

response = requests.post(
    "https://your-lemmy-instance.com/api/v3/user/check_email",
    json={
        "email": "user@example.com",
        "api_secret": "your-secure-secret-here"
    }
)

data = response.json()
if data["exists"]:
    print(f"Email registered to user: {data['username']}")
else:
    print("Email not registered")
```

**Example (JavaScript/Node.js)**:
```javascript
const response = await fetch('https://your-lemmy-instance.com/api/v3/user/check_email', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    email: 'user@example.com',
    api_secret: 'your-secure-secret-here'
  })
});

const data = await response.json();
console.log(data.exists ? `User: ${data.username}` : 'Not registered');
```

---

### 2. Privileged User Registration

Register a new user with automatic approval and email verification, bypassing all registration restrictions.

**Endpoint**: `POST /api/v3/user/privileged_register`

**Request Headers**:
```
Content-Type: application/json
```

**Request Body**:
```json
{
  "username": "newuser",
  "password": "SecurePassword123!",
  "email": "user@example.com",
  "api_secret": "your-secure-secret-here"
}
```

**Field Requirements**:
- `username`: 3-20 characters, alphanumeric and underscores only
- `password`: 10-60 characters (enforced by Lemmy)
- `email`: Valid email format (required)
- `api_secret`: Must match configured secret

**Response (Success)**:
```json
{
  "jwt": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "registration_created": false,
  "verify_email_sent": false
}
```

**Response Fields**:
- `jwt`: Authentication token for immediate login (always present on success)
- `registration_created`: Always `false` for privileged registration
- `verify_email_sent`: Always `false` (email is auto-verified)

**Error Responses**:
- `401 Unauthorized`: Invalid or missing API secret
- `400 Bad Request`:
  - Username already exists
  - Email already registered
  - Invalid username format
  - Password too short/long
  - Invalid request format

**Example (curl)**:
```bash
curl -X POST https://your-lemmy-instance.com/api/v3/user/privileged_register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "password": "SecurePassword123!",
    "email": "user@example.com",
    "api_secret": "your-secure-secret-here"
  }'
```

**Example (Python)**:
```python
import requests

response = requests.post(
    "https://your-lemmy-instance.com/api/v3/user/privileged_register",
    json={
        "username": "newuser",
        "password": "SecurePassword123!",
        "email": "user@example.com",
        "api_secret": "your-secure-secret-here"
    }
)

if response.status_code == 200:
    data = response.json()
    jwt_token = data["jwt"]
    print(f"User created successfully. JWT: {jwt_token}")
else:
    print(f"Registration failed: {response.json()}")
```

**Example (JavaScript/Node.js)**:
```javascript
const response = await fetch('https://your-lemmy-instance.com/api/v3/user/privileged_register', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    username: 'newuser',
    password: 'SecurePassword123!',
    email: 'user@example.com',
    api_secret: 'your-secure-secret-here'
  })
});

if (response.ok) {
  const data = await response.json();
  console.log('User created. JWT:', data.jwt);
} else {
  const error = await response.json();
  console.error('Registration failed:', error);
}
```

---

## Typical Integration Flow

### Complete Registration Workflow

1. **Check if user already exists**:
   ```
   POST /api/v3/user/check_email
   ```

2. **If email not registered, create user**:
   ```
   POST /api/v3/user/privileged_register
   ```

3. **Use returned JWT for authenticated requests** (optional):
   The JWT can be used to make authenticated API calls on behalf of the user.

### Example Complete Flow (Python)

```python
import requests

LEMMY_URL = "https://your-lemmy-instance.com"
API_SECRET = "your-secure-secret-here"

def check_and_create_user(username, email, password):
    # Step 1: Check if email exists
    check_response = requests.post(
        f"{LEMMY_URL}/api/v3/user/check_email",
        json={"email": email, "api_secret": API_SECRET}
    )

    if check_response.status_code != 200:
        return {"error": "Failed to check email status"}

    check_data = check_response.json()

    # Step 2: If email exists, return existing user info
    if check_data["exists"]:
        return {
            "status": "existing",
            "username": check_data["username"],
            "person_id": check_data["person_id"]
        }

    # Step 3: Create new user
    register_response = requests.post(
        f"{LEMMY_URL}/api/v3/user/privileged_register",
        json={
            "username": username,
            "password": password,
            "email": email,
            "api_secret": API_SECRET
        }
    )

    if register_response.status_code != 200:
        return {"error": f"Registration failed: {register_response.json()}"}

    register_data = register_response.json()

    return {
        "status": "created",
        "jwt": register_data["jwt"]
    }

# Usage
result = check_and_create_user("newuser", "user@example.com", "SecurePassword123!")
print(result)
```

---

## What Gets Bypassed

Privileged registration **bypasses**:
- ✅ Captcha requirements
- ✅ Honeypot checks
- ✅ Registration mode (open/closed/application-required)
- ✅ Email verification (automatically verified)
- ✅ Registration application requirements

Privileged registration **still enforces**:
- ❌ Username validation (format, length, slurs)
- ❌ Password strength requirements (10-60 characters)
- ❌ Duplicate username/email checks
- ❌ Rate limiting

---

## User Account Details

Users created via privileged registration:
- ✅ Email is **automatically verified** (`email_verified: true`)
- ✅ Application is **automatically accepted** (`accepted_application: true`)
- ✅ Can **login immediately** with returned JWT
- ✅ Have all default user settings applied
- ✅ Are assigned the local instance's default language(s)
- ❌ Are **not** administrators (for security)
- ✅ Have access to all standard Lemmy features

---

## Error Handling

### Common Error Scenarios

| Error Code | Error Type | Description | Solution |
|------------|------------|-------------|----------|
| 401 | `incorrect_login` | Invalid API secret | Verify `LEMMY_PRIVATE_API_SECRET` is correct |
| 400 | `private_api_secret_not_configured` | Secret not set on server | Configure `LEMMY_PRIVATE_API_SECRET` |
| 400 | `user_already_exists` | Username taken | Choose different username |
| 400 | `email_already_exists` | Email already registered | Use different email or check existing user |
| 400 | `invalid_password` | Password too short/long | Ensure password is 10-60 characters |
| 400 | `invalid_name` | Invalid username format | Use alphanumeric + underscores, 3-20 chars |
| 429 | `rate_limit_error` | Too many requests | Implement backoff/retry logic |

### Error Response Format

```json
{
  "error": "error_type_name",
  "message": "Human readable error message"
}
```

### Recommended Error Handling (Python)

```python
def safe_register_user(username, email, password):
    try:
        response = requests.post(
            f"{LEMMY_URL}/api/v3/user/privileged_register",
            json={
                "username": username,
                "password": password,
                "email": email,
                "api_secret": API_SECRET
            },
            timeout=10
        )

        if response.status_code == 200:
            return {"success": True, "data": response.json()}
        elif response.status_code == 401:
            return {"success": False, "error": "Invalid API secret"}
        elif response.status_code == 400:
            error_data = response.json()
            if "email_already_exists" in str(error_data):
                return {"success": False, "error": "Email already registered"}
            elif "user_already_exists" in str(error_data):
                return {"success": False, "error": "Username already taken"}
            else:
                return {"success": False, "error": str(error_data)}
        else:
            return {"success": False, "error": f"Unexpected error: {response.status_code}"}

    except requests.exceptions.Timeout:
        return {"success": False, "error": "Request timed out"}
    except requests.exceptions.RequestException as e:
        return {"success": False, "error": f"Network error: {str(e)}"}
```

---

## Rate Limiting

Both endpoints are rate-limited:
- **check_email**: Standard message rate limit (typically ~60 requests/minute)
- **privileged_register**: Registration rate limit (typically ~6 requests/minute)

Exceeding rate limits returns:
```json
{
  "error": "rate_limit_error"
}
```

Implement exponential backoff when encountering rate limits.

---

## Security Considerations

### Secret Management
- **Never** log the API secret
- **Never** expose the secret in client-side code
- **Never** commit the secret to version control
- Use environment variables or secrets management systems
- Rotate secrets periodically (coordinate with Lemmy admin)

### Input Validation
Always validate user input before sending to Lemmy:
- Sanitize username (alphanumeric + underscore only)
- Validate email format
- Enforce password complexity on your side
- Validate data length limits

### HTTPS Only
**Always** use HTTPS in production. The API secret is sent in the request body, so HTTP would expose it to interception.

### Audit Logging
Log all privileged registration attempts (success and failure) for security auditing:
```python
import logging

logging.info(f"Privileged registration attempt for {email} - {status}")
```

---

## Testing

### Test Endpoints

Use a test Lemmy instance with a test API secret:

```bash
# Set test secret
export LEMMY_PRIVATE_API_SECRET="test-secret-12345"

# Test email check (should be false)
curl -X POST http://localhost:8536/api/v3/user/check_email \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "api_secret": "test-secret-12345"}'

# Test user creation
curl -X POST http://localhost:8536/api/v3/user/privileged_register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "TestPassword123!",
    "email": "test@example.com",
    "api_secret": "test-secret-12345"
  }'

# Test email check again (should be true)
curl -X POST http://localhost:8536/api/v3/user/check_email \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "api_secret": "test-secret-12345"}'

# Test wrong secret (should fail with 401)
curl -X POST http://localhost:8536/api/v3/user/check_email \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "api_secret": "wrong-secret"}'
```

### Integration Test Checklist

- [ ] Secret validation works (wrong secret returns 401)
- [ ] Email check returns correct status
- [ ] User creation succeeds with valid data
- [ ] Duplicate email is rejected
- [ ] Duplicate username is rejected
- [ ] Password validation works (too short/long)
- [ ] Username validation works (invalid chars)
- [ ] Created user can login with returned JWT
- [ ] Email is auto-verified (check via Lemmy UI)
- [ ] Rate limiting triggers correctly

---

## Troubleshooting

### "private_api_secret_not_configured" error
**Problem**: Server returns error saying secret is not configured.

**Solution**: Ensure `LEMMY_PRIVATE_API_SECRET` environment variable is set or `private_api_secret` is in config file, then restart Lemmy.

### "incorrect_login" error
**Problem**: Valid-looking secret is rejected.

**Solution**:
- Verify secret exactly matches server configuration
- Check for trailing whitespace or special characters
- Ensure secret is URL-safe (no encoding issues)

### User created but can't login
**Problem**: Privileged registration succeeds but user cannot login normally.

**Solution**: This should not happen as privileged registration auto-verifies email and accepts application. Check Lemmy logs for errors.

### Rate limit errors
**Problem**: Getting rate limited frequently.

**Solution**:
- Implement request queuing/throttling on your side
- Use exponential backoff when rate limited
- Consider caching email check results

---

## Changelog

### Version 0.19.13+
- Initial implementation of Private API
- Added `/api/v3/user/check_email` endpoint
- Added `/api/v3/user/privileged_register` endpoint
- Constant-time secret comparison for security

---

## Support

For issues or questions:
- File an issue on the Lemmy GitHub repository
- Contact your Lemmy instance administrator
- Check Lemmy Matrix chat for developer support
