# Rust Mailer API

A fast, reliable REST API for sending emails built with Rust and Axum. Send emails via JSON API with support for both plain text and HTML content.

##  Features

-  **REST API**: Simple JSON endpoints for sending emails
-  **Multi-format Support**: Plain text and HTML emails
-  **Bulk Email Sending**: Send emails to multiple recipients simultaneously
-  **Configurable**: Environment-based configuration
-  **Fast**: Built with Rust and Axum for high performance
-  **Validation**: Email format and content validation
-  **CORS Enabled**: Ready for web applications
-  **Health Check**: Built-in health monitoring endpoint

## Setup

### 1. Clone and Build

```bash
git clone <your-repo>
cd rust-mailer
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

The server will start on `http://0.0.0.0:6969` (or your configured PORT).

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
| `PORT` | `6969` | API server port |

## Error Handling

The API provides detailed error messages for various scenarios:

- **400 Bad Request**: Invalid email format, empty subject/body
- **500 Internal Server Error**: SMTP connection issues, authentication failures

## Status Codes

- `200` - Email sent successfully
- `206` - Partial success in bulk email sending
- `400` - Bad request (validation errors)
- `500` - Internal server error (SMTP issues)

## Future Enhancements

- [x] Bulk email sending
- [ ] Email scheduling
- [ ] Delivery tracking
- [ ] Rate limiting
- [ ] Authentication/API keys
- [ ] Multiple email providers
- [ ] Email history/logging

---

**Happy Emailing!** 
