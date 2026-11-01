# ADR-0036: Communications providers

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D36, §15, §8.2
- **Related:** ADR-0008, ADR-0035, ADR-0037, ADR-0056, ADR-0074, ADR-0080, ADR-0103, ADR-0104

## Context

Verification, recovery, OTP, magic links, invitations, security notifications and support-access consent all need reliable message delivery. v0.2 had no communications subsystem. SMS is costly and fraud-prone (SMS pumping), WhatsApp is the dominant OTP channel in markets like India and Brazil, and self-hosters need SMTP.

## Decision

1. **Channels:** `email`, `sms`, `whatsapp`, `voice`, `push` (APNs/FCM, ADR-0056) and `webhook` (customer-delivered, any channel).
2. **Built-in providers:**

   | Channel | Providers |
   |---|---|
   | Email | SMTP (STARTTLS/implicit TLS, required by default), AWS SES (API), Postmark, SendGrid, Resend, webhook |
   | SMS / voice | Twilio, Vonage, MessageBird (Bird), AWS SNS (SMS), webhook |
   | WhatsApp | Meta WhatsApp Cloud API, Twilio WhatsApp, webhook |
   | Push | APNs (token-based auth), FCM HTTP v1 |

   Providers are implemented in `auvora-comms` behind a `MessageProvider` trait. New providers ship as declarative manifests where possible.
3. **Configuration:**
   - Per project (with an installation default in `[comms]` for self-hosting).
   - Provider credentials are envelope-encrypted (ADR-0072).
   - **Failover chains** per channel (`comms.email.providers = ["ses", "smtp"]`) with a health-based circuit breaker.
   - Per-locale and per-country routing for SMS (`comms.sms.routes`).
4. **Delivery pipeline:**
   - Every message is enqueued through the transactional outbox (ADR-0008) and sent by the `worker` role.
   - At-least-once delivery with idempotency keys to providers that support them.
   - Retries with exponential backoff (max 5 attempts within 15 min for OTP, 24 h for notifications).
   - OTP messages expire from the queue after their code TTL.
5. **Templates:**
   - MJML for email layout, with a logic-less Handlebars-compatible syntax restricted to variable substitution, conditionals and loops over provided data. No helpers that execute code.
   - Per-locale variants with a fallback chain (ADR-0104); versioned; preview and test-send in the console; per-org overrides for branding.
   - SMS/WhatsApp templates have length validation, and WhatsApp template registration support (pre-approved Meta templates).
6. **Sender identity:** per-project sender domains with SPF/DKIM/DMARC verification checks shown in the console. The Cloud default sender is `no-reply@{project}.auvora.app`-style, replaced when a custom domain is verified.
7. **Abuse and cost controls:**
   - SMS/WhatsApp/voice are **disabled by default** per project.
   - Per-project monthly spend cap (`comms.sms.monthly_cap`); country allowlist (default: none enabled until chosen).
   - Per-recipient (3 per 10 min) and per-IP limits; per-prefix velocity anomaly detection; OTP resend backoff (30 s, 60 s, 120 s…).
   - Bot challenge before sending to new numbers (ADR-0037).
   - Bounce/complaint webhooks feed a suppression list.
8. **Dev mode:** `auvora dev` routes all messages to the built-in **message catcher** UI (`/dev/messages`). Nothing is sent externally.
9. **Privacy:** message bodies containing OTPs/links are never logged. Delivery logs keep a recipient hash + status for 30 days (configurable).

## Rationale

The provider abstraction with failover covers self-hosted (SMTP) through global SaaS. The outbox gives reliability and decouples timing (needed for ADR-0035). Spend caps and default-off SMS address the largest real-world abuse vector.

## Alternatives considered

- **Webhook-only delivery** (customers bring their own): poor out-of-box experience. Kept as one option.
- **SMS enabled by default**: fraud and cost exposure. Rejected.

## Consequences

- Positive: complete messaging out of the box; fraud-resistant.
- Negative: many provider integrations to maintain. Mitigated by manifests and the shared trait.
- Risks and mitigations: provider outages are handled by failover chains; deliverability by DKIM/SPF guidance and bounce handling.

## Fallback / revisit trigger

A provider is removed if its API is deprecated or it has had no users for 2 years (announced 2 minors ahead).

## Implementation notes

- P1: email (SMTP, SES, webhook), templates, catcher.
- P2: SMS/WhatsApp/voice, remaining email providers, fraud controls.
- P7: push.
- Crate `auvora-comms`. Outbound calls go through the egress guard (ADR-0103).

## Verification

Provider contract tests with recorded fixtures; an outbox delivery test under provider failure; fraud-control tests (cap, velocity); a log scanner for OTP leakage.
