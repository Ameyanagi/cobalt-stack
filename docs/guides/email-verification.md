# Email Verification Guide

Complete guide to implementing and using email verification in Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Configuration](#configuration)
- [Backend Implementation](#backend-implementation)
- [Frontend Implementation](#frontend-implementation)
- [Email Templates](#email-templates)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)

## Overview

Cobalt Stack includes a secure email verification system that:

- Generates unique verification tokens
- Sends verification emails after registration
- Validates tokens with 24-hour expiration
- Marks user accounts as verified
- Supports both mock (development) and SMTP (production) modes

### Verification Flow

```
┌─────────────┐                 ┌─────────────┐                 ┌─────────────┐
│   Client    │                 │   Backend   │                 │    Email    │
└─────────────┘                 └─────────────┘                 └─────────────┘
       │                               │                               │
       │  POST /api/auth/register     │                               │
       │  { username, email, ... }    │                               │
       │─────────────────────────────>│                               │
       │                               │                               │
       │                          ✓ Create user                       │
       │                          ✓ Generate token                    │
       │                               │                               │
       │                               │  Send verification email      │
       │                               │─────────────────────────────>│
       │                               │                               │
       │  201 Created                  │                               │
       │  { user, message }            │                               │
       │<─────────────────────────────│                               │
       │                               │                               │
       │                               │                          📧 Email sent
       │                               │                               │
       │  User clicks link in email    │                               │
       │                               │                               │
       │  GET /verify-email?token=xyz  │                               │
       │─────────────────────────────>│                               │
       │                               │                               │
       │                          ✓ Validate token                    │
       │                          ✓ Mark user verified                │
       │                               │                               │
       │  Success page                 │                               │
       │<─────────────────────────────│                               │
```

## Configuration

### Environment Variables

Configure email settings in `backend/.env`:

```bash
# Email Service Configuration
EMAIL_MODE=mock  # mock or smtp

# SMTP Configuration (when EMAIL_MODE=smtp)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@yourapp.com
SMTP_FROM_NAME=Your App Name

# Frontend URL (for verification links)
FRONTEND_URL=http://localhost:2727
```

### Development Mode

For local development, use **mock mode**:

```bash
EMAIL_MODE=mock
```

Mock mode:
- Prints emails to console instead of sending
- No SMTP credentials required
- Perfect for testing verification flow
- Shows full email content including token

### Production Mode

For production, configure **SMTP**:

```bash
EMAIL_MODE=smtp
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=SG.your_sendgrid_api_key
SMTP_FROM_EMAIL=noreply@yourdomain.com
SMTP_FROM_NAME=Your App
```

Popular SMTP providers:
- **SendGrid**: smtp.sendgrid.net:587
- **Mailgun**: smtp.mailgun.org:587
- **AWS SES**: email-smtp.us-east-1.amazonaws.com:587
- **Gmail**: smtp.gmail.com:587 (requires app password)

## Backend Implementation

### Creating Verification Tokens

```rust
use cobalt_stack::services::email::verification::create_verification_token;
use uuid::Uuid;

// After user registration
let user_id = Uuid::new_v4();

// Create verification token (valid for 24 hours)
let token = create_verification_token(&db, user_id).await?;

// Token is automatically hashed and stored in database
// Returns plain token to send in email
println!("Verification token: {}", token);
```

### Sending Verification Emails

```rust
use cobalt_stack::services::email::{EmailService, EmailConfig};

// Initialize email service
let config = EmailConfig::from_env();
let email_service = EmailService::new(config);

// Send verification email
email_service
    .send_verification_email(
        &user.email,
        &user.username,
        &token,
    )
    .await?;
```

### Verifying Email Tokens

```rust
use cobalt_stack::services::email::verification::verify_email_token;

// Extract token from query parameter
let token = query.token;

// Verify token and mark user as verified
match verify_email_token(&db, &token).await {
    Ok(user_id) => {
        // Token valid, user now verified
        println!("User {} verified successfully", user_id);
    }
    Err(e) => {
        // Handle verification errors
        eprintln!("Verification failed: {}", e);
    }
}
```

### Registration Endpoint with Email Verification

```rust
#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Validate input
    validate_registration(&req)?;

    // 2. Hash password
    let password_hash = hash_password(&req.password)?;

    // 3. Create user
    let user = create_user(
        &state.db,
        req.username,
        req.email,
        password_hash,
    ).await?;

    // 4. Generate verification token
    let token = create_verification_token(&state.db, user.id).await?;

    // 5. Send verification email
    state
        .email_service
        .send_verification_email(&user.email, &user.username, &token)
        .await?;

    // 6. Return success
    Ok(Json(RegisterResponse {
        user,
        message: "Registration successful. Please check your email to verify your account.".to_string(),
    }))
}
```

### Verification Endpoint

```rust
#[derive(Deserialize)]
struct VerifyQuery {
    token: String,
}

async fn verify_email(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> Result<Redirect, (StatusCode, String)> {
    // Verify token
    match verify_email_token(&state.db, &query.token).await {
        Ok(_user_id) => {
            // Redirect to success page
            Ok(Redirect::to("/verify-email/success"))
        }
        Err(e) => {
            // Redirect to error page with message
            let error_msg = match e.to_string().as_str() {
                s if s.contains("Invalid") => "invalid",
                s if s.contains("expired") => "expired",
                s if s.contains("already verified") => "already-verified",
                _ => "unknown",
            };
            Ok(Redirect::to(&format!("/verify-email/error?reason={}", error_msg)))
        }
    }
}
```

### Resending Verification Emails

```rust
async fn resend_verification(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser, // From auth middleware
) -> Result<Json<MessageResponse>, StatusCode> {
    // Get user
    let user = get_user(&state.db, auth_user.user_id).await?;

    // Check if already verified
    if user.email_verified {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create new token
    let token = create_verification_token(&state.db, user.id).await?;

    // Send email
    state
        .email_service
        .send_verification_email(&user.email, &user.username, &token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MessageResponse {
        message: "Verification email sent successfully".to_string(),
    }))
}
```

## Frontend Implementation

### Registration Form

```tsx
'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { env } from '@/lib/env'

export function RegisterForm() {
  const [formData, setFormData] = useState({
    username: '',
    email: '',
    password: '',
  })
  const [message, setMessage] = useState('')
  const [error, setError] = useState('')
  const router = useRouter()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setMessage('')

    try {
      const response = await fetch(`${env.apiUrl}/api/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      })

      const data = await response.json()

      if (!response.ok) {
        setError(data.message || 'Registration failed')
        return
      }

      // Show success message
      setMessage('Registration successful! Please check your email to verify your account.')

      // Optionally redirect to login after a delay
      setTimeout(() => {
        router.push('/login')
      }, 3000)
    } catch (err) {
      setError('Network error. Please try again.')
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      {message && (
        <div className="success-message">
          {message}
        </div>
      )}

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      <input
        type="text"
        value={formData.username}
        onChange={(e) => setFormData({ ...formData, username: e.target.value })}
        placeholder="Username"
        required
      />

      <input
        type="email"
        value={formData.email}
        onChange={(e) => setFormData({ ...formData, email: e.target.value })}
        placeholder="Email"
        required
      />

      <input
        type="password"
        value={formData.password}
        onChange={(e) => setFormData({ ...formData, password: e.target.value })}
        placeholder="Password"
        required
      />

      <button type="submit">Register</button>
    </form>
  )
}
```

### Unverified Email Banner

Display a banner for users who haven't verified their email:

```tsx
'use client'

