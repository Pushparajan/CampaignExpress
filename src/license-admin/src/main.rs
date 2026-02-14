//! License Admin CLI — generate keys, create licenses, verify license files,
//! manage the billing dashboard, and operate the SaaS admin console.

use campaign_admin_console::{
    notifications::Severity, FeatureFlagManager, NotificationManager, ProviderDashboard,
    SystemSettings, TenantOps, UserOps,
};
use campaign_billing::billing::BillingEngine;
use campaign_licensing::{
    dashboard::{DashboardEngine, InvoiceStatus, PaymentStatus},
    sign_license, verify_license, License, LicenseKey, LicenseTier, LicenseType, LicensedModule,
};
use campaign_platform::auth::AuthManager;
use campaign_platform::rbac::RbacEngine;
use campaign_platform::tenancy::TenantManager;
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "license-admin")]
#[command(about = "Campaign Express License Administration Tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new HMAC-SHA256 signing key
    GenerateKey {
        /// Output file path for the key (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Create a signed license file
    CreateLicense {
        /// Path to the signing key file
        #[arg(short, long)]
        key: String,

        /// Tenant name
        #[arg(short, long)]
        tenant: String,

        /// Tenant UUID (auto-generated if omitted)
        #[arg(long)]
        tenant_id: Option<String>,

        /// License tier: starter, professional, enterprise
        #[arg(long, default_value = "professional")]
        tier: String,

        /// License type: commercial, trial, internal
        #[arg(long, default_value = "commercial")]
        license_type: String,

        /// Comma-separated list of modules (overrides tier defaults)
        #[arg(short, long)]
        modules: Option<String>,

        /// Max cluster nodes
        #[arg(long)]
        max_nodes: Option<u32>,

        /// Max offers per hour
        #[arg(long)]
        max_offers_per_hour: Option<u64>,

        /// Validity period in days
        #[arg(long, default_value = "365")]
        days: i64,

        /// Issuer name
        #[arg(long, default_value = "license-admin")]
        issued_by: String,

        /// Output file path for the license (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Verify a license file and display its contents
    Verify {
        /// Path to the signing key file
        #[arg(short, long)]
        key: String,

        /// Path to the license file
        #[arg(short, long)]
        license: String,
    },

    /// List all available licensable modules
    ListModules,

    /// Billing dashboard — view installations, usage, billing, and payments
    Dashboard {
        #[command(subcommand)]
        action: DashboardAction,
    },

    /// SaaS admin console — tenants, users, feature flags, settings
    Admin {
        #[command(subcommand)]
        action: AdminAction,
    },
}

#[derive(Subcommand)]
enum AdminAction {
    /// Show provider-level overview (all tenants, billing, usage)
    Overview,

    /// List all tenants with status and tier
    Tenants,

    /// Suspend a tenant
    SuspendTenant {
        /// Tenant UUID
        id: String,
        /// Reason for suspension
        #[arg(short, long, default_value = "Administrative action")]
        reason: String,
    },

    /// Reactivate a suspended tenant
    ReactivateTenant {
        /// Tenant UUID
        id: String,
        /// Reason for reactivation
        #[arg(short, long, default_value = "Administrative action")]
        reason: String,
    },

    /// Show all users for a tenant
    Users {
        /// Tenant UUID
        tenant_id: String,
    },

    /// Show feature flags and their status
    FeatureFlags,

    /// Show notification inbox
    Notifications,

    /// Show system settings
    Settings,

    /// Toggle maintenance mode
    Maintenance {
        /// "on" or "off"
        mode: String,
        /// Maintenance message (for on mode)
        #[arg(short, long)]
        message: Option<String>,
    },
}

#[derive(Subcommand)]
enum DashboardAction {
    /// Show fleet overview across all installations
    Overview,

    /// List all installations
    Installations,

    /// Show detailed info for a specific installation
    Installation {
        /// Installation UUID
        id: String,
    },

    /// Show usage across all installations
    Usage,

    /// Show billing summary and invoices
    Billing,

    /// Show pending payments and overdue invoices
    Payments,
}

fn parse_tier(s: &str) -> LicenseTier {
    match s.to_lowercase().as_str() {
        "starter" => LicenseTier::Starter,
        "professional" | "pro" => LicenseTier::Professional,
        "enterprise" | "ent" => LicenseTier::Enterprise,
        _ => {
            eprintln!("Warning: unknown tier '{s}', defaulting to Professional");
            LicenseTier::Professional
        }
    }
}

fn parse_license_type(s: &str) -> LicenseType {
    match s.to_lowercase().as_str() {
        "commercial" => LicenseType::Commercial,
        "trial" => LicenseType::Trial,
        "internal" => LicenseType::Internal,
        _ => {
            eprintln!("Warning: unknown license type '{s}', defaulting to Commercial");
            LicenseType::Commercial
        }
    }
}

fn parse_module(s: &str) -> Option<LicensedModule> {
    match s.trim().to_lowercase().as_str() {
        "loyalty" => Some(LicensedModule::Loyalty),
        "dsp" => Some(LicensedModule::Dsp),
        "channels" => Some(LicensedModule::Channels),
        "management" => Some(LicensedModule::Management),
        "journey" => Some(LicensedModule::Journey),
        "dco" => Some(LicensedModule::Dco),
        "cdp" => Some(LicensedModule::Cdp),
        "billing" => Some(LicensedModule::Billing),
        "ops" => Some(LicensedModule::Ops),
        "personalization" => Some(LicensedModule::Personalization),
        "segmentation" => Some(LicensedModule::Segmentation),
        "reporting" => Some(LicensedModule::Reporting),
        "integrations" => Some(LicensedModule::Integrations),
        "intelligent_delivery" | "intelligent-delivery" => {
            Some(LicensedModule::IntelligentDelivery)
        }
        "rl_engine" | "rl-engine" => Some(LicensedModule::RlEngine),
        "mobile_sdk" | "mobile-sdk" => Some(LicensedModule::MobileSdk),
        "plugin_marketplace" | "plugin-marketplace" => Some(LicensedModule::PluginMarketplace),
        "sdk_docs" | "sdk-docs" => Some(LicensedModule::SdkDocs),
        "wasm_edge" | "wasm-edge" => Some(LicensedModule::WasmEdge),
        _ => None,
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateKey { output } => cmd_generate_key(output),
        Commands::CreateLicense {
            key,
            tenant,
            tenant_id,
            tier,
            license_type,
            modules,
            max_nodes,
            max_offers_per_hour,
            days,
            issued_by,
            output,
        } => cmd_create_license(
            key,
            tenant,
            tenant_id,
            tier,
            license_type,
            modules,
            max_nodes,
            max_offers_per_hour,
            days,
            issued_by,
            output,
        ),
        Commands::Verify { key, license } => cmd_verify(key, license),
        Commands::ListModules => cmd_list_modules(),
        Commands::Dashboard { action } => cmd_dashboard(action),
        Commands::Admin { action } => cmd_admin(action),
    }
}

// ---------------------------------------------------------------------------
// License commands
// ---------------------------------------------------------------------------

fn cmd_generate_key(output: Option<String>) {
    let key = LicenseKey::generate();
    let b64 = key.to_base64();

    if let Some(path) = output {
        std::fs::write(&path, &b64).expect("Failed to write key file");
        println!("Signing key written to: {path}");
    } else {
        println!("{b64}");
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_create_license(
    key: String,
    tenant: String,
    tenant_id: Option<String>,
    tier: String,
    license_type: String,
    modules: Option<String>,
    max_nodes: Option<u32>,
    max_offers_per_hour: Option<u64>,
    days: i64,
    issued_by: String,
    output: Option<String>,
) {
    let signing_key =
        LicenseKey::load_from_file(std::path::Path::new(&key)).expect("Failed to load key");

    let tier = parse_tier(&tier);
    let license_type = parse_license_type(&license_type);

    let module_list = if let Some(ref m) = modules {
        let mut list = Vec::new();
        for name in m.split(',') {
            match parse_module(name) {
                Some(module) => list.push(module),
                None => {
                    eprintln!("Unknown module: '{}'", name.trim());
                    std::process::exit(1);
                }
            }
        }
        list
    } else {
        tier.default_modules()
    };

    let tenant_uuid = tenant_id
        .and_then(|id| Uuid::parse_str(&id).ok())
        .unwrap_or_else(Uuid::new_v4);

    let license = License {
        license_id: Uuid::new_v4(),
        tenant_id: tenant_uuid,
        tenant_name: tenant,
        license_type,
        tier,
        modules: module_list,
        max_nodes: max_nodes.unwrap_or_else(|| tier.default_max_nodes()),
        max_offers_per_hour: max_offers_per_hour
            .unwrap_or_else(|| tier.default_max_offers_per_hour()),
        issued_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(days),
        issued_by,
    };

    let signed = sign_license(&license, &signing_key).expect("Failed to sign license");

    if let Some(path) = output {
        std::fs::write(&path, &signed).expect("Failed to write license file");
        println!("License written to: {path}");
        println!("  License ID:  {}", license.license_id);
        println!("  Tenant:      {}", license.tenant_name);
        println!("  Tier:        {:?}", license.tier);
        println!("  Modules:     {}", license.modules.len());
        println!("  Max nodes:   {}", license.max_nodes);
        println!("  Expires:     {}", license.expires_at.format("%Y-%m-%d"));
    } else {
        println!("{signed}");
    }
}

fn cmd_verify(key: String, license: String) {
    let signing_key =
        LicenseKey::load_from_file(std::path::Path::new(&key)).expect("Failed to load key");
    let contents = std::fs::read_to_string(&license).expect("Failed to read license file");

    match verify_license(&contents, &signing_key) {
        Ok(lic) => {
            println!("License is VALID");
            println!();
            println!("  License ID:       {}", lic.license_id);
            println!("  Tenant ID:        {}", lic.tenant_id);
            println!("  Tenant:           {}", lic.tenant_name);
            println!("  Type:             {:?}", lic.license_type);
            println!("  Tier:             {:?}", lic.tier);
            println!("  Max nodes:        {}", lic.max_nodes);
            println!(
                "  Max offers/hr:    {}",
                format_number(lic.max_offers_per_hour)
            );
            println!(
                "  Issued at:        {}",
                lic.issued_at.format("%Y-%m-%d %H:%M UTC")
            );
            println!(
                "  Expires at:       {}",
                lic.expires_at.format("%Y-%m-%d %H:%M UTC")
            );
            println!("  Issued by:        {}", lic.issued_by);
            println!();

            if lic.is_expired() {
                println!("  WARNING: License has EXPIRED!");
            } else {
                let days_left = (lic.expires_at - Utc::now()).num_days();
                println!("  Days remaining:   {days_left}");
            }

            println!();
            println!("  Licensed modules ({}):", lic.modules.len());
            for m in &lic.modules {
                println!("    - {:<25} {}", m.as_str(), m.description());
            }
        }
        Err(e) => {
            eprintln!("License verification FAILED: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_list_modules() {
    println!("Available licensable modules:");
    println!();
    for m in LicensedModule::ALL {
        println!("  {:<25} {}", m.as_str(), m.description());
    }
    println!();
    println!("Tier presets:");
    for tier in [
        LicenseTier::Starter,
        LicenseTier::Professional,
        LicenseTier::Enterprise,
    ] {
        let mods = tier.default_modules();
        let names: Vec<&str> = mods.iter().map(|m| m.as_str()).collect();
        println!(
            "  {:?} ({} modules, {} nodes, {} offers/hr): {}",
            tier,
            mods.len(),
            tier.default_max_nodes(),
            format_number(tier.default_max_offers_per_hour()),
            names.join(", ")
        );
    }
}

// ---------------------------------------------------------------------------
// Dashboard commands
// ---------------------------------------------------------------------------

fn cmd_dashboard(action: DashboardAction) {
    let engine = DashboardEngine::new();
    engine.seed_demo_data();

    match action {
        DashboardAction::Overview => dashboard_overview(&engine),
        DashboardAction::Installations => dashboard_installations(&engine),
        DashboardAction::Installation { id } => dashboard_installation_detail(&engine, &id),
        DashboardAction::Usage => dashboard_usage(&engine),
        DashboardAction::Billing => dashboard_billing(&engine),
        DashboardAction::Payments => dashboard_payments(&engine),
    }
}

fn dashboard_overview(engine: &DashboardEngine) {
    let overview = engine.fleet_overview();

    println!("=== Campaign Express Fleet Overview ===");
    println!();
    println!("  Installations");
    println!("    Total:      {}", overview.total_installations);
    println!("    Active:     {}", overview.active_installations);
    println!("    Suspended:  {}", overview.suspended_installations);
    println!("    Expired:    {}", overview.expired_installations);
    println!("    Nodes:      {}", overview.total_nodes);
    println!();
    println!("  Revenue");
    println!(
        "    Total collected:   ${}",
        format_cents(overview.total_revenue_cents)
    );
    println!(
        "    Pending payments:  ${}",
        format_cents(overview.pending_payments_cents)
    );
    println!("    Overdue invoices:  {}", overview.overdue_invoices);
    println!();
    println!("  Module Adoption");
    for (name, count) in &overview.module_adoption {
        if *count > 0 {
            let bar = "#".repeat(*count);
            println!("    {:<25} {} {}", name, bar, count);
        }
    }
}

fn dashboard_installations(engine: &DashboardEngine) {
    let installations = engine.list_installations();

    println!("=== All Installations ===");
    println!();
    println!(
        "  {:<38} {:<20} {:<15} {:<12} {:<10} {:<8} Modules",
        "ID", "Tenant", "Tier", "Type", "Status", "Nodes"
    );
    println!("  {}", "-".repeat(130));

    for inst in &installations {
        let days_left = if inst.license.is_expired() {
            "EXPIRED".to_string()
        } else {
            let d = (inst.license.expires_at - Utc::now()).num_days();
            format!("{d}d left")
        };

        println!(
            "  {:<38} {:<20} {:<15} {:<12} {:<10} {:<8} {} ({})",
            inst.installation_id,
            truncate(&inst.tenant_name, 18),
            format!("{:?}", inst.license.tier),
            format!("{:?}", inst.license.license_type),
            inst.status,
            inst.node_count,
            inst.license.modules.len(),
            days_left,
        );
    }
    println!();
    println!("  Total: {} installations", installations.len());
}

fn dashboard_installation_detail(engine: &DashboardEngine, id_str: &str) {
    let id = match Uuid::parse_str(id_str) {
        Ok(u) => u,
        Err(_) => {
            eprintln!("Invalid UUID: {id_str}");
            std::process::exit(1);
        }
    };

    let summary = match engine.installation_summary(id) {
        Some(s) => s,
        None => {
            eprintln!("Installation not found: {id}");
            std::process::exit(1);
        }
    };

    let inst = &summary.installation;
    println!("=== Installation: {} ===", inst.tenant_name);
    println!();
    println!("  Installation ID:  {}", inst.installation_id);
    println!("  Tenant ID:        {}", inst.tenant_id);
    println!("  Status:           {}", inst.status);
    println!("  Region:           {}", inst.region);
    println!("  Environment:      {}", inst.environment);
    println!(
        "  Nodes:            {}/{}",
        inst.node_count, inst.license.max_nodes
    );
    println!("  Tier:             {:?}", inst.license.tier);
    println!("  License type:     {:?}", inst.license.license_type);
    println!(
        "  Max offers/hr:    {}",
        format_number(inst.license.max_offers_per_hour)
    );
    println!(
        "  Activated:        {}",
        inst.activated_at.format("%Y-%m-%d")
    );
    println!(
        "  License expires:  {}",
        inst.license.expires_at.format("%Y-%m-%d")
    );
    println!(
        "  Last heartbeat:   {}",
        inst.last_heartbeat.format("%Y-%m-%d %H:%M UTC")
    );

    // Modules
    println!();
    println!("  Licensed Modules ({}):", inst.license.modules.len());
    for m in &inst.license.modules {
        println!("    - {:<25} {}", m.as_str(), m.description());
    }

    // Usage
    if let Some(usage) = &summary.current_usage {
        println!();
        println!("  Current Usage (period: {}):", usage.period);
        println!(
            "    {:<25} {:>15} {:>10} {:>10}",
            "Meter", "Quantity", "Quota", "Usage %"
        );
        println!("    {}", "-".repeat(65));
        for entry in &usage.entries {
            let quota_str = entry.quota.map(format_number).unwrap_or_else(|| "-".into());
            let pct_str = if entry.quota.is_some() {
                format!("{:.1}%", entry.usage_percent())
            } else {
                "-".into()
            };
            println!(
                "    {:<25} {:>15} {:>10} {:>10}",
                entry.meter.as_str(),
                format_number(entry.quantity),
                quota_str,
                pct_str,
            );
        }
    }

    // Billing summary
    println!();
    println!("  Billing Summary:");
    println!(
        "    Total billed:     ${}",
        format_cents(summary.total_billed_cents)
    );
    println!(
        "    Total paid:       ${}",
        format_cents(summary.total_paid_cents)
    );
    println!(
        "    Outstanding:      ${}",
        format_cents(summary.outstanding_cents)
    );

    // Invoices
    if !summary.invoices.is_empty() {
        println!();
        println!("  Invoices:");
        println!(
            "    {:<38} {:<10} {:>12} {:<10} Due Date",
            "Invoice ID", "Period", "Amount", "Status"
        );
        println!("    {}", "-".repeat(90));
        for inv in &summary.invoices {
            println!(
                "    {:<38} {:<10} {:>12} {:<10} {}",
                inv.invoice_id,
                inv.period,
                format!("${}", format_cents(inv.total_cents)),
                inv.status,
                inv.due_at.format("%Y-%m-%d"),
            );
        }
    }

    // Payments
    if !summary.payments.is_empty() {
        println!();
        println!("  Payments:");
        println!(
            "    {:<38} {:>12} {:<12} {:<25} Date",
            "Payment ID", "Amount", "Status", "Method"
        );
        println!("    {}", "-".repeat(100));
        for pay in &summary.payments {
            println!(
                "    {:<38} {:>12} {:<12} {:<25} {}",
                pay.payment_id,
                format!("${}", format_cents(pay.amount_cents)),
                pay.status,
                truncate(&pay.method, 23),
                pay.created_at.format("%Y-%m-%d"),
            );
        }
    }
}

fn dashboard_usage(engine: &DashboardEngine) {
    let installations = engine.list_installations();

    println!("=== Usage Across All Installations ===");
    println!();

    for inst in &installations {
        if let Some(usage) = engine.get_latest_usage(inst.installation_id) {
            println!(
                "  {} ({:?}, {} nodes) - period: {}",
                inst.tenant_name, inst.license.tier, inst.node_count, usage.period
            );
            for entry in &usage.entries {
                let quota_str = entry
                    .quota
                    .map(|q| {
                        let pct = entry.usage_percent();
                        let indicator = if pct > 90.0 {
                            " [CRITICAL]"
                        } else if pct > 75.0 {
                            " [WARNING]"
                        } else {
                            ""
                        };
                        format!(
                            "{}/{} ({:.0}%){}",
                            format_number(entry.quantity),
                            format_number(q),
                            pct,
                            indicator
                        )
                    })
                    .unwrap_or_else(|| format_number(entry.quantity));
                println!("    {:<25} {}", entry.meter.as_str(), quota_str,);
            }
            println!();
        }
    }
}

fn dashboard_billing(engine: &DashboardEngine) {
    let installations = engine.list_installations();

    println!("=== Billing Summary ===");
    println!();

    let mut grand_total_billed: u64 = 0;
    let mut grand_total_paid: u64 = 0;
    let mut grand_total_outstanding: u64 = 0;

    println!(
        "  {:<20} {:<15} {:>14} {:>14} {:>14}",
        "Tenant", "Tier", "Total Billed", "Total Paid", "Outstanding"
    );
    println!("  {}", "-".repeat(82));

    for inst in &installations {
        if let Some(summary) = engine.installation_summary(inst.installation_id) {
            grand_total_billed += summary.total_billed_cents;
            grand_total_paid += summary.total_paid_cents;
            grand_total_outstanding += summary.outstanding_cents;

            println!(
                "  {:<20} {:<15} {:>14} {:>14} {:>14}",
                truncate(&inst.tenant_name, 18),
                format!("{:?}", inst.license.tier),
                format!("${}", format_cents(summary.total_billed_cents)),
                format!("${}", format_cents(summary.total_paid_cents)),
                format!("${}", format_cents(summary.outstanding_cents)),
            );
        }
    }

    println!("  {}", "-".repeat(82));
    println!(
        "  {:<20} {:<15} {:>14} {:>14} {:>14}",
        "TOTAL",
        "",
        format!("${}", format_cents(grand_total_billed)),
        format!("${}", format_cents(grand_total_paid)),
        format!("${}", format_cents(grand_total_outstanding)),
    );

    // Detailed invoice list
    println!();
    println!("  All Invoices:");
    println!(
        "    {:<20} {:<10} {:>12} {:<10} {:<12} Paid",
        "Tenant", "Period", "Amount", "Status", "Due"
    );
    println!("    {}", "-".repeat(80));

    let mut all_invoices: Vec<_> = Vec::new();
    for inst in &installations {
        let invs = engine.get_invoices_for_installation(inst.installation_id);
        all_invoices.extend(invs);
    }
    all_invoices.sort_by(|a, b| b.issued_at.cmp(&a.issued_at));

    for inv in &all_invoices {
        let paid_str = inv
            .paid_at
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "-".into());

        let status_str = match inv.status {
            InvoiceStatus::PastDue => "PAST DUE",
            InvoiceStatus::Open if inv.due_at < Utc::now() => "OVERDUE",
            _ => match inv.status {
                InvoiceStatus::Draft => "Draft",
                InvoiceStatus::Open => "Open",
                InvoiceStatus::Paid => "Paid",
                InvoiceStatus::PastDue => "Past Due",
                InvoiceStatus::Void => "Void",
            },
        };

        println!(
            "    {:<20} {:<10} {:>12} {:<10} {:<12} {}",
            truncate(&inv.tenant_name, 18),
            inv.period,
            format!("${}", format_cents(inv.total_cents)),
            status_str,
            inv.due_at.format("%Y-%m-%d"),
            paid_str,
        );
    }
}

fn dashboard_payments(engine: &DashboardEngine) {
    println!("=== Payments & Outstanding ===");
    println!();

    // Pending payments
    let pending = engine.get_pending_payments();
    println!("  Pending Payments ({}):", pending.len());
    if pending.is_empty() {
        println!("    No pending payments.");
    } else {
        println!(
            "    {:<20} {:>12} {:<25} Since",
            "Tenant", "Amount", "Method"
        );
        println!("    {}", "-".repeat(70));
        for p in &pending {
            println!(
                "    {:<20} {:>12} {:<25} {}",
                truncate(&p.tenant_name, 18),
                format!("${}", format_cents(p.amount_cents)),
                truncate(&p.method, 23),
                p.created_at.format("%Y-%m-%d"),
            );
        }
    }

    // Overdue invoices
    println!();
    let overdue = engine.get_overdue_invoices();
    println!("  Overdue Invoices ({}):", overdue.len());
    if overdue.is_empty() {
        println!("    No overdue invoices.");
    } else {
        println!(
            "    {:<20} {:<10} {:>12} {:<12} Days Overdue",
            "Tenant", "Period", "Amount", "Due Date"
        );
        println!("    {}", "-".repeat(70));
        for inv in &overdue {
            let days_overdue = (Utc::now() - inv.due_at).num_days();
            println!(
                "    {:<20} {:<10} {:>12} {:<12} {}",
                truncate(&inv.tenant_name, 18),
                inv.period,
                format!("${}", format_cents(inv.total_cents)),
                inv.due_at.format("%Y-%m-%d"),
                days_overdue,
            );
        }
    }

    // Recent completed payments
    println!();
    let installations = engine.list_installations();
    let mut all_payments: Vec<_> = Vec::new();
    for inst in &installations {
        all_payments.extend(engine.get_payments_for_installation(inst.installation_id));
    }
    all_payments.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let completed: Vec<_> = all_payments
        .iter()
        .filter(|p| p.status == PaymentStatus::Completed)
        .take(10)
        .collect();

    println!("  Recent Completed Payments ({} shown):", completed.len());
    if completed.is_empty() {
        println!("    No completed payments.");
    } else {
        println!(
            "    {:<20} {:>12} {:<25} {:<15} Date",
            "Tenant", "Amount", "Method", "Reference"
        );
        println!("    {}", "-".repeat(85));
        for p in &completed {
            println!(
                "    {:<20} {:>12} {:<25} {:<15} {}",
                truncate(&p.tenant_name, 18),
                format!("${}", format_cents(p.amount_cents)),
                truncate(&p.method, 23),
                p.reference.as_deref().unwrap_or("-"),
                p.created_at.format("%Y-%m-%d"),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn format_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_cents(cents: u64) -> String {
    let dollars = cents / 100;
    let remainder = cents % 100;
    if dollars >= 1_000 {
        format!(
            "{},{:03}.{:02}",
            dollars / 1_000,
            dollars % 1_000,
            remainder
        )
    } else {
        format!("{dollars}.{remainder:02}")
    }
}

// ---------------------------------------------------------------------------
// Admin console commands
// ---------------------------------------------------------------------------

fn cmd_admin(action: AdminAction) {
    // Bootstrap subsystems with demo data
    let tenant_mgr = TenantManager::new();
    tenant_mgr.seed_demo_tenants();
    let billing = BillingEngine::new();
    billing.seed_demo_data();
    let auth = AuthManager::new();
    auth.seed_demo_providers();
    let rbac = RbacEngine::new();
    rbac.seed_default_roles();
    let feature_flags = FeatureFlagManager::new();
    feature_flags.seed_defaults();
    let notifications = NotificationManager::new();
    notifications.seed_demo();
    let settings = SystemSettings::new();

    match action {
        AdminAction::Overview => cmd_admin_overview(&tenant_mgr, &billing),
        AdminAction::Tenants => cmd_admin_tenants(&tenant_mgr),
        AdminAction::SuspendTenant { id, reason } => {
            cmd_admin_suspend(&tenant_mgr, &id, &reason);
        }
        AdminAction::ReactivateTenant { id, reason } => {
            cmd_admin_reactivate(&tenant_mgr, &id, &reason);
        }
        AdminAction::Users { tenant_id } => cmd_admin_users(&tenant_mgr, &auth, &rbac, &tenant_id),
        AdminAction::FeatureFlags => cmd_admin_feature_flags(&feature_flags),
        AdminAction::Notifications => cmd_admin_notifications(&notifications),
        AdminAction::Settings => cmd_admin_settings(&settings),
        AdminAction::Maintenance { mode, message } => {
            cmd_admin_maintenance(&settings, &mode, message);
        }
    }
}

fn cmd_admin_overview(tenant_mgr: &TenantManager, billing: &BillingEngine) {
    let dashboard = ProviderDashboard::new(tenant_mgr, billing);
    let overview = dashboard.overview();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              CAMPAIGN EXPRESS — PROVIDER OVERVIEW           ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    println!("  TENANTS");
    println!("  ─────────────────────────────────────────");
    println!(
        "  Total: {}    Active: {}    Suspended: {}    Trial: {}    Cancelled: {}",
        overview.total_tenants,
        overview.active_tenants,
        overview.suspended_tenants,
        overview.trial_tenants,
        overview.cancelled_tenants
    );
    println!();

    println!("  TIER BREAKDOWN");
    println!("  ─────────────────────────────────────────");
    let tiers = &overview.tenants_by_tier;
    println!(
        "  Free: {}    Starter: {}    Professional: {}    Enterprise: {}    Custom: {}",
        tiers.free, tiers.starter, tiers.professional, tiers.enterprise, tiers.custom
    );
    println!();

    println!("  BILLING");
    println!("  ─────────────────────────────────────────");
    let b = &overview.billing_summary;
    println!(
        "  MRR: ${:.2}    ARR: ${:.2}    Active Subscriptions: {}",
        b.total_mrr, b.total_arr, b.active_subscriptions
    );
    println!(
        "  Open Invoices: {} (${:.2})    Paid: {}    Total Revenue: ${:.2}",
        b.open_invoices, b.open_invoice_amount, b.paid_invoices, b.total_revenue
    );
    println!();

    println!("  USAGE (TODAY)");
    println!("  ─────────────────────────────────────────");
    let u = &overview.usage_summary;
    println!(
        "  Offers Served: {}    API Calls: {}    Active Campaigns: {}",
        format_number(u.total_offers_today),
        format_number(u.total_api_calls_today),
        u.total_campaigns_active
    );
    println!(
        "  Total Users: {}    Storage: {} bytes",
        u.total_users, u.total_storage_bytes
    );
    println!();

    // Tenant table
    let rows = dashboard.tenant_table();
    println!("  TENANT TABLE");
    println!("  ─────────────────────────────────────────");
    println!(
        "  {:<20} {:<14} {:<10} {:<6} {:<6} {:<12} {:<8}",
        "Name", "Tier", "Status", "Users", "Camps", "Offers/Day", "MRR"
    );
    for row in &rows {
        println!(
            "  {:<20} {:<14} {:<10} {:<6} {:<6} {:<12} ${:<.2}",
            truncate(&row.name, 18),
            format!("{:?}", row.tier),
            format!("{:?}", row.status),
            row.users,
            row.campaigns,
            format_number(row.offers_today),
            row.mrr
        );
    }
    println!();
}

fn cmd_admin_tenants(tenant_mgr: &TenantManager) {
    let ops = TenantOps::new(tenant_mgr);
    let tenants = ops.list_all();

    println!(
        "╔══════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!("║                              ALL TENANTS                                       ║");
    println!(
        "╚══════════════════════════════════════════════════════════════════════════════════╝"
    );
    println!();
    println!(
        "  {:<36}  {:<20}  {:<14}  {:<10}  {:<6}  {:<10}",
        "ID", "Name", "Tier", "Status", "Users", "Offers/Day"
    );
    for t in &tenants {
        println!(
            "  {:<36}  {:<20}  {:<14}  {:<10}  {:<6}  {:<10}",
            t.id,
            truncate(&t.name, 18),
            format!("{:?}", t.pricing_tier),
            format!("{:?}", t.status),
            t.usage.users_count,
            format_number(t.usage.offers_served_today)
        );
    }
    println!();
    println!("  Total: {} tenants", tenants.len());

    // Quota alerts
    let alerts = ops.tenants_near_quota(80.0);
    if !alerts.is_empty() {
        println!();
        println!("  ⚠ QUOTA ALERTS (>80% usage)");
        println!("  ─────────────────────────────────────────");
        for alert in &alerts {
            let resources: Vec<_> = alert
                .alerts
                .iter()
                .map(|(r, pct)| format!("{r}: {pct:.1}%"))
                .collect();
            println!("  {} — {}", alert.tenant_name, resources.join(", "));
        }
    }
    println!();
}

fn cmd_admin_suspend(tenant_mgr: &TenantManager, id: &str, reason: &str) {
    let ops = TenantOps::new(tenant_mgr);
    let tenant_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: invalid UUID '{id}'");
            return;
        }
    };

    let action_reason =
        campaign_admin_console::tenant_ops::ActionReason::new(Uuid::new_v4(), reason);
    match ops.suspend(tenant_id, action_reason) {
        Ok(result) => {
            println!("Tenant {} suspended.", tenant_id);
            println!(
                "  Previous status: {:?} → New status: {:?}",
                result.previous_status, result.new_status
            );
            println!("  Reason: {reason}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn cmd_admin_reactivate(tenant_mgr: &TenantManager, id: &str, reason: &str) {
    let ops = TenantOps::new(tenant_mgr);
    let tenant_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: invalid UUID '{id}'");
            return;
        }
    };

    let action_reason =
        campaign_admin_console::tenant_ops::ActionReason::new(Uuid::new_v4(), reason);
    match ops.reactivate(tenant_id, action_reason) {
        Ok(result) => {
            println!("Tenant {} reactivated.", tenant_id);
            println!(
                "  Previous status: {:?} → New status: {:?}",
                result.previous_status, result.new_status
            );
            println!("  Reason: {reason}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn cmd_admin_users(
    tenant_mgr: &TenantManager,
    auth: &AuthManager,
    rbac: &RbacEngine,
    tenant_id_str: &str,
) {
    let tenant_id = match Uuid::parse_str(tenant_id_str) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: invalid UUID '{tenant_id_str}'");
            return;
        }
    };

    let tenant = tenant_mgr.get_tenant(tenant_id);
    let tenant_name = tenant
        .as_ref()
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown");

    let user_ops = UserOps::new(auth, rbac);
    user_ops.seed_demo_users(tenant_id);
    let users = user_ops.list_users(tenant_id);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  USERS — {:<50} ║", truncate(tenant_name, 48));
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "  {:<36}  {:<20}  {:<25}  {:<10}  {:<10}",
        "ID", "Name", "Email", "Status", "Provider"
    );
    for u in &users {
        println!(
            "  {:<36}  {:<20}  {:<25}  {:<10}  {:<10}",
            u.id,
            truncate(&u.display_name, 18),
            truncate(&u.email, 23),
            format!("{:?}", u.status),
            format!("{:?}", u.auth_provider)
        );
    }
    println!();
    println!("  Total: {} users", users.len());

    // Show roles
    let roles = rbac.list_roles();
    println!();
    println!("  AVAILABLE ROLES");
    println!("  ─────────────────────────────────────────");
    for r in &roles {
        let system_tag = if r.is_system { " [system]" } else { "" };
        println!(
            "  {:<20} — {} ({} permissions){}",
            r.name,
            r.description,
            r.permissions.len(),
            system_tag
        );
    }

    // Show pending invitations
    let invitations = user_ops.list_invitations(tenant_id);
    if !invitations.is_empty() {
        println!();
        println!("  PENDING INVITATIONS");
        println!("  ─────────────────────────────────────────");
        for inv in &invitations {
            println!(
                "  {} — {:?} (expires {})",
                inv.email,
                inv.status,
                inv.expires_at.format("%Y-%m-%d")
            );
        }
    }
    println!();
}

fn cmd_admin_feature_flags(flags: &FeatureFlagManager) {
    let all_flags = flags.list_flags();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                       FEATURE FLAGS                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "  {:<25} {:<12} {:<8} {:<8} Description",
        "Key", "Strategy", "Pct", "Tenants"
    );
    for f in &all_flags {
        let pct_str = f
            .percentage
            .map(|p| format!("{p}%"))
            .unwrap_or_else(|| "-".into());
        let tenants_str = if f.allowed_tenants.is_empty() {
            "-".into()
        } else {
            format!("{}", f.allowed_tenants.len())
        };
        println!(
            "  {:<25} {:<12} {:<8} {:<8} {}",
            f.key,
            format!("{:?}", f.strategy),
            pct_str,
            tenants_str,
            truncate(&f.description, 40)
        );
    }
    println!();
    println!("  Total: {} flags", all_flags.len());
    println!();
}

fn cmd_admin_notifications(notifications: &NotificationManager) {
    let unread = notifications.unread(None);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     NOTIFICATION INBOX                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    if unread.is_empty() {
        println!("  No unread notifications.");
    } else {
        for n in &unread {
            let sev = match n.severity {
                Severity::Critical => "CRIT",
                Severity::Error => "ERR ",
                Severity::Warning => "WARN",
                Severity::Info => "INFO",
            };
            let tenant = n
                .tenant_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "GLOBAL".into());
            println!("  [{sev}] {}", n.title);
            println!(
                "         Category: {:?}  Tenant: {}  {}",
                n.category,
                truncate(&tenant, 12),
                n.created_at.format("%Y-%m-%d %H:%M")
            );
            println!("         {}", n.message);
            let channels: Vec<_> = n.channels.iter().map(|c| format!("{c:?}")).collect();
            println!("         Channels: {}", channels.join(", "));
            println!();
        }
        println!("  {} unread notifications", unread.len());
    }
    println!();
}

fn cmd_admin_settings(settings: &SystemSettings) {
    let cfg = settings.get_config();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     SYSTEM SETTINGS                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  GENERAL");
    println!("  ─────────────────────────────────────────");
    println!("  Platform:            {}", cfg.platform_name);
    println!("  Version:             {}", cfg.platform_version);
    println!("  Environment:         {:?}", cfg.environment);
    println!("  Maintenance Mode:    {}", cfg.maintenance_mode);
    if let Some(ref msg) = cfg.maintenance_message {
        println!("  Maintenance Message: {msg}");
    }
    println!();

    println!("  LIMITS");
    println!("  ─────────────────────────────────────────");
    println!("  Rate Limit (RPS):    {}", cfg.default_rate_limit_rps);
    println!("  Rate Limit (RPM):    {}", cfg.default_rate_limit_rpm);
    println!("  Max Tenants:         {}", cfg.max_tenants);
    println!("  Data Retention:      {} days", cfg.data_retention_days);
    println!();

    println!("  SECURITY");
    println!("  ─────────────────────────────────────────");
    println!("  Self Registration:   {}", cfg.allow_self_registration);
    println!("  Email Verification:  {}", cfg.require_email_verification);
    println!("  MFA Required:        {}", cfg.mfa_required);
    println!("  Min Password Length: {}", cfg.password_min_length);
    println!("  Session TTL:         {} hours", cfg.session_ttl_hours);
    println!("  API Key TTL:         {} days", cfg.api_key_ttl_days);
    println!();

    let changes = settings.full_change_log();
    if !changes.is_empty() {
        println!("  RECENT CHANGES");
        println!("  ─────────────────────────────────────────");
        for c in changes.iter().take(10) {
            println!(
                "  {} — {} → {} (by {})",
                c.key, c.old_value, c.new_value, c.changed_by
            );
        }
        println!();
    }
}

fn cmd_admin_maintenance(settings: &SystemSettings, mode: &str, message: Option<String>) {
    match mode.to_lowercase().as_str() {
        "on" | "enable" | "true" => {
            settings.enable_maintenance(message.clone(), "admin-cli");
            println!("Maintenance mode ENABLED.");
            if let Some(msg) = message {
                println!("  Message: {msg}");
            }
        }
        "off" | "disable" | "false" => {
            settings.disable_maintenance("admin-cli");
            println!("Maintenance mode DISABLED.");
        }
        _ => {
            eprintln!("Error: mode must be 'on' or 'off'");
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if max < 3 {
        return s.chars().take(max).collect();
    }
    let char_count = s.chars().count();
    if char_count > max {
        let truncated: String = s.chars().take(max - 2).collect();
        format!("{truncated}..")
    } else {
        s.to_string()
    }
}
