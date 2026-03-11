# SES Email Kit for Convex

Drop-in email sending for Convex projects using AWS SES. Plain text emails, delivery tracking, automatic bounce suppression, ~$0.50/month.

## One-Time Setup

### 1. Get out of the SES sandbox

- Go to [AWS SES Console](https://console.aws.amazon.com/ses/) → Account dashboard → Request production access
- Fill out the form (use case: "transactional emails for web applications")
- Takes ~24 hours for approval
- Until approved, you can only send to verified email addresses (fine for testing)

### 2. Create an IAM user

- Go to [IAM Console](https://console.aws.amazon.com/iam/) → Users → Create user
- Name it something like `ses-sender`
- Attach this inline policy (covers both sending and kiln setup):

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "ses:SendEmail",
        "ses:SendRawEmail",
        "ses:VerifyDomainIdentity",
        "ses:VerifyDomainDkim",
        "ses:GetIdentityVerificationAttributes",
        "sns:CreateTopic",
        "sns:SetTopicAttributes",
        "sns:Subscribe",
        "sesv2:CreateConfigurationSet",
        "sesv2:CreateConfigurationSetEventDestination",
        "sesv2:PutEmailIdentityConfigurationSetAttributes",
        "sts:GetCallerIdentity"
      ],
      "Resource": "*"
    }
  ]
}
```

- Create an access key (use case: "Application running outside AWS")
- Save the Access Key ID and Secret Access Key

### 3. Get a Cloudflare API token

Go to [Cloudflare API Tokens](https://dash.cloudflare.com/profile/api-tokens) and create a token with "Zone: Read" and "DNS: Edit" permissions.

### 4. Configure AWS CLI

Make sure the AWS CLI is installed and your credentials are configured:

```bash
aws configure
```

Or set environment variables in your `~/.zshrc`:

```bash
export AWS_ACCESS_KEY_ID="your-key"
export AWS_SECRET_ACCESS_KEY="your-secret"
export AWS_REGION="us-east-1"
```

## Setting Up a Domain

Once the one-time setup is done, setting up email for a new project is one command:

```bash
kiln setup-email
```

It will ask you for:
- **Domain** — e.g. `mycoolproject.com` (must already be in Cloudflare)
- **AWS region** — defaults to `us-east-1`
- **Cloudflare API token** — the token from step 3
- **Convex deployment URL** — your `.convex.site` URL (e.g. `https://your-app-123.convex.site`)
- **Default from email** — defaults to `noreply@yourdomain.com`
- **Auth from email** — defaults to `auth@yourdomain.com`

The command then:
1. Registers the domain with AWS SES and enables DKIM signing
2. Adds all DNS records in Cloudflare (verification TXT, DKIM CNAMEs, SPF, DMARC)
3. Creates an SNS topic and SES configuration set for delivery webhooks
4. Subscribes your Convex endpoint to receive bounce/complaint/delivery notifications
5. Waits for SES to confirm domain verification
6. Prints the exact env vars to set in Convex

## Per-Project Convex Setup

### 1. Install the backend dependency

```bash
cd your-project
npm install @aws-sdk/client-ses
```

You should already have `convex`, `@convex-dev/auth`, and `oslo` from Convex Auth setup.

### 2. Copy the files

```
ses-kit/convex/email.ts           →  your-project/convex/email.ts
ses-kit/convex/auth/sesEmail.ts   →  your-project/convex/auth/sesEmail.ts
ses-kit/convex/http.ts            →  your-project/convex/http.ts
ses-kit/convex/emailEvents.ts     →  your-project/convex/emailEvents.ts
```

If you already have a `convex/http.ts`, merge the `/ses-webhook` route into your existing `httpRouter()`.

### 3. Add schema tables

Add these to your `convex/schema.ts`:

```typescript
import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  // ...your existing tables...

  emailEvents: defineTable({
    eventType: v.string(),
    email: v.string(),
    details: v.string(),
    timestamp: v.number(),
    snsMessageId: v.string(),
  }).index("by_snsMessageId", ["snsMessageId"]),

  suppressedEmails: defineTable({
    email: v.string(),
    reason: v.string(),
    timestamp: v.number(),
  }).index("by_email", ["email"]),
});
```

