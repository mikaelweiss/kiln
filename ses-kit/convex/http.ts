import { httpRouter } from "convex/server";
import { httpAction } from "./_generated/server";
import { internal } from "./_generated/api";

const http = httpRouter();

http.route({
  path: "/ses-webhook",
  method: "POST",
  handler: httpAction(async (ctx, request) => {
    try {
      const body = await request.text();
      const message = JSON.parse(body);
      const messageType = request.headers.get("x-amz-sns-message-type");

      if (messageType === "SubscriptionConfirmation") {
        await fetch(message.SubscribeURL);
        console.log("SNS subscription confirmed");
        return new Response("OK", { status: 200 });
      }

      if (messageType === "Notification") {
        const notification = JSON.parse(message.Message);
        const snsMessageId = message.MessageId;
        const notificationType: string = notification.notificationType;
        const now = Date.now();

        if (notificationType === "Bounce") {
          const bounce = notification.bounce;
          const recipients: { emailAddress: string }[] =
            bounce.bouncedRecipients ?? [];
          for (const recipient of recipients) {
            const email = recipient.emailAddress;
            await ctx.runMutation(internal.emailEvents.recordEvent, {
              eventType: "Bounce",
              email,
              details: JSON.stringify(bounce),
              timestamp: now,
              snsMessageId,
            });
            if (bounce.bounceType === "Permanent") {
              await ctx.runMutation(internal.emailEvents.suppressEmail, {
                email,
                reason: `Permanent bounce: ${bounce.bounceSubType ?? "General"}`,
                timestamp: now,
              });
            }
          }
        } else if (notificationType === "Complaint") {
          const complaint = notification.complaint;
          const recipients: { emailAddress: string }[] =
            complaint.complainedRecipients ?? [];
          for (const recipient of recipients) {
            const email = recipient.emailAddress;
            await ctx.runMutation(internal.emailEvents.recordEvent, {
              eventType: "Complaint",
              email,
              details: JSON.stringify(complaint),
              timestamp: now,
              snsMessageId,
            });
            await ctx.runMutation(internal.emailEvents.suppressEmail, {
              email,
              reason: `Complaint: ${complaint.complaintFeedbackType ?? "unknown"}`,
              timestamp: now,
            });
          }
        } else if (notificationType === "Delivery") {
          const delivery = notification.delivery;
          const recipients: string[] = delivery.recipients ?? [];
          for (const email of recipients) {
            await ctx.runMutation(internal.emailEvents.recordEvent, {
              eventType: "Delivery",
              email,
              details: JSON.stringify(delivery),
              timestamp: now,
              snsMessageId,
            });
          }
        }
      }

      return new Response("OK", { status: 200 });
    } catch (error) {
      console.error("SES webhook error:", error);
      return new Response("OK", { status: 200 });
    }
  }),
});

export default http;
