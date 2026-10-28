# Rust Mailer API

A fast, reliable REST API for sending emails built with Rust and Axum. Send emails via JSON API with support for both plain text and HTML content.

##  Features

-  **REST API**: Simple JSON endpoints for sending emails
-  **Multi-format Support**: Plain text and HTML emails
-  **Bulk Email Sending**: Send emails to multiple recipients simultaneously
-  **Personalised Batches**: Send a different subject and body to each recipient in one call
-  **Email Validation**: Check one address or up to 100 at once (syntax, domain, MX, disposable, role-based, typo hints) before you send
-  **Scheduled Batches**: Queue a personalised batch to go out at a set time
-  **Sent Log**: Every successful send is recorded, and you can email the same person again whenever you like
-  **Configurable**: Environment-based configuration
-  **Fast**: Built with Rust and Axum for high performance
-  **Validation**: Email format and content validation
-  **CORS Enabled**: Ready for web applications
-  **Health Check**: Built-in health monitoring endpoint

## Setup

### 1. Clone and Build

```bash
git clone <your-repo>
cd EmailEngine
cargo build --release
```

### 2. Environment Variables

Create a `.env` file in the project root:

```env
# Email Configuration (Required)
EMAIL_ADDRESS=your-email@gmail.com
APP_PASSWORD=your-app-password-here

# SMTP Configuration (Optional)
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587

# Server Configuration (Optional)
PORT=6969
```

### 3. Gmail Setup

For Gmail, you'll need an **App Password**:
1. Enable 2-Factor Authentication on your Google account
2. Go to Google Account Settings → Security → App passwords
3. Generate a new app password for "Mail"
4. Use this password in the `APP_PASSWORD` environment variable

## Running the Server

```bash
cargo run
```

The server will start on `http://localhost:6969` (or your configured PORT).

## API Documentation

### Health Check

**GET** `/`

```bash
curl http://localhost:6969/
```

**Response:**
```json
{
  "status": "healthy",
  "service": "rust-mailer-api",
  "version": "0.1.0"
}
```

### Send Single Email

**POST** `/send-email`

**Request Body:**
```json
{
  "to": "recipient@example.com",
  "subject": "Hello from Rust Mailer!",
  "body": "This is a plain text message.",
  "html": "<h1>Hello!</h1><p>This is an <strong>HTML</strong> message.</p>"
}
```

**Fields:**
- `to` (required): Recipient email address
- `subject` (required): Email subject
- `body` (required): Plain text content
- `html` (optional): HTML content for rich emails

**Success Response (200):**
```json
{
  "success": true,
  "message": "Email sent successfully",
  "email_sent_to": "recipient@example.com"
}
```

**Error Response (400/500):**
```json
{
  "success": false,
  "message": "Invalid email address format",
  "email_sent_to": null
}
```

### Send Bulk Emails

**POST** `/send-bulk-email`

**Request Body:**
```json
{
  "recipients": [
    "user1@example.com",
    "user2@example.com",
    "user3@example.com"
  ],
  "subject": "Bulk Email Test",
  "body": "This is a bulk email message sent to multiple recipients.",
  "html": "<h1>Bulk Email</h1><p>This is a <strong>bulk email</strong> message.</p>"
}
```

**Fields:**
- `recipients` (required): Array of recipient email addresses
- `subject` (required): Email subject
- `body` (required): Plain text content
- `html` (optional): HTML content for rich emails

**Success Response (200/206):**
```json
{
  "total_emails": 3,
  "successful": 2,
  "failed": 1,
  "results": [
    {
      "email": "user1@example.com",
      "success": true,
      "message": "Email sent successfully"
    },
    {
      "email": "user2@example.com",
      "success": false,
      "message": "SMTP connection error"
    }
  ]
}
```

### Send a Personalised Batch

**POST** `/send-batch`