### 4. Set Convex environment variables

The `kiln setup-email` command prints these at the end. Set them in the [Convex dashboard](https://dashboard.convex.dev/) → Settings → Environment variables:

| Variable | Value | Required |
|---|---|---|
| `AWS_REGION` | `us-east-1` | Yes |
| `AWS_ACCESS_KEY_ID` | your IAM access key | Yes |
| `AWS_SECRET_ACCESS_KEY` | your IAM secret key | Yes |
| `DEFAULT_FROM_EMAIL` | `noreply@mycoolproject.com` | Yes |
| `AUTH_FROM_EMAIL` | `auth@mycoolproject.com` | Yes |
| `SITE_URL` | `https://mycoolproject.com` | Only if using magic links |

### 5. Wire up Convex Auth

In your `convex/auth.ts`:

```typescript
import { convexAuth } from "@convex-dev/auth/server";
import { SesOTP } from "./auth/sesEmail";
// or: import { SesMagicLink } from "./auth/sesEmail";
// or both

export const { auth, signIn, signOut, store, isAuthenticated } = convexAuth({
  providers: [
    SesOTP,
    // SesMagicLink,
  ],
});
```

### 6. Deploy

```bash
npx convex deploy
```

SNS will automatically confirm the webhook subscription on first deploy.

## How Suppression Works

- **Permanent bounces** (mailbox doesn't exist, address suppressed): email is added to the suppression list immediately
- **Complaints** (user marked as spam): email is added to the suppression list immediately
- **Transient bounces** (mailbox full, temporary failure): logged but not suppressed
- **Deliveries**: logged for tracking

The `email.ts` send action checks the suppression list before every send. Suppressed emails are skipped with a console log.

## Usage

### Form submissions / contact forms

From any Convex mutation or action:

```typescript
import { api } from "./_generated/api";

// Inside a mutation handler:
await ctx.scheduler.runAfter(0, api.email.send, {
  to: "you@weisssolutions.org",
  subject: `Feedback from ${name}`,
  text: `From: ${name} (${email})\n\n${message}`,
  replyTo: email, // hit reply → goes to the person who submitted
});
```

Or from an action (which can call other actions directly):

```typescript
await ctx.runAction(api.email.send, {
  to: "you@weisssolutions.org",
  subject: `Bug report: ${title}`,
  text: body,
  replyTo: userEmail,
});
```

### Auth (OTP)

On the client, sign in with:

```typescript
signIn("ses-otp", { email: "user@example.com" });
```

User receives a plain text email with an 8-digit code. They enter it on your site.

### Auth (Magic Link)

On the client:

```typescript
signIn("ses-magic-link", { email: "user@example.com" });
```

User receives an email with a link to `{SITE_URL}/auth/verify?token=...&email=...`.

You need a page at `/auth/verify` that reads those params and completes sign-in:

```typescript
const token = searchParams.get("token");
const email = searchParams.get("email");
await signIn("ses-magic-link", { token, email });
```

## All args for email.send

| Arg | Type | Required | Description |
|---|---|---|---|
| `to` | string | Yes | Recipient email |
| `subject` | string | Yes | Email subject line |
| `text` | string | Yes | Plain text body |
| `html` | string | No | HTML body (use sparingly — plain text has better deliverability) |
| `replyTo` | string | No | Reply-to address (for form submissions) |
| `from` | string | No | Override the sender (defaults to `DEFAULT_FROM_EMAIL`) |

## New Project Checklist

1. Buy domain, point nameservers to Cloudflare
2. `kiln setup-email` (one command, answers a few prompts, ~1 min)
3. Copy all four Convex files into project
4. `npm install @aws-sdk/client-ses`
5. Add schema tables to `convex/schema.ts`
6. Set env vars in Convex dashboard (printed by the command)
7. Add provider to `convex/auth.ts`
8. `npx convex deploy`
9. Ship