import { useAuth } from '@/contexts/auth-context'
import { useState } from 'react'
import { env } from '@/lib/env'

export function UnverifiedEmailBanner() {
  const { user, accessToken } = useAuth()
  const [message, setMessage] = useState('')
  const [isResending, setIsResending] = useState(false)

  if (!user || user.email_verified) {
    return null
  }

  const handleResend = async () => {
    setIsResending(true)
    setMessage('')

    try {
      const response = await fetch(`${env.apiUrl}/api/auth/resend-verification`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${accessToken}`,
        },
      })

      if (response.ok) {
        setMessage('Verification email sent! Please check your inbox.')
      } else {
        setMessage('Failed to send email. Please try again.')
      }
    } catch (err) {
      setMessage('Network error. Please try again.')
    } finally {
      setIsResending(false)
    }
  }

  return (
    <div className="bg-yellow-100 border-l-4 border-yellow-500 text-yellow-700 p-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="font-bold">Email Not Verified</p>
          <p className="text-sm">
            Please verify your email address to access all features.
          </p>
        </div>

        <button
          onClick={handleResend}
          disabled={isResending}
          className="ml-4 px-4 py-2 bg-yellow-500 text-white rounded hover:bg-yellow-600 disabled:opacity-50"
        >
          {isResending ? 'Sending...' : 'Resend Email'}
        </button>
      </div>

      {message && (
        <p className="mt-2 text-sm">{message}</p>
      )}
    </div>
  )
}
```

### Verification Success Page

```tsx
// app/verify-email/success/page.tsx
import Link from 'next/link'

export default function VerifySuccessPage() {
  return (
    <div className="flex min-h-screen items-center justify-center">
      <div className="max-w-md text-center">
        <div className="mb-4 text-6xl">✅</div>
        <h1 className="mb-2 text-3xl font-bold">Email Verified!</h1>
        <p className="mb-6 text-gray-600">
          Your email has been successfully verified. You can now access all features.
        </p>
        <Link
          href="/login"
          className="inline-block px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          Go to Login
        </Link>
      </div>
    </div>
  )
}
```

### Verification Error Page

```tsx
// app/verify-email/error/page.tsx
'use client'

import { useSearchParams } from 'next/navigation'
import Link from 'next/link'

export default function VerifyErrorPage() {
  const searchParams = useSearchParams()
  const reason = searchParams.get('reason')

  const getErrorMessage = () => {
    switch (reason) {
      case 'invalid':
        return 'The verification link is invalid.'
      case 'expired':
        return 'The verification link has expired. Please request a new one.'
      case 'already-verified':
        return 'This email has already been verified.'
      default:
        return 'An error occurred during verification.'
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center">
      <div className="max-w-md text-center">
        <div className="mb-4 text-6xl">❌</div>
        <h1 className="mb-2 text-3xl font-bold">Verification Failed</h1>
        <p className="mb-6 text-gray-600">
          {getErrorMessage()}
        </p>
        <Link
          href="/login"
          className="inline-block px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          Go to Login
        </Link>
      </div>
    </div>
  )
}
```

## Email Templates

### HTML Email Template

The default verification email template (in `backend/src/services/email/mod.rs`):

```rust
fn create_verification_email_html(username: &str, verification_url: &str) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Verify Your Email</title>
        </head>
        <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
            <div style="background-color: #f4f4f4; padding: 20px; border-radius: 5px;">
                <h1 style="color: #2563eb;">Welcome to Cobalt Stack, {}!</h1>

                <p>Thank you for registering. Please verify your email address by clicking the button below:</p>

                <div style="text-align: center; margin: 30px 0;">
                    <a href="{}"
                       style="background-color: #2563eb; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block; font-weight: bold;">
                        Verify Email Address
                    </a>
                </div>

                <p>Or copy and paste this link into your browser:</p>
                <p style="word-break: break-all; background-color: #e5e7eb; padding: 10px; border-radius: 3px;">
                    {}
                </p>

                <p style="color: #666; font-size: 14px; margin-top: 30px;">
                    This link will expire in 24 hours. If you didn't create an account, you can safely ignore this email.
                </p>
            </div>
        </body>
        </html>
        "#,
        username, verification_url, verification_url
    )
}
```

### Customizing Email Templates

To customize the email template:

1. Edit the HTML in `backend/src/services/email/mod.rs`
2. Add your branding and styling
3. Test with mock mode first
4. Ensure responsive design for mobile devices

Example custom template with branding:

```rust
fn create_verification_email_html(username: &str, verification_url: &str) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
        <body style="font-family: 'Helvetica Neue', Arial, sans-serif; background-color: #f9fafb;">
            <div style="max-width: 600px; margin: 40px auto; background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                <!-- Header with logo -->
                <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 40px; text-align: center; border-radius: 8px 8px 0 0;">
                    <img src="https://yourapp.com/logo.png" alt="Logo" style="height: 50px;">
                    <h1 style="color: white; margin: 20px 0 0 0;">Welcome, {}!</h1>
                </div>

                <!-- Content -->
                <div style="padding: 40px;">
                    <p style="font-size: 16px; color: #374151; margin-bottom: 20px;">
                        Thanks for joining us! Let's verify your email address to get started.
                    </p>

                    <a href="{}"
                       style="display: block; width: fit-content; margin: 30px auto; background: #667eea; color: white; padding: 15px 40px; text-decoration: none; border-radius: 6px; font-weight: 600; text-align: center;">
                        Verify My Email
                    </a>

                    <p style="font-size: 14px; color: #6b7280; margin-top: 30px; padding-top: 30px; border-top: 1px solid #e5e7eb;">
                        Link expires in 24 hours. Didn't sign up? Ignore this email.
                    </p>
                </div>

                <!-- Footer -->
                <div style="background: #f9fafb; padding: 20px; text-align: center; border-radius: 0 0 8px 8px; color: #6b7280; font-size: 12px;">
                    © 2024 Your Company. All rights reserved.
                </div>
            </div>
        </body>
        </html>
        "#,
        username, verification_url
    )
}
```

## Testing

### Testing with Mock Mode

1. Set `EMAIL_MODE=mock` in backend `.env`

2. Start the backend:
   ```bash
   cd backend
   cargo run
   ```

3. Register a new user

4. Check the backend console output:
   ```
   [INFO] Mock email mode - Email would be sent:
   To: user@example.com
   Subject: Verify your email address

   Verification URL: http://localhost:2727/verify-email?token=abc123...
   ```

5. Copy the verification URL and test in browser

### Testing SMTP in Development

1. Use a test SMTP service like [Mailtrap](https://mailtrap.io):
   ```bash
   EMAIL_MODE=smtp
   SMTP_HOST=smtp.mailtrap.io
   SMTP_PORT=2525
   SMTP_USERNAME=your_mailtrap_username
   SMTP_PASSWORD=your_mailtrap_password
   ```

2. Register a user and check Mailtrap inbox

3. Verify email rendering and links work correctly

### Automated Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_verification_token() {
        let db = setup_test_db().await;
        let user_id = Uuid::new_v4();

        let token = create_verification_token(&db, user_id)
            .await
            .expect("Failed to create token");

        assert!(!token.is_empty());
        assert_eq!(token.len(), 64); // Token length
    }

    #[tokio::test]
    async fn test_verify_valid_token() {
        let db = setup_test_db().await;
        let user_id = create_test_user(&db).await;
        let token = create_verification_token(&db, user_id).await.unwrap();

        let verified_user_id = verify_email_token(&db, &token)
            .await
            .expect("Token verification failed");

        assert_eq!(verified_user_id, user_id);
    }

    #[tokio::test]
    async fn test_verify_expired_token() {
        // Test token expiration after 24 hours
    }

    #[tokio::test]
    async fn test_verify_already_verified() {
        // Test duplicate verification
    }
}
```

## Troubleshooting

### Emails Not Being Sent

**Problem**: Users not receiving verification emails

**Solutions**:
1. Check `EMAIL_MODE` is set to `smtp` in production
2. Verify SMTP credentials are correct
3. Check spam/junk folders
4. Test SMTP connection manually
5. Review backend logs for email errors
6. Check SMTP provider's sending limits

### Invalid Token Errors

**Problem**: Verification links show "Invalid token"

**Solutions**:
1. Check token hasn't expired (24-hour limit)
2. Verify token is complete in URL (not truncated)
3. Check database has email_verifications table
4. Ensure token hashing is consistent
5. Check for typos in token from email

### Token Expired

**Problem**: Verification link shows "Token expired"

**Solutions**:
1. User must request new verification email
2. Implement resend verification endpoint
3. Consider extending token lifetime (not recommended)
4. Add reminder email before expiration

### SMTP Authentication Failed

**Problem**: SMTP authentication errors in logs

**Solutions**:
1. Verify SMTP username and password
2. For Gmail, use App Password, not account password
3. Enable "Less secure apps" for Gmail (if applicable)
4. Check SMTP host and port are correct
5. Try different SMTP ports (587, 465, 25)

### Verification URL Not Working

**Problem**: Clicking verification link gives 404 or error

**Solutions**:
1. Check `FRONTEND_URL` in backend `.env`
2. Verify frontend route exists: `/verify-email`
3. Ensure token query parameter is being extracted
4. Check frontend and backend are both running
5. Test URL directly in browser

### Database Connection Issues

**Problem**: Token creation or verification fails

**Solutions**:
1. Check database is running
2. Verify database connection string
3. Run migrations: `make migrate`
4. Check email_verifications table exists
5. Review database permissions

## Best Practices

1. **Use HTTPS in production**: Secure token transmission
2. **Set reasonable expiration**: 24 hours balances security and UX
3. **Hash tokens in database**: Never store plain tokens
4. **One-time use tokens**: Mark tokens as used after verification
5. **Rate limit resends**: Prevent email spam
6. **Clear error messages**: Help users troubleshoot issues
7. **Mobile-friendly emails**: Test on various devices
8. **Include plaintext version**: For email clients that don't support HTML
9. **Graceful degradation**: Handle email service failures
10. **Monitor email metrics**: Track delivery rates and failures

## Related Documentation

- [Authentication Guide](./authentication.md) - User authentication system
- [API Client Guide](./api-client.md) - Making API calls
- [Database Guide](./database.md) - Database setup and migrations
- [Testing Guide](./testing.md) - Testing strategies
- [API Reference](../api/README.md) - Complete API documentation
