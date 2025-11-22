# Kleos Event Streaming API - Frontend Integration Guide

**Version:** 1.0
**Last Updated:** 2025-11-16
**Backend Stack:** AWS Lambda (Rust) + Kinesis Data Streams
**API Type:** HTTP API (AWS API Gateway v2)

---

## Table of Contents

1. [Overview](#overview)
2. [API Endpoint](#api-endpoint)
3. [Authentication](#authentication)
4. [API Contract](#api-contract)
5. [Request Validation Rules](#request-validation-rules)
6. [Response Formats](#response-formats)
7. [Error Handling](#error-handling)
8. [CORS Configuration](#cors-configuration)
9. [Rate Limits](#rate-limits)
10. [Example Requests](#example-requests)
11. [Architecture Flow](#architecture-flow)
12. [Testing Guide](#testing-guide)
13. [Monitoring & Observability](#monitoring--observability)

---

## Overview

Kleos is a serverless event streaming platform built on AWS that accepts user action events via HTTP API and processes them asynchronously through Kinesis Data Streams.

**What the Frontend Needs to Do:**
- Send POST requests to the `/events` endpoint
- Include valid JSON payload with event data
- Handle success (200) and error (400, 500) responses
- Display request_id to users for tracking

**What Happens Behind the Scenes:**
1. API Gateway receives the request
2. Producer Lambda validates the payload
3. Event is pushed to Kinesis Data Stream (async processing)
4. Consumer Lambda processes the event in batches
5. Failed events go to Dead Letter Queue for retry

---

## API Endpoint

### Production API
```
Base URL: https://73jumnuhth.execute-api.us-east-1.amazonaws.com
```

### Available Routes

| Method | Path      | Description              |
|--------|-----------|--------------------------|
| POST   | `/events` | Submit a user action event |

**Full Endpoint:**
```
POST https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events
```

---

## Authentication

**Current Status:** No authentication required (development mode)

**Future Considerations:**
- API keys via `x-api-key` header
- JWT bearer tokens
- AWS IAM signature (for internal services)

---

## API Contract

### Request Format

**HTTP Method:** `POST`
**Path:** `/events`
**Content-Type:** `application/json` (required)

**Request Body Schema:**

```typescript
interface EventRequest {
  user_id: string;   // 1-100 characters, trimmed, non-empty
  action: string;    // One of: "create", "update", "delete", "view" (case-insensitive)
  details: string;   // 1-1000 characters, trimmed, non-empty
}
```

**Exact JSON Structure:**
```json
{
  "user_id": "alice-123",
  "action": "create",
  "details": "User created a new resource"
}
```

### Response Format

#### Success Response (HTTP 200)

```typescript
interface EventResponse {
  request_id: string;  // UUID-like string for tracking this event
  message: string;     // Always "Event accepted"
}
```

**Exact JSON:**
```json
{
  "request_id": "req_1a2b3c4d5e6f7890",
  "message": "Event accepted"
}
```

#### Error Response (HTTP 400 or 500)

```typescript
interface ErrorResponse {
  error: string;  // Human-readable error message
}
```

**Examples:**
```json
{
  "error": "Invalid request: JSON parse error: expected value at line 1 column 1"
}
```

```json
{
  "error": "Validation failed: Invalid user_id: String too long: max 100, got 150"
}
```

```json
{
  "error": "Validation failed: Invalid action type: approve"
}
```

---

## Request Validation Rules

### Field: `user_id`

**Type:** String
**Constraints:**
- **Minimum length:** 1 character (after trimming whitespace)
- **Maximum length:** 100 characters (after trimming whitespace)
- **Trimming:** Leading and trailing whitespace is automatically removed
- **Empty check:** Cannot be empty or only whitespace

**Valid Examples:**
```json
"user_id": "alice"
"user_id": "user-12345"
"user_id": "alice@example.com"
"user_id": "  bob  "  // Will be trimmed to "bob"
```

**Invalid Examples:**
```json
"user_id": ""                    // Empty - Error: "String is empty"
"user_id": "   "                 // Only whitespace - Error: "String is empty"
"user_id": "a".repeat(101)       // Too long - Error: "String too long: max 100, got 101"
```

### Field: `action`

**Type:** String
**Constraints:**
- **Allowed values:** `"create"`, `"update"`, `"delete"`, `"view"`
- **Case sensitivity:** Case-insensitive (converted to lowercase)
- **Exact match:** Must match one of the 4 allowed values

**Valid Examples:**
```json
"action": "create"   // ✓
"action": "CREATE"   // ✓ (converted to lowercase)
"action": "Create"   // ✓ (converted to lowercase)
"action": "update"   // ✓
"action": "delete"   // ✓
"action": "view"     // ✓
```

**Invalid Examples:**
```json
"action": "approve"   // ✗ Error: "Invalid action type: approve"
"action": "read"      // ✗ Error: "Invalid action type: read"
"action": ""          // ✗ Error: "Invalid action type: "
"action": "created"   // ✗ Error: "Invalid action type: created"
```

### Field: `details`

**Type:** String
**Constraints:**
- **Minimum length:** 1 character (after trimming whitespace)
- **Maximum length:** 1000 characters (after trimming whitespace)
- **Trimming:** Leading and trailing whitespace is automatically removed
- **Empty check:** Cannot be empty or only whitespace

**Valid Examples:**
```json
"details": "User created a new project"
"details": "Updated user profile with new email address"
"details": "x".repeat(1000)  // Exactly 1000 chars - OK
```

**Invalid Examples:**
```json
"details": ""                // Empty - Error: "String is empty"
"details": "   "             // Only whitespace - Error: "String is empty"
"details": "x".repeat(1001)  // Too long - Error: "String too long: max 1000, got 1001"
```

---

## Response Formats

### HTTP Status Codes

| Status | Meaning | When It Occurs |
|--------|---------|----------------|
| 200 | Success | Event accepted and pushed to Kinesis |
| 400 | Bad Request | Invalid JSON, validation failure, or missing fields |
| 500 | Internal Server Error | Kinesis connection failure or internal error |

### Success Flow (200)

**When:** Event passes validation and is successfully pushed to Kinesis Data Stream

**Response:**
```json
{
  "request_id": "req_abc123def456",
  "message": "Event accepted"
}
```

**Frontend Action:**
- Show success message to user
- Store `request_id` for support/debugging
- Clear form or navigate to next page

### Validation Error (400)

**When:**
- Invalid JSON syntax
- Missing required fields
- `user_id` or `details` violates length constraints
- `action` is not one of the 4 allowed values
- Empty request body

**Response Examples:**

**Invalid JSON:**
```json
{
  "error": "Invalid request: JSON parse error: expected value at line 1 column 1"
}
```

**Missing field:**
```json
{
  "error": "Invalid request: JSON parse error: missing field `action`"
}
```

**user_id too long:**
```json
{
  "error": "Validation failed: Invalid user_id: String too long: max 100, got 150"
}
```

**Empty user_id:**
```json
{
  "error": "Validation failed: Invalid user_id: String is empty"
}
```

**Invalid action:**
```json
{
  "error": "Validation failed: Invalid action type: approve"
}
```

**Frontend Action:**
- Parse error message from `error` field
- Display user-friendly error (e.g., "User ID is too long. Maximum 100 characters.")
- Highlight invalid field in form
- Do NOT retry automatically (user needs to fix input)

### Server Error (500)

**When:**
- Kinesis Data Stream is unavailable
- AWS SDK connection timeout
- Internal Lambda error

**Response:**
```json
{
  "error": "Failed to process request"
}
```

**Frontend Action:**
- Show generic error: "Something went wrong. Please try again."
- Implement retry logic (exponential backoff)
- Log error to frontend monitoring (Sentry, etc.)

---

## Error Handling

### Error Response Structure

All errors return the same JSON structure:

```typescript
interface ErrorResponse {
  error: string;
}
```

### Error Categories

#### 1. JSON Parse Errors (400)

**Triggers:**
- Malformed JSON
- Invalid UTF-8
- Unexpected data types

**Example Messages:**
```
"JSON parse error: expected value at line 1 column 1"
"JSON parse error: invalid type: integer `123`, expected a string at line 1 column 15"
```

**Frontend Fix:**
- Ensure Content-Type: application/json header is set
- Use `JSON.stringify()` to serialize request body
- Validate form data before sending

#### 2. Validation Errors (400)

**Triggers:**
- Field length violations
- Invalid action type
- Empty required fields

**Example Messages:**
```
"Invalid user_id: String too long: max 100, got 150"
"Invalid details: String is empty"
"Invalid action type: approve"
```

**Frontend Fix:**
- Implement client-side validation matching backend rules
- Show field-specific error messages
- Prevent form submission until valid

#### 3. Server Errors (500)

**Triggers:**
- Kinesis unavailable
- AWS service outage
- Lambda timeout

**Example Messages:**
```
"Failed to process request"
```

**Frontend Fix:**
- Retry with exponential backoff
- Show user-friendly error message
- Offer "Contact Support" option

### Recommended Error Handling Logic

```typescript
async function submitEvent(data: EventRequest): Promise<EventResponse> {
  try {
    const response = await fetch('https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    });

    const result = await response.json();

    if (response.ok) {
      // Success (200)
      return result as EventResponse;
    } else if (response.status === 400) {
      // Validation error - show to user, don't retry
      throw new ValidationError(result.error);
    } else if (response.status === 500) {
      // Server error - can retry
      throw new ServerError(result.error);
    } else {
      // Unexpected status
      throw new Error(`Unexpected status: ${response.status}`);
    }
  } catch (error) {
    if (error instanceof TypeError) {
      // Network error (fetch failed)
      throw new NetworkError('Unable to connect to server');
    }
    throw error;
  }
}
```

---

## CORS Configuration

### Current Configuration (Development)

**Allowed Origins:** `*` (all origins)
**Allowed Methods:** `POST`
**Allowed Headers:** `Content-Type`
**Max Age:** 300 seconds (5 minutes)

### Preflight Request

The browser will send an OPTIONS request before POST. The API Gateway handles this automatically.

**OPTIONS Request:**
```
OPTIONS /events HTTP/1.1
Host: 73jumnuhth.execute-api.us-east-1.amazonaws.com
Origin: http://localhost:3000
Access-Control-Request-Method: POST
Access-Control-Request-Headers: content-type
```

**OPTIONS Response:**
```
HTTP/1.1 200 OK
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: POST
Access-Control-Allow-Headers: Content-Type
Access-Control-Max-Age: 300
```

### Production CORS

In production, `Access-Control-Allow-Origin` will be restricted to:
```
https://kleos.example.com
```

**Frontend Impact:**
- Requests from other domains will be blocked
- Update your production frontend URL in the backend configuration

---

## Rate Limits

### Environment-Specific Limits

| Environment | Sustained Rate | Burst Limit | Monthly Cost |
|-------------|----------------|-------------|--------------|
| Development | 100 req/s      | 200 req    | ~$11         |
| Staging     | 500 req/s      | 1000 req   | ~$55         |
| Production  | 2000 req/s     | 5000 req   | ~$220        |

**Current Active Environment:** Development

### Rate Limit Behavior

**When Exceeded:**
- HTTP 429 Too Many Requests
- Response body: `{"message": "Too Many Requests"}`

**Frontend Handling:**
```typescript
if (response.status === 429) {
  // Wait and retry after delay
  await sleep(1000);
  return submitEvent(data);
}
```

### Throttling Algorithm

- **Token bucket** algorithm
- Tokens refill at sustained rate (e.g., 100/s)
- Burst limit allows temporary spikes (e.g., 200 tokens)
- Once bucket is empty, requests are rejected

---

## Example Requests

### Example 1: Create Action (cURL)

```bash
curl -X POST https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "alice-123",
    "action": "create",
    "details": "User created a new project named HelloWorld"
  }'
```

**Response:**
```json
{
  "request_id": "req_a1b2c3d4e5f6",
  "message": "Event accepted"
}
```

### Example 2: Update Action (JavaScript)

```javascript
const response = await fetch('https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    user_id: 'bob-456',
    action: 'update',
    details: 'User updated profile picture',
  }),
});

const data = await response.json();
console.log('Request ID:', data.request_id);
```

### Example 3: Delete Action (Python)

```python
import requests

response = requests.post(
    'https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events',
    headers={'Content-Type': 'application/json'},
    json={
        'user_id': 'charlie-789',
        'action': 'delete',
        'details': 'User deleted their account',
    }
)

data = response.json()
print(f"Request ID: {data['request_id']}")
```

### Example 4: View Action (React)

```typescript
import { useState } from 'react';

function EventForm() {
  const [status, setStatus] = useState<string>('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    try {
      const response = await fetch('https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: 'david-999',
          action: 'view',
          details: 'User viewed dashboard',
        }),
      });

      const data = await response.json();

      if (response.ok) {
        setStatus(`Success! Request ID: ${data.request_id}`);
      } else {
        setStatus(`Error: ${data.error}`);
      }
    } catch (error) {
      setStatus('Network error. Please try again.');
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <button type="submit">Submit Event</button>
      <p>{status}</p>
    </form>
  );
}
```

### Example 5: Error - Invalid Action

```bash
curl -X POST https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "alice",
    "action": "approve",
    "details": "Trying invalid action"
  }'
```

**Response (400):**
```json
{
  "error": "Validation failed: Invalid action type: approve"
}
```

### Example 6: Error - user_id Too Long

```bash
curl -X POST https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "'"$(python3 -c "print('x' * 101)")"'",
    "action": "create",
    "details": "Testing length limit"
  }'
```

**Response (400):**
```json
{
  "error": "Validation failed: Invalid user_id: String too long: max 100, got 101"
}
```

---

## Architecture Flow

### Request Flow Diagram

```
┌─────────────┐
│   Frontend  │
│   (React)   │
└──────┬──────┘
       │ POST /events
       │ {"user_id":"alice","action":"create","details":"..."}
       ▼
┌──────────────────────────────┐
│   AWS API Gateway (HTTP API) │
│   - CORS validation          │
│   - Rate limiting            │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│   Producer Lambda (Rust)     │
│   1. Parse JSON              │
│   2. Validate fields         │
│   3. Generate request_id     │
│   4. Push to Kinesis         │
└──────────┬───────────────────┘
           │
           │ Return response immediately
           ▼
┌─────────────┐
│   Frontend  │
│   (displays request_id)
└─────────────┘

           │
           │ (Async processing)
           ▼
┌──────────────────────────────┐
│   Kinesis Data Stream        │
│   - Stores events (24h)      │
│   - Batches records          │
└──────────┬───────────────────┘
           │
           │ Event source mapping (batches of 100)
           ▼
┌──────────────────────────────┐
│   Consumer Lambda (Rust)     │
│   1. Receive batch           │
│   2. Process each event      │
│   3. Log results             │
│   4. Report failures         │
└──────────┬───────────────────┘
           │
           │ (On failure)
           ▼
┌──────────────────────────────┐
│   Dead Letter Queue (SQS)    │
│   - Stores failed events     │
│   - 14-day retention         │
└──────────────────────────────┘
```

### What Frontend Needs to Know

1. **Synchronous Response:** API returns immediately (200) after pushing to Kinesis
2. **Async Processing:** Actual processing happens in the background (1-10 seconds later)
3. **No Processing Status:** Frontend cannot query processing status via API (currently)
4. **request_id:** Used for log searching and support tickets (not for status polling)

### Future Enhancements (Not Implemented Yet)

- WebSocket for real-time processing updates
- GET /events/{request_id} to check processing status
- Webhook callbacks when processing completes

---

## Testing Guide

### Manual Testing (cURL)

**Test 1: Valid Create Event**
```bash
curl -X POST https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events \
  -H "Content-Type: application/json" \
  -d '{"user_id":"test-user","action":"create","details":"Manual test"}' \
  -w "\nStatus: %{http_code}\n"
```
**Expected:** Status 200, response with request_id

**Test 2: Invalid Action**
```bash
curl -X POST https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events \
  -H "Content-Type: application/json" \
  -d '{"user_id":"test","action":"invalid","details":"Test"}' \
  -w "\nStatus: %{http_code}\n"
```
**Expected:** Status 400, error message about invalid action

**Test 3: Empty user_id**
```bash
curl -X POST https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events \
  -H "Content-Type: application/json" \
  -d '{"user_id":"","action":"create","details":"Test"}' \
  -w "\nStatus: %{http_code}\n"
```
**Expected:** Status 400, error about empty string

**Test 4: Missing Content-Type**
```bash
curl -X POST https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events \
  -d '{"user_id":"test","action":"create","details":"Test"}' \
  -w "\nStatus: %{http_code}\n"
```
**Expected:** Status 400, JSON parse error

### Automated Testing (Jest)

```typescript
describe('Kleos Events API', () => {
  const API_URL = 'https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events';

  it('should accept valid create event', async () => {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        user_id: 'test-user-123',
        action: 'create',
        details: 'Test event from Jest',
      }),
    });

    expect(response.status).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty('request_id');
    expect(data.message).toBe('Event accepted');
  });

  it('should reject invalid action', async () => {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        user_id: 'test-user',
        action: 'invalid-action',
        details: 'Test',
      }),
    });

    expect(response.status).toBe(400);

    const data = await response.json();
    expect(data.error).toContain('Invalid action type');
  });

  it('should reject user_id exceeding 100 chars', async () => {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        user_id: 'x'.repeat(101),
        action: 'create',
        details: 'Test',
      }),
    });

    expect(response.status).toBe(400);

    const data = await response.json();
    expect(data.error).toContain('String too long: max 100');
  });

  it('should handle network errors gracefully', async () => {
    const invalidUrl = 'https://invalid-domain-12345.com/events';

    await expect(
      fetch(invalidUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: 'test',
          action: 'create',
          details: 'Test',
        }),
      })
    ).rejects.toThrow();
  });
});
```

### Postman Collection

**Save this as `kleos-api.postman_collection.json`:**

```json
{
  "info": {
    "name": "Kleos Events API",
    "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
  },
  "item": [
    {
      "name": "Create Event",
      "request": {
        "method": "POST",
        "header": [
          {
            "key": "Content-Type",
            "value": "application/json"
          }
        ],
        "body": {
          "mode": "raw",
          "raw": "{\n  \"user_id\": \"alice-123\",\n  \"action\": \"create\",\n  \"details\": \"User created a new project\"\n}"
        },
        "url": {
          "raw": "https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events",
          "protocol": "https",
          "host": ["73jumnuhth", "execute-api", "us-east-1", "amazonaws", "com"],
          "path": ["events"]
        }
      }
    },
    {
      "name": "Invalid Action (Error Test)",
      "request": {
        "method": "POST",
        "header": [
          {
            "key": "Content-Type",
            "value": "application/json"
          }
        ],
        "body": {
          "mode": "raw",
          "raw": "{\n  \"user_id\": \"test\",\n  \"action\": \"approve\",\n  \"details\": \"This should fail\"\n}"
        },
        "url": {
          "raw": "https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events",
          "protocol": "https",
          "host": ["73jumnuhth", "execute-api", "us-east-1", "amazonaws", "com"],
          "path": ["events"]
        }
      }
    }
  ]
}
```

---

## Monitoring & Observability

### CloudWatch Logs

**Log Group:** `/aws/lambda/kleos-api-producer-dev`

**Log Format:** Structured JSON

**Example Log Entry:**
```json
{
  "timestamp": "2025-11-16T12:34:56.789Z",
  "level": "INFO",
  "fields": {
    "message": "Received API request",
    "request_id": "req_abc123",
    "method": "POST",
    "path": "/events"
  }
}
```

### CloudWatch Metrics

**Available Metrics (AWS/Lambda namespace):**
- `Invocations` - Total requests
- `Errors` - Failed requests (500 errors)
- `Duration` - Response time (p50, p99)
- `Throttles` - Rate limit hits
- `ConcurrentExecutions` - Active Lambda instances

### Dead Letter Queue Monitoring

**Queue Name:** `kleos-failed-records-dev`

**How to Check:**
```bash
aws sqs get-queue-attributes \
  --queue-url https://sqs.us-east-1.amazonaws.com/.../kleos-failed-records-dev \
  --attribute-names ApproximateNumberOfMessages
```

**If DLQ has messages:**
- Processing failed for some events
- Backend team needs to investigate
- Frontend should show generic error to user

### Request Tracing

**AWS X-Ray Integration:** Enabled

Every request gets a trace ID in response headers:
```
x-amzn-trace-id: Root=1-65678abc-def123456789abcdef012345
```

Use this for support tickets and debugging.

### Support Flow

When user reports an issue:

1. **Collect request_id** from success response
2. **Collect timestamp** when issue occurred
3. **Collect trace_id** from response headers (if available)
4. **Search CloudWatch Logs:**
   ```bash
   aws logs filter-log-events \
     --log-group-name /aws/lambda/kleos-api-producer-dev \
     --filter-pattern "req_abc123"
   ```

---

## Frontend Implementation Checklist

### Required Features

- [ ] POST request to `/events` endpoint
- [ ] Content-Type: application/json header
- [ ] JSON body with user_id, action, details
- [ ] Handle 200 success response
- [ ] Handle 400 validation errors
- [ ] Handle 500 server errors
- [ ] Display request_id to user
- [ ] Client-side validation (optional but recommended)

### Client-Side Validation (Recommended)

```typescript
function validateEventRequest(data: EventRequest): string[] {
  const errors: string[] = [];

  // user_id validation
  const userId = data.user_id.trim();
  if (userId.length === 0) {
    errors.push('User ID cannot be empty');
  } else if (userId.length > 100) {
    errors.push('User ID must be 100 characters or less');
  }

  // action validation
  const validActions = ['create', 'update', 'delete', 'view'];
  if (!validActions.includes(data.action.toLowerCase())) {
    errors.push('Action must be one of: create, update, delete, view');
  }

  // details validation
  const details = data.details.trim();
  if (details.length === 0) {
    errors.push('Details cannot be empty');
  } else if (details.length > 1000) {
    errors.push('Details must be 1000 characters or less');
  }

  return errors;
}
```

### Retry Logic (Recommended)

```typescript
async function submitEventWithRetry(
  data: EventRequest,
  maxRetries = 3
): Promise<EventResponse> {
  let lastError: Error;

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await submitEvent(data);
    } catch (error) {
      lastError = error as Error;

      // Don't retry validation errors (400)
      if (error instanceof ValidationError) {
        throw error;
      }

      // Retry server errors (500) with exponential backoff
      if (error instanceof ServerError && attempt < maxRetries - 1) {
        const delay = Math.pow(2, attempt) * 1000; // 1s, 2s, 4s
        await sleep(delay);
        continue;
      }

      // Retry network errors
      if (error instanceof NetworkError && attempt < maxRetries - 1) {
        const delay = Math.pow(2, attempt) * 1000;
        await sleep(delay);
        continue;
      }

      throw error;
    }
  }

  throw lastError!;
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
```

### Loading States (Recommended)

```typescript
function EventSubmitButton() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [requestId, setRequestId] = useState<string | null>(null);

  const handleSubmit = async () => {
    setLoading(true);
    setError(null);
    setRequestId(null);

    try {
      const response = await submitEvent({
        user_id: 'user-123',
        action: 'create',
        details: 'Test event',
      });

      setRequestId(response.request_id);
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <button onClick={handleSubmit} disabled={loading}>
        {loading ? 'Submitting...' : 'Submit Event'}
      </button>
      {error && <p className="error">{error}</p>}
      {requestId && <p className="success">Request ID: {requestId}</p>}
    </div>
  );
}
```

---

## API Versioning

**Current Version:** v1 (implicit - no version in URL)

**Future Versioning Strategy:**
- URL-based: `/v2/events`
- Header-based: `X-API-Version: 2`

**Deprecation Policy:**
- 6-month notice before breaking changes
- Old version maintained for 12 months after deprecation

---

## Performance Expectations

### Response Times (p99)

| Scenario | Expected Latency |
|----------|------------------|
| Valid request | < 200ms |
| Validation error | < 50ms |
| Server error | < 500ms |

### Throughput

| Environment | Max Throughput |
|-------------|----------------|
| Development | 100 req/s sustained |
| Staging | 500 req/s sustained |
| Production | 2000 req/s sustained |

### Availability

**SLA:** 99.9% uptime (target)

**Downtime Budget:** ~43 minutes/month

**Monitoring:** CloudWatch Alarms for:
- Error rate > 10 errors/5min
- Lambda duration > 3s (p99)
- DLQ depth > 0 messages

---

## Security Considerations

### Data in Transit

- **HTTPS Only:** All requests encrypted with TLS 1.2+
- **Certificate:** AWS-managed (*.amazonaws.com)

### Data at Rest

- **Kinesis Encryption:** KMS encryption with AWS managed key
- **DLQ Encryption:** Default SQS encryption

### Input Validation

- **Max payload size:** 256 KB (API Gateway limit)
- **String validation:** Length checks prevent buffer overflow
- **No SQL injection risk:** No database queries
- **No XSS risk:** API doesn't render HTML

### PII Handling

**Current Implementation:**
- No PII filtering
- `user_id` and `details` can contain sensitive data

**Recommendations:**
- Don't send passwords, credit cards, or SSNs
- Hash or pseudonymize user_id if possible
- Sanitize details field before sending

---

## Changelog

### v1.0.0 (2025-11-16)

- Initial API release
- POST /events endpoint
- Support for 4 action types: create, update, delete, view
- String validation (100/1000 char limits)
- Kinesis Data Stream integration
- CloudWatch logging
- Dead Letter Queue for failures

---

## Support

**Backend Team Contact:**
- Repository: `/Users/sergiorobayo/projects/kleos`
- Documentation: See README.md
- Issues: Use GitHub Issues

**Useful Commands:**

Check API status:
```bash
curl https://73jumnuhth.execute-api.us-east-1.amazonaws.com/events \
  -X OPTIONS \
  -H "Origin: http://localhost:3000"
```

Check stack outputs:
```bash
aws cloudformation describe-stacks \
  --stack-name kleos-dev \
  --query 'Stacks[0].Outputs'
```

---

## Appendix: Full TypeScript SDK

```typescript
// kleos-client.ts

const API_BASE_URL = 'https://73jumnuhth.execute-api.us-east-1.amazonaws.com';

export interface EventRequest {
  user_id: string;
  action: 'create' | 'update' | 'delete' | 'view';
  details: string;
}

export interface EventResponse {
  request_id: string;
  message: string;
}

export interface ErrorResponse {
  error: string;
}

export class KleosClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  async submitEvent(event: EventRequest): Promise<EventResponse> {
    const response = await fetch(`${this.baseUrl}/events`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(event),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || 'Unknown error');
    }

    return data as EventResponse;
  }

  async submitEventWithRetry(
    event: EventRequest,
    maxRetries: number = 3
  ): Promise<EventResponse> {
    let lastError: Error;

    for (let attempt = 0; attempt < maxRetries; attempt++) {
      try {
        return await this.submitEvent(event);
      } catch (error) {
        lastError = error as Error;

        // Only retry on network or server errors
        if (attempt < maxRetries - 1) {
          const delay = Math.pow(2, attempt) * 1000;
          await new Promise(resolve => setTimeout(resolve, delay));
          continue;
        }

        throw error;
      }
    }

    throw lastError!;
  }

  validateEvent(event: EventRequest): string[] {
    const errors: string[] = [];

    const userId = event.user_id.trim();
    if (userId.length === 0) {
      errors.push('user_id cannot be empty');
    } else if (userId.length > 100) {
      errors.push('user_id must be 100 characters or less');
    }

    const validActions = ['create', 'update', 'delete', 'view'];
    if (!validActions.includes(event.action)) {
      errors.push('action must be one of: create, update, delete, view');
    }

    const details = event.details.trim();
    if (details.length === 0) {
      errors.push('details cannot be empty');
    } else if (details.length > 1000) {
      errors.push('details must be 1000 characters or less');
    }

    return errors;
  }
}

// Usage example
const client = new KleosClient();

try {
  const response = await client.submitEventWithRetry({
    user_id: 'alice-123',
    action: 'create',
    details: 'User created a new resource',
  });

  console.log('Success! Request ID:', response.request_id);
} catch (error) {
  console.error('Failed to submit event:', error.message);
}
```

---

**End of Document**
