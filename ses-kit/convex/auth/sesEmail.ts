import { Email } from "@convex-dev/auth/providers/Email";
import { alphabet, generateRandomString } from "oslo/crypto";
import { SESClient, SendEmailCommand } from "@aws-sdk/client-ses";

function getSesClient() {
  return new SESClient({
    region: process.env.AWS_REGION ?? "us-east-1",
    credentials: {
      accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
      secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
    },
  });
}

async function sendSesEmail(to: string, subject: string, text: string) {
  const ses = getSesClient();
  await ses.send(
    new SendEmailCommand({
      Source:
        process.env.AUTH_FROM_EMAIL ?? "auth@weisssolutions.org",
      Destination: { ToAddresses: [to] },
      Message: {
        Subject: { Data: subject, Charset: "UTF-8" },
        Body: { Text: { Data: text, Charset: "UTF-8" } },
      },
    }),
  );
}

// OTP: sends an 8-digit code the user types in
export const SesOTP = Email({
  id: "ses-otp",
  maxAge: 60 * 15, // 15 minutes
  async generateVerificationToken() {
    return generateRandomString(8, alphabet("0-9"));
  },
  async sendVerificationRequest({ identifier: email, token }) {
    await sendSesEmail(
      email,
      `Your verification code: ${token}`,
      `Your verification code is: ${token}\n\nThis code expires in 15 minutes.\n\nIf you didn't request this, ignore this email.`,
    );
  },
});

// Magic link: sends a clickable link
// Requires SITE_URL env var (your frontend URL, e.g. https://myapp.com)
// Requires a /auth/verify page that reads ?token=&email= and calls signIn()
export const SesMagicLink = Email({
  id: "ses-magic-link",
  maxAge: 60 * 15,
  authorize: undefined,
  async sendVerificationRequest({ identifier: email, token }) {
    const siteUrl = process.env.SITE_URL;
    if (!siteUrl) {
      throw new Error("SITE_URL environment variable is required for magic links");
    }
    const url = `${siteUrl}/auth/verify?token=${encodeURIComponent(token)}&email=${encodeURIComponent(email)}`;
    await sendSesEmail(
      email,
      "Your sign-in link",
      `Click here to sign in:\n\n${url}\n\nThis link expires in 15 minutes.\n\nIf you didn't request this, ignore this email.`,
    );
  },
});
