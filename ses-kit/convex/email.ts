"use node";

import { action } from "./_generated/server";
import { internal } from "./_generated/api";
import { v } from "convex/values";
import { SESClient, SendEmailCommand } from "@aws-sdk/client-ses";

export const send = action({
  args: {
    to: v.string(),
    subject: v.string(),
    text: v.string(),
    html: v.optional(v.string()),
    replyTo: v.optional(v.string()),
    from: v.optional(v.string()),
  },
  handler: async (ctx, { to, subject, text, html, replyTo, from }) => {
    const isSuppressed = await ctx.runQuery(
      internal.emailEvents.isEmailSuppressed,
      { email: to },
    );
    if (isSuppressed) {
      console.log(`Email to ${to} suppressed — address is on suppression list`);
      return { messageId: null, suppressed: true };
    }

    const ses = new SESClient({
      region: process.env.AWS_REGION ?? "us-east-1",
      credentials: {
        accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
        secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
      },
    });

    const command = new SendEmailCommand({
      Source: from ?? process.env.DEFAULT_FROM_EMAIL ?? "noreply@weisssolutions.org",
      Destination: { ToAddresses: [to] },
      Message: {
        Subject: { Data: subject, Charset: "UTF-8" },
        Body: {
          Text: { Data: text, Charset: "UTF-8" },
          ...(html ? { Html: { Data: html, Charset: "UTF-8" } } : {}),
        },
      },
      ReplyToAddresses: replyTo ? [replyTo] : undefined,
    });

    const response = await ses.send(command);
    return { messageId: response.MessageId };
  },
});
