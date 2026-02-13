//! License Admin CLI — generate keys, create licenses, and verify license files.

use campaign_licensing::{
    License, LicenseKey, LicenseTier, LicenseType, LicensedModule,
    sign_license, verify_license,
};
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
        Commands::GenerateKey { output } => {
            let key = LicenseKey::generate();
            let b64 = key.to_base64();

            if let Some(path) = output {
                std::fs::write(&path, &b64).expect("Failed to write key file");
                println!("Signing key written to: {path}");
            } else {
                println!("{b64}");
            }
        }

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
        } => {
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

        Commands::Verify { key, license } => {
            let signing_key =
                LicenseKey::load_from_file(std::path::Path::new(&key)).expect("Failed to load key");
            let contents =
                std::fs::read_to_string(&license).expect("Failed to read license file");

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
                    println!("  Issued at:        {}", lic.issued_at.format("%Y-%m-%d %H:%M UTC"));
                    println!("  Expires at:       {}", lic.expires_at.format("%Y-%m-%d %H:%M UTC"));
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

        Commands::ListModules => {
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
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        n.to_string()
    }
}