**Why it exists:** `/send-bulk-email` sends the *same* subject and body to everyone. When every recipient needs their own message (their name, their company), you would have to make one `/send-email` call per person and handle pacing and retries yourself. `/send-batch` takes the whole list in one request, sends the messages one after another with a short pause between them (so the SMTP provider doesn't flag a burst of connections).

**Request Body:**
```json
{
  "messages": [
    {
      "to": "alice@example.com",
      "subject": "Hi Alice",
      "body": "Plain text version for Alice.",
      "html": "<p>HTML version for <strong>Alice</strong>.</p>"
    },
    {
      "to": "bob@example.com",
      "subject": "Hi Bob",
      "body": "Plain text version for Bob."
    }
  ]
}
```

**Fields:**
- `messages` (required): 1 to 50 messages per request
  - `to` (required): Recipient email address
  - `subject` (required): Email subject
  - `body` (required): Plain text content
  - `html` (optional): HTML content for rich emails

**Success Response (200):** same shape as `/send-bulk-email`, with one result per message, in the same order as the request:
```json
{
  "total_emails": 2,
  "successful": 2,
  "failed": 0,
  "results": [
    { "email": "alice@example.com", "success": true, "message": "Email sent successfully" },
    { "email": "bob@example.com", "success": true, "message": "Email sent successfully" }
  ]
}
```

Invalid addresses and empty subjects or bodies come back as `success: false`. A request with no messages, or more than 50, returns `400`. Addresses already in the sent log are sent again like any other.

Only one batch runs at a time. A second request waits for the first to finish.

### Validate an Email

**GET** `/validate-email?email=<address>`

A thin wrapper around the public [rapid-email-verifier](https://rapid-email-verifier.fly.dev) service. It checks an address without sending anything, so you can clean a list before a campaign. The address is sent to that third-party service, which says it processes addresses in memory and does not store them.

```bash
curl "http://localhost:6969/validate-email?email=anirban@jurisiq.in"
```

**Response (200):**
```json
{
  "email": "anirban@jurisiq.in",
  "validations": {
    "syntax": true,
    "domain_exists": true,
    "mx_records": true,
    "mailbox_exists": true,
    "is_disposable": false,
    "is_role_based": false
  },
  "score": 100,
  "status": "VALID"
}
```

Optional fields appear when relevant: `aliasOf` (the canonical address, e.g. `user.name+tag@gmail.com` becomes `username@gmail.com`) and `typoSuggestion` (e.g. `john@gmial.com` suggests `john@gmail.com`).

**What this does and does not tell you.** It checks syntax, whether the domain exists and has MX records, disposable domains and role addresses (`admin@`, `info@`). It does **not** confirm that the specific inbox exists: `mailbox_exists` is just a copy of `mx_records`, so `VALID` means "the domain can accept mail", not "this person's inbox is real". It also can't detect catch-all domains or predict bounces. Treat it as a filter for obvious junk, not a guarantee.

`status` is the verdict (for example `VALID`, `INVALID_FORMAT`, `INVALID_DOMAIN`, `NO_MX_RECORDS`) and `score` runs from 0 to 100. An address that fails validation is still a normal `200` response, so read `status` rather than the HTTP code. `400` means the `email` was empty, and `502` means the verifier could not be reached (15 second timeout).

### Validate Many Emails

**POST** `/validate-emails`

Checks up to 100 addresses in one request instead of 100 separate `/validate-email` calls (the verifier also groups them by domain, so it is much faster). Addresses are trimmed, and results come back in the same order you sent them.

```bash
curl -X POST http://localhost:6969/validate-emails \
  -H "Content-Type: application/json" \
  -d '{ "emails": ["anirban@jurisiq.in", "john@gmial.com", "bad"] }'
```

**Response (200):** `{ "results": [ ... ] }`, where each entry has the same shape as `/validate-email` (including `aliasOf` and `typoSuggestion` when present). As with the single endpoint, read each entry's `status`. `400` means an empty list, more than 100 addresses or a blank entry, and `502` means the verifier could not be reached. The same limits apply: it checks syntax, domain and MX, not whether the inbox exists.

### Schedule a Batch

**POST** `/schedule-batch`

Queues a personalised batch (same messages as `/send-batch`) to be sent at a later time. A background task checks every 10 seconds and sends each job when its time comes, using the same pacing, timeout and sent logging as `/send-batch`.

```json
{
  "send_at": "2026-11-02T10:30:00+05:30",
  "messages": [
    { "to": "alice@example.com", "subject": "Hi Alice", "body": "Hello Alice!" }
  ]
}
```

- `send_at` (required): RFC 3339 timestamp **with an offset** (`+05:30` for IST, `Z` for UTC). It must be in the future.
- `messages` (required): 1 to 50 messages, validated up front, so a bad address is rejected now rather than at send time.

**Response (201):** the job, with an `id` to track it.
```json
{ "id": "99f0cbb4-...", "send_at": "2026-11-02T05:00:00Z", "status": "pending", "total_emails": 1, "successful": null, "failed": null }
```

**GET** `/scheduled` lists all jobs. **GET** `/scheduled/{id}` shows one job in full, including per-message results once it has run. **DELETE** `/scheduled/{id}` cancels a job that is still `pending` (otherwise `409`).

Job statuses: `pending`, `running`, `done`, `cancelled`, `interrupted`.

**Things to know (this runs locally):**
- Jobs are stored in `scheduled_jobs.json` (git-ignored), so they survive restarts.
- Jobs only send while the server is running. If the server was off at the scheduled time, the job is sent right after the next start.
- If the server stops *while* a job is sending, that job is marked `interrupted` and is **not** retried, because some people may already have the email. Check `GET /scheduled/{id}` or `sent_log.txt` and re-schedule the rest by hand.
- Only one batch runs at a time, scheduled or not.

### Sent Log

**GET** `/sent`

**Why it exists:** the sent log lives on the server, so it remembers who has been emailed even if a client restarts or clears its own storage. This endpoint lets a client read that list (for example to show who was already contacted). It is read-only and sends nothing.

```bash
curl http://localhost:6969/sent
```

**Response:**
```json
{
  "addresses": ["alice@example.com", "bob@example.com"]
}
```

**How the log works:**
- The server appends one line to `sent_log.txt` (next to where you run it) after each successful send, from **any** endpoint (`/send-email`, `/send-bulk-email` and `/send-batch`): `address<TAB>unix-time`. So the log is the complete list of everyone the server has emailed.
- Only the first word of each line is read. Addresses are stored in lowercase.
- Failed sends are not logged.
- The log is a record only. No endpoint skips an address because it is in the log, so you can email anyone again.
- The file lists people you emailed, so it is git-ignored. Back it up if you want to keep the history.

## Testing Examples

### Plain Text Single Email

```bash
curl -X POST http://localhost:6969/send-email \
  -H "Content-Type: application/json" \
  -d '{
    "to": "test@example.com",
    "subject": "Test Email",
    "body": "This is a test message from Rust Mailer API!"
  }'
```

### Bulk Email

```bash
curl -X POST http://localhost:6969/send-bulk-email \
  -H "Content-Type: application/json" \
  -d '{
    "recipients": [
      "user1@example.com",
      "user2@example.com",
      "user3@example.com"
    ],
    "subject": "Bulk Email Test",
    "body": "This is a bulk email message sent to multiple recipients."
  }'
```

### Personalised Batch

```bash
curl -X POST http://localhost:6969/send-batch \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      { "to": "alice@example.com", "subject": "Hi Alice", "body": "Hello Alice!" },
      { "to": "bob@example.com", "subject": "Hi Bob", "body": "Hello Bob!" }
    ]
  }'
```

Run it twice and both people get the email twice.

### HTML Email

```bash
curl -X POST http://localhost:6969/send-email \
  -H "Content-Type: application/json" \
  -d '{
    "to": "test@example.com",
    "subject": "Beautiful HTML Email",
    "body": "This is the plain text version.",
    "html": "<div style=\"font-family: Arial, sans-serif;\"><h1 style=\"color: #333;\">Welcome!</h1><p>This is a <strong>beautiful</strong> HTML email with <em>styling</em>.</p><button style=\"background: #007bff; color: white; padding: 10px 20px; border: none; border-radius: 5px;\">Click Me!</button></div>"
  }'
```

### Using JavaScript/Node.js

```javascript
const response = await fetch('http://localhost:6969/send-email', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    to: 'user@example.com',
    subject: 'Welcome to our service!',
    body: 'Thanks for signing up!',
    html: '<h2>Welcome!</h2><p>Thanks for <strong>signing up</strong>!</p>'
  })
});

const result = await response.json();
console.log(result);
```

### Using Python

```python
import requests

response = requests.post('http://localhost:6969/send-email', json={
    'to': 'user@example.com',
    'subject': 'Python Test Email',
    'body': 'This email was sent from Python!',
    'html': '<h1>Python Email</h1><p>This email was sent from <strong>Python</strong>!</p>'
})

print(response.json())
```

## Configuration Options

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `EMAIL_ADDRESS` | Required | Your email address (sender) |
| `APP_PASSWORD` | Required | App password for authentication |
| `SMTP_SERVER` | `smtp.gmail.com` | SMTP server hostname |
| `SMTP_PORT` | `587` | SMTP server port |
| `SENDER_NAME` | `JurisIQ Support Team` | Display name shown as the sender |
| `PORT` | `6969` | API server port |

## Error Handling

The API provides detailed error messages for various scenarios:

- **400 Bad Request**: Invalid email format, empty subject/body, an empty / oversized `/send-batch`, `/schedule-batch` or `/validate-emails` request, or a `send_at` in the past
- **404 / 409**: unknown scheduled job id / cancelling a job that is no longer pending
- **500 Internal Server Error**: SMTP connection issues, authentication failures
- **502 Bad Gateway**: the email verifier behind `/validate-email(s)` could not be reached

## Status Codes

- `200` - Email sent successfully (for `/send-batch`, check each result)
- `201` - Batch scheduled
- `206` - Partial success in bulk email sending
- `400` - Bad request (validation errors)
- `500` - Internal server error (SMTP issues)
- `502` - Email verifier unreachable

## Future Enhancements

- [x] Bulk email sending
- [x] Per-recipient batch sending
- [x] Sent log (record only, no duplicate skipping)
- [x] Email scheduling (one-off, no recurring jobs)
- [x] Email validation (single and batch, via an external verifier)
- [ ] Delivery tracking
- [ ] Rate limiting
- [ ] Authentication/API keys
- [ ] Multiple email providers
- [ ] Full email history (the sent log only records addresses)

---

**Happy Emailing!** 
