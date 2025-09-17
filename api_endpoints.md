# Auth API Design

---

## Utils

### GET
- `/utils/version`
- `/utils/health`

### POST
- `/utils/config`

**Request:**
```json
{
  "the param name + new value you want to change": "Rate Limits - да пазя таблица с rate limits за всеки endpoint?"
}
```

**Response:**
```http
201 Created
```

---

## Onboarding

### POST
- `/onboarding/prepare-session` (WEB ONLY)

**Request:**
```json
{
  "os": "",
  "os_version": "",
  "locale": "",
  "device_type": "",
  "app_version": "",
  "client": {
    "ip": "",
    "browser_name": "",
    "browser_version": "",
    "country": "",
    "<extra data>": ""
  }
}
```

**Response:**
- **200 OK**
```json
{
  "device_id": "if the device is already in the db",
  "cookie": "JWT (pre-token). Nonce stored in Redis"
}
```

- **429 Too Many Requests**
```json
{
  "error": {
    "trace_id": "",
    "description": ""
  }
}
```

**Outcome:**
- If the user does not start auth in **3 minutes**, they need to restart.
- Time limit exists for login/register.
- Rate limits: If the same IP comes **more than 10 times**, wait **5 minutes**.
- Redis decides whether to enforce **2FA** on success (hot config possible).

---

## Auth

### POST
- `/auth`

**Headers:** `<HEADERS>`

**Request:**
```json
{
  "method": "email (in the future phone_number)",
  "email": ""
}
```

Check pre-session and nonce in Redis.

**Response:**
```json
{
  "next_step": "two_fa | show_failure | go_to_register | go_to_login | block",
  "cookie": "JWT (pre-pass-token). Previous cookie is invalid."
}
```

---

- `/auth/two-fa`
- `/auth/login`
- `/auth/register`  
  *(Cookie required to confirm email sending and 2FA flow)*

---

## Users

### POST
- `/users`

### PUT
- `/users/{id}`

### DELETE
- `/users/{id}`

---

## Audit

### POST
- `/audit/login`
- `/audit/key-usage`
- `/audit/cert-usage`
- `/audit/interaction`
- `/audit/event`

---

## Recovery

### GET
- `/recovery?nonce=<some nonce>`  
  *(returned after registration + 2FA to get recovery codes, along with session)*

### POST
- `/recovery`

---

## Devices

### POST
- `/devices`
- `/devices/{id}/revoke-access`
- `/devices/{id}/grant-device`

### DELETE
- `/devices/{id}`

---

## Admin

### GET
- `/admin/users?...`
- `/admin/users/{id}/info`
- `/admin/users/{id}/sessions?...`
- `/admin/users/{id}/interactions?...`

### POST
- `/admin/users/report?...`  
  *(Generate AI report of user interactions, find anomalies)*

### PUT
- `/admin/users/{id}/revoke`
- `/admin/users/{id}/devices/{device_id}/revoke`

### DELETE
- `/admin/users/{id}/delete`
- `/admin/users/{id}/devices/{device_id}/delete`
