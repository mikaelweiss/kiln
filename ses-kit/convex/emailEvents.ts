import { internalMutation, internalQuery } from "./_generated/server";
import { v } from "convex/values";

// Schema to add to your convex/schema.ts:
//
// emailEvents: defineTable({
//   eventType: v.string(),
//   email: v.string(),
//   details: v.string(),
//   timestamp: v.number(),
//   snsMessageId: v.string(),
// }).index("by_snsMessageId", ["snsMessageId"]),
//
// suppressedEmails: defineTable({
//   email: v.string(),
//   reason: v.string(),
//   timestamp: v.number(),
// }).index("by_email", ["email"]),

export const recordEvent = internalMutation({
  args: {
    eventType: v.string(),
    email: v.string(),
    details: v.string(),
    timestamp: v.number(),
    snsMessageId: v.string(),
  },
  handler: async (ctx, args) => {
    const existing = await ctx.db
      .query("emailEvents")
      .withIndex("by_snsMessageId", (q) => q.eq("snsMessageId", args.snsMessageId))
      .first();
    if (existing) return;

    await ctx.db.insert("emailEvents", {
      eventType: args.eventType,
      email: args.email,
      details: args.details,
      timestamp: args.timestamp,
      snsMessageId: args.snsMessageId,
    });
  },
});

export const suppressEmail = internalMutation({
  args: {
    email: v.string(),
    reason: v.string(),
    timestamp: v.number(),
  },
  handler: async (ctx, args) => {
    const existing = await ctx.db
      .query("suppressedEmails")
      .withIndex("by_email", (q) => q.eq("email", args.email))
      .first();
    if (existing) return;

    await ctx.db.insert("suppressedEmails", {
      email: args.email,
      reason: args.reason,
      timestamp: args.timestamp,
    });
  },
});

export const isEmailSuppressed = internalQuery({
  args: {
    email: v.string(),
  },
  handler: async (ctx, args) => {
    const entry = await ctx.db
      .query("suppressedEmails")
      .withIndex("by_email", (q) => q.eq("email", args.email))
      .first();
    return entry !== null;
  },
});
