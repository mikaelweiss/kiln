use std::process::Command;
use std::thread;
use std::time::Duration;

use dialoguer::Input;
use serde_json::Value;

const CF_API_BASE: &str = "https://api.cloudflare.com/client/v4";

fn sanitize_domain(domain: &str) -> String {
    domain.replace('.', "-")
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Check AWS CLI is available
    let aws_check = Command::new("aws").arg("--version").output();
    match aws_check {
        Ok(output) if output.status.success() => {}
        _ => return Err("AWS CLI not found. Install it: https://aws.amazon.com/cli/".into()),
    }

    println!("SES Email Setup\n");
    println!("This will set up SES email sending, DNS records, and delivery webhooks.\n");

    // Collect all inputs upfront
    let domain: String = Input::new()
        .with_prompt("Domain (e.g. mycoolproject.com)")
        .validate_with(|input: &String| -> Result<(), &str> {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Err("Domain cannot be empty");
            }
            if !trimmed.contains('.') {
                return Err("Domain must contain a dot");
            }
            Ok(())
        })
        .interact_text()?;
    let domain = domain.trim();

    let aws_region: String = Input::new()
        .with_prompt("AWS region")
        .default("us-east-1".to_string())
        .interact_text()?;
    let aws_region = aws_region.trim();

    let cf_token: String = Input::new()
        .with_prompt("Cloudflare API token")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                return Err("Token cannot be empty");
            }
            Ok(())
        })
        .interact_text()?;
    let cf_token = cf_token.trim();

    let convex_url: String = Input::new()
        .with_prompt("Convex deployment URL (e.g. https://your-app-123.convex.site)")
        .validate_with(|input: &String| -> Result<(), &str> {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Err("URL cannot be empty");
            }
            if !trimmed.starts_with("https://") {
                return Err("URL must start with https://");
            }
            Ok(())
        })
        .interact_text()?;
    let convex_url = convex_url.trim().trim_end_matches('/');

    let from_email: String = Input::new()
        .with_prompt("Default from email")
        .default(format!("noreply@{domain}"))
        .interact_text()?;
    let from_email = from_email.trim();

    let auth_email: String = Input::new()
        .with_prompt("Auth from email (for OTP/magic links)")
        .default(format!("auth@{domain}"))
        .interact_text()?;
    let auth_email = auth_email.trim();

    let forward_email: String = Input::new()
        .with_prompt("Forward incoming email to (leave empty to skip)")
        .allow_empty(true)
        .interact_text()?;
    let forward_email = forward_email.trim();
    let setup_email_routing = !forward_email.is_empty();

    println!();

    // ── Phase 1: SES Domain Verification ───────────────────────────────

    println!("── SES Domain Verification ──\n");

    println!("→ Verifying domain with SES...");
    let output = Command::new("aws")
        .args([
            "ses",
            "verify-domain-identity",
            "--domain",
            domain,
            "--region",
            aws_region,
            "--output",
            "json",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("SES verify-domain-identity failed: {stderr}").into());
    }

    let verify_result: Value = serde_json::from_slice(&output.stdout)?;
    let verification_token = verify_result["VerificationToken"]
        .as_str()
        .ok_or("Missing VerificationToken in SES response")?
        .to_string();

    println!("  Token: {verification_token}");

    println!("→ Enabling DKIM...");
    let output = Command::new("aws")
        .args([
            "ses",
            "verify-domain-dkim",
            "--domain",
            domain,
            "--region",
            aws_region,
            "--output",
            "json",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("SES verify-domain-dkim failed: {stderr}").into());
    }

    let dkim_result: Value = serde_json::from_slice(&output.stdout)?;
    let dkim_tokens: Vec<String> = dkim_result["DkimTokens"]
        .as_array()
        .ok_or("Missing DkimTokens in SES response")?
        .iter()
        .filter_map(|t| t.as_str().map(String::from))
        .collect();

    println!("  {} DKIM tokens received", dkim_tokens.len());

    // ── Phase 2: Cloudflare DNS ────────────────────────────────────────

    println!("\n── Cloudflare DNS ──\n");

    println!("→ Looking up Cloudflare zone...");
    let client = reqwest::blocking::Client::new();
    let zones_response = client
        .get(format!("{CF_API_BASE}/zones"))
        .query(&[("name", domain)])
        .bearer_auth(cf_token)
        .send()?
        .json::<Value>()?;

    let zone = zones_response["result"]
        .as_array()
        .and_then(|zones| zones.first())
        .ok_or(format!(
            "Domain {domain} not found in Cloudflare. Add it there first."
        ))?;

    let zone_id = zone["id"]
        .as_str()
        .ok_or("Missing zone ID in Cloudflare response")?
        .to_string();

    let cf_account_id = zone["account"]["id"]
        .as_str()
        .ok_or("Missing account ID in Cloudflare zone response")?
        .to_string();

    println!("  Zone ID: {zone_id}");

    println!("→ Adding DNS records...");

    add_cf_record(
        &client,
        cf_token,
        &zone_id,
        "TXT",
        &format!("_amazonses.{domain}"),
        &verification_token,
        None,
    )?;
    println!("  + TXT _amazonses.{domain}");

    for token in &dkim_tokens {
        add_cf_record(
            &client,
            cf_token,
            &zone_id,
            "CNAME",
            &format!("{token}._domainkey.{domain}"),
            &format!("{token}.dkim.amazonses.com"),
            None,
        )?;
        println!("  + CNAME {token}._domainkey.{domain}");
    }

    let spf_value = if setup_email_routing {
        "v=spf1 include:amazonses.com include:_spf.mx.cloudflare.net ~all"
    } else {
        "v=spf1 include:amazonses.com ~all"
    };

    add_cf_record(
        &client,
        cf_token,
        &zone_id,
        "TXT",
        domain,
        spf_value,
        None,
    )?;
    println!("  + TXT {domain} (SPF)");

    add_cf_record(
        &client,
        cf_token,
        &zone_id,
        "TXT",
        &format!("_dmarc.{domain}"),
        &format!("v=DMARC1; p=quarantine; rua=mailto:dmarc@{domain}"),
        None,
    )?;
    println!("  + TXT _dmarc.{domain} (DMARC)");

    // ── Phase 3: Email Routing ───────────────────────────────────────────

    if setup_email_routing {
        println!("\n── Email Routing ──\n");

        println!("→ Enabling email routing...");
        let response = client
            .post(format!("{CF_API_BASE}/zones/{zone_id}/email/routing/enable"))
            .bearer_auth(cf_token)
            .send()?
            .json::<Value>()?;

        if response["success"].as_bool() != Some(true) {
            let errors = &response["errors"];
            let error_str = errors.to_string();
            if !error_str.contains("already enabled") {
                return Err(format!("Failed to enable email routing: {errors}").into());
            }
        }
        println!("  Enabled");

        println!("→ Adding MX records...");
        for (server, priority) in [
            ("route1.mx.cloudflare.net", 12),
            ("route2.mx.cloudflare.net", 41),
            ("route3.mx.cloudflare.net", 69),
        ] {
            add_cf_record(
                &client,
                cf_token,
                &zone_id,
                "MX",
                domain,
                server,
                Some(priority),
            )?;
            println!("  + MX {server} (priority {priority})");
        }

        println!("→ Setting up destination: {forward_email}...");

        let destinations = client
            .get(format!(
                "{CF_API_BASE}/accounts/{cf_account_id}/email/routing/addresses"
            ))
            .bearer_auth(cf_token)
            .send()?
            .json::<Value>()?;

        let already_verified = destinations["result"]
            .as_array()
            .and_then(|addrs| {
                addrs
                    .iter()
                    .find(|a| a["email"].as_str() == Some(forward_email))
            })
            .is_some_and(|addr| !addr["verified"].is_null());

        if already_verified {
            println!("  Already verified");
        } else {
            let response = client
                .post(format!(
                    "{CF_API_BASE}/accounts/{cf_account_id}/email/routing/addresses"
                ))
                .bearer_auth(cf_token)
                .json(&serde_json::json!({"email": forward_email}))
                .send()?
                .json::<Value>()?;

            if response["success"].as_bool() == Some(true) {
                println!("  Verification email sent to {forward_email}");
                println!("  Check your inbox and click the verification link");
            } else {
                let errors = &response["errors"];
                let error_str = errors.to_string();
                if error_str.contains("already exists") {
                    println!("  Already exists (check inbox if not yet verified)");
                } else {
                    return Err(
                        format!("Failed to create destination address: {errors}").into(),
                    );
                }
            }
        }

        println!("→ Creating catch-all routing rule...");
        let response = client
            .put(format!(
                "{CF_API_BASE}/zones/{zone_id}/email/routing/rules/catch_all"
            ))
            .bearer_auth(cf_token)
            .json(&serde_json::json!({
                "actions": [{"type": "forward", "value": [forward_email]}],
                "matchers": [{"type": "all"}],
                "enabled": true
            }))
            .send()?
            .json::<Value>()?;

        if response["success"].as_bool() != Some(true) {
            let errors = &response["errors"];
            return Err(format!("Failed to create catch-all rule: {errors}").into());
        }
        println!("  *@{domain} → {forward_email}");
    }

    // ── Phase 4: Delivery Webhooks ─────────────────────────────────────

    println!("\n── Delivery Webhooks ──\n");

    let sanitized = sanitize_domain(domain);

    // Get AWS account ID
    println!("→ Getting AWS account ID...");
    let output = Command::new("aws")
        .args(["sts", "get-caller-identity", "--output", "json"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to get AWS account ID: {stderr}").into());
    }

    let caller_identity: Value = serde_json::from_slice(&output.stdout)?;
    let account_id = caller_identity["Account"]
        .as_str()
        .ok_or("Missing Account in STS response")?
        .to_string();

    println!("  Account: {account_id}");

    // Create SNS topic
    let topic_name = format!("ses-{sanitized}-notifications");
    println!("→ Creating SNS topic: {topic_name}...");
    let output = Command::new("aws")
        .args([
            "sns",
            "create-topic",
            "--name",
            &topic_name,
            "--region",
            aws_region,
            "--output",
            "json",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create SNS topic: {stderr}").into());
    }

    let topic_result: Value = serde_json::from_slice(&output.stdout)?;
    let topic_arn = topic_result["TopicArn"]
        .as_str()
        .ok_or("Missing TopicArn in SNS response")?
        .to_string();

    println!("  ARN: {topic_arn}");

    // Set SNS topic policy
    println!("→ Setting SNS topic policy...");
    let policy = serde_json::json!({
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": { "Service": "ses.amazonaws.com" },
            "Action": "SNS:Publish",
            "Resource": topic_arn,
            "Condition": {
                "StringEquals": {
                    "AWS:SourceAccount": account_id
                }
            }
        }]
    });

    let output = Command::new("aws")
        .args([
            "sns",
            "set-topic-attributes",
            "--topic-arn",
            &topic_arn,
            "--attribute-name",
            "Policy",
            "--attribute-value",
            &policy.to_string(),
            "--region",
            aws_region,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to set SNS topic policy: {stderr}").into());
    }

    println!("  Policy applied");

    // Create SES configuration set
    let config_set_name = format!("ses-{sanitized}-config");
    println!("→ Creating SES configuration set: {config_set_name}...");
    let output = Command::new("aws")
        .args([
            "sesv2",
            "create-configuration-set",
            "--configuration-set-name",
            &config_set_name,
            "--region",
            aws_region,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("AlreadyExistsException") {
            println!("  Already exists, skipping");
        } else {
            return Err(format!("Failed to create configuration set: {stderr}").into());
        }
    } else {
        println!("  Created");
    }

    // Create event destination
    let destination_name = format!("ses-{sanitized}-sns");
    println!("→ Creating event destination: {destination_name}...");
    let event_destination = serde_json::json!({
        "SnsDestination": { "TopicArn": topic_arn },
        "Enabled": true,
        "MatchingEventTypes": ["BOUNCE", "COMPLAINT", "DELIVERY", "SEND", "REJECT"]
    });

    let output = Command::new("aws")
        .args([
            "sesv2",
            "create-configuration-set-event-destination",
            "--configuration-set-name",
            &config_set_name,
            "--event-destination-name",
            &destination_name,
            "--event-destination",
            &event_destination.to_string(),
            "--region",
            aws_region,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("AlreadyExistsException") {
            println!("  Already exists, skipping");
        } else {
            return Err(format!("Failed to create event destination: {stderr}").into());
        }
    } else {
        println!("  Created");
    }

    // Assign config set to domain
    println!("→ Assigning config set to {domain}...");
    let output = Command::new("aws")
        .args([
            "sesv2",
            "put-email-identity-configuration-set-attributes",
            "--email-identity",
            domain,
            "--configuration-set-name",
            &config_set_name,
            "--region",
            aws_region,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to assign config set to identity: {stderr}").into());
    }

    println!("  Assigned");

    // Subscribe Convex endpoint
    let endpoint = format!("{convex_url}/ses-webhook");
    println!("→ Subscribing {endpoint} to SNS topic...");
    let output = Command::new("aws")
        .args([
            "sns",
            "subscribe",
            "--topic-arn",
            &topic_arn,
            "--protocol",
            "https",
            "--notification-endpoint",
            &endpoint,
            "--region",
            aws_region,
            "--output",
            "json",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to subscribe endpoint: {stderr}").into());
    }

    println!("  Subscription: pending confirmation");

    // ── Phase 5: Wait for SES verification ─────────────────────────────

    println!("\n── SES Verification ──\n");

    println!("→ Waiting for SES verification (checking every 10s)...");
    for attempt in 1..=30 {
        thread::sleep(Duration::from_secs(10));

        let output = Command::new("aws")
            .args([
                "ses",
                "get-identity-verification-attributes",
                "--identities",
                domain,
                "--region",
                aws_region,
                "--output",
                "json",
            ])
            .output()?;

        if !output.status.success() {
            continue;
        }

        let status_result: Value = serde_json::from_slice(&output.stdout)?;
        let status = status_result["VerificationAttributes"][domain]["VerificationStatus"]
            .as_str()
            .unwrap_or("Pending");

        match status {
            "Success" => {
                println!("  Verified!");
                break;
            }
            "Failed" => {
                return Err(
                    format!("SES verification failed for {domain}. Check DNS records.").into(),
                );
            }
            _ => {
                println!("  Attempt {attempt}/30 — pending...");
            }
        }
    }

    // ── Done ───────────────────────────────────────────────────────────

    println!("\n── Setup Complete ──\n");
    println!("Copy these files into your Convex project:\n");
    println!("  ses-kit/convex/email.ts          →  your-project/convex/email.ts");
    println!("  ses-kit/convex/auth/sesEmail.ts   →  your-project/convex/auth/sesEmail.ts");
    println!("  ses-kit/convex/http.ts            →  your-project/convex/http.ts");
    println!("  ses-kit/convex/emailEvents.ts     →  your-project/convex/emailEvents.ts");
    println!("\nSet these environment variables in Convex dashboard:\n");
    println!("  AWS_REGION              {aws_region}");
    println!("  AWS_ACCESS_KEY_ID       (your IAM access key)");
    println!("  AWS_SECRET_ACCESS_KEY   (your IAM secret key)");
    println!("  DEFAULT_FROM_EMAIL      {from_email}");
    println!("  AUTH_FROM_EMAIL         {auth_email}");
    println!("\nThen deploy: npx convex deploy");
    println!("SNS will automatically confirm the webhook subscription on first deploy.");

    if setup_email_routing {
        println!("\nEmail routing: *@{domain} → {forward_email}");
        println!("Note: Your Cloudflare API token needs Email Routing edit permissions.");
    }

    Ok(())
}

fn add_cf_record(
    client: &reqwest::blocking::Client,
    token: &str,
    zone_id: &str,
    record_type: &str,
    name: &str,
    content: &str,
    priority: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut body = serde_json::json!({
        "type": record_type,
        "name": name,
        "content": content,
        "ttl": 1,
    });

    if record_type == "CNAME" {
        body["proxied"] = serde_json::json!(false);
    }

    if let Some(pri) = priority {
        body["priority"] = serde_json::json!(pri);
    }

    let response = client
        .post(format!("{CF_API_BASE}/zones/{zone_id}/dns_records"))
        .bearer_auth(token)
        .json(&body)
        .send()?
        .json::<Value>()?;

    if response["success"].as_bool() != Some(true) {
        let errors = &response["errors"];
        let error_str = errors.to_string();
        if error_str.contains("already exist") {
            return Ok(());
        }
        return Err(format!("Cloudflare error for {name}: {errors}").into());
    }

    Ok(())
}
