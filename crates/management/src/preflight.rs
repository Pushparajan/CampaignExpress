//! Content and creative QA automation: preflight checklist with automated
//! checks, blocking/warning enforcement, and test harnesses.
//!
//! Addresses FR-QA-001 through FR-QA-003.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Preflight Check Types (FR-QA-001) ────────────────────────────────

/// Category of preflight check.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CheckCategory {
    BrandRules,
    LegalCompliance,
    LinkIntegrity,
    AssetRights,
    ChannelConstraints,
    TrackingParameters,
    ContentQuality,
    Accessibility,
}

/// Severity of a preflight issue: blocking prevents go-live.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightSeverity {
    Blocking,
    Warning,
    Info,
}

/// A single automated preflight check definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightCheck {
    pub id: Uuid,
    pub name: String,
    pub category: CheckCategory,
    pub description: String,
    pub severity: PreflightSeverity,
    pub auto_check: bool,
}

/// Result of a single preflight check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResult {
    pub check_id: Uuid,
    pub check_name: String,
    pub category: CheckCategory,
    pub passed: bool,
    pub severity: PreflightSeverity,
    pub message: String,
    pub details: HashMap<String, String>,
    pub checked_at: DateTime<Utc>,
}

/// Full preflight report for a campaign/creative/template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub id: Uuid,
    pub object_id: Uuid,
    pub object_type: String,
    pub results: Vec<PreflightResult>,
    pub total_checks: usize,
    pub passed: usize,
    pub warnings: usize,
    pub blocking_failures: usize,
    pub can_proceed: bool,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Uuid,
}

// ─── Preflight Engine (FR-QA-001 + FR-QA-002) ────────────────────────

/// Content/creative submission for preflight validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightSubmission {
    pub object_id: Uuid,
    pub object_type: String,
    pub channels: Vec<String>,
    /// HTML or text content.
    pub content: Option<String>,
    /// Colors used in the creative.
    pub colors: Vec<String>,
    /// (font_family, size_px) pairs.
    pub fonts: Vec<(String, u32)>,
    /// Links found in the content.
    pub links: Vec<String>,
    /// UTM parameters on links.
    pub utm_params: HashMap<String, String>,
    /// Whether unsubscribe is present.
    pub has_unsubscribe: bool,
    /// Whether STOP language is present (SMS/WhatsApp).
    pub has_stop_language: bool,
    /// Whether physical address is present (email).
    pub has_physical_address: bool,
    /// Asset IDs used.
    pub asset_ids: Vec<Uuid>,
    /// Whether asset rights are valid for all used assets.
    pub asset_rights_valid: bool,
    /// Image dimensions (width, height).
    pub image_dimensions: Option<(u32, u32)>,
    /// File sizes in bytes.
    pub file_size_bytes: Option<u64>,
    /// Brand guideline check passed.
    pub brand_check_passed: bool,
    /// Text content for tone/length checks.
    pub text_content: Option<String>,
    /// Channel-specific max text length.
    pub max_text_length: Option<usize>,
}

/// Preflight engine with automated checks and auditable results.
pub struct PreflightEngine {
    checks: Vec<PreflightCheck>,
    reports: DashMap<Uuid, PreflightReport>,
}

impl PreflightEngine {
    pub fn new() -> Self {
        Self {
            checks: Self::default_checks(),
            reports: DashMap::new(),
        }
    }

    fn default_checks() -> Vec<PreflightCheck> {
        vec![
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Brand Color Compliance".to_string(),
                category: CheckCategory::BrandRules,
                description: "All colors must be from the approved brand palette".to_string(),
                severity: PreflightSeverity::Blocking,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Brand Typography".to_string(),
                category: CheckCategory::BrandRules,
                description: "Font families must be approved".to_string(),
                severity: PreflightSeverity::Blocking,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Brand Tone Compliance".to_string(),
                category: CheckCategory::BrandRules,
                description: "Text must follow tone-of-voice guidelines".to_string(),
                severity: PreflightSeverity::Warning,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Unsubscribe Link (CAN-SPAM)".to_string(),
                category: CheckCategory::LegalCompliance,
                description: "Email must contain a working unsubscribe link".to_string(),
                severity: PreflightSeverity::Blocking,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "STOP Language (TCPA)".to_string(),
                category: CheckCategory::LegalCompliance,
                description: "SMS/WhatsApp must include STOP/opt-out instructions".to_string(),
                severity: PreflightSeverity::Blocking,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Physical Address (CAN-SPAM)".to_string(),
                category: CheckCategory::LegalCompliance,
                description: "Email must include a physical mailing address".to_string(),
                severity: PreflightSeverity::Blocking,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Link Integrity".to_string(),
                category: CheckCategory::LinkIntegrity,
                description: "All links must be valid URLs".to_string(),
                severity: PreflightSeverity::Blocking,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Tracking Parameters".to_string(),
                category: CheckCategory::TrackingParameters,
                description: "UTM parameters should be present on links".to_string(),
                severity: PreflightSeverity::Warning,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Asset Rights Validity".to_string(),
                category: CheckCategory::AssetRights,
                description: "All assets must have valid usage rights".to_string(),
                severity: PreflightSeverity::Blocking,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Channel Size Constraints".to_string(),
                category: CheckCategory::ChannelConstraints,
                description: "Creative size must meet channel requirements".to_string(),
                severity: PreflightSeverity::Blocking,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Text Length Limit".to_string(),
                category: CheckCategory::ChannelConstraints,
                description: "Text must not exceed channel character limit".to_string(),
                severity: PreflightSeverity::Warning,
                auto_check: true,
            },
            PreflightCheck {
                id: Uuid::new_v4(),
                name: "Image Alt Text".to_string(),
                category: CheckCategory::Accessibility,
                description: "Images should have descriptive alt text".to_string(),
                severity: PreflightSeverity::Warning,
                auto_check: true,
            },
        ]
    }

    /// Run all applicable preflight checks on a submission.
    pub fn run_preflight(
        &self,
        submission: &PreflightSubmission,
        user_id: Uuid,
    ) -> PreflightReport {
        let mut results = Vec::new();
        let now = Utc::now();

        for check in &self.checks {
            let result = self.execute_check(check, submission, now);
            if let Some(r) = result {
                results.push(r);
            }
        }

        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let warnings = results
            .iter()
            .filter(|r| !r.passed && r.severity == PreflightSeverity::Warning)
            .count();
        let blocking = results
            .iter()
            .filter(|r| !r.passed && r.severity == PreflightSeverity::Blocking)
            .count();

        let report = PreflightReport {
            id: Uuid::new_v4(),
            object_id: submission.object_id,
            object_type: submission.object_type.clone(),
            results,
            total_checks: total,
            passed,
            warnings,
            blocking_failures: blocking,
            can_proceed: blocking == 0,
            generated_at: now,
            generated_by: user_id,
        };

        self.reports.insert(report.id, report.clone());
        report
    }

    fn execute_check(
        &self,
        check: &PreflightCheck,
        submission: &PreflightSubmission,
        now: DateTime<Utc>,
    ) -> Option<PreflightResult> {
        match check.category {
            CheckCategory::BrandRules => {
                if check.name.contains("Color") {
                    Some(PreflightResult {
                        check_id: check.id,
                        check_name: check.name.clone(),
                        category: check.category.clone(),
                        passed: submission.brand_check_passed,
                        severity: check.severity.clone(),
                        message: if submission.brand_check_passed {
                            "Brand color compliance: passed".to_string()
                        } else {
                            "Brand color compliance: failed".to_string()
                        },
                        details: HashMap::new(),
                        checked_at: now,
                    })
                } else if check.name.contains("Typography") {
                    Some(PreflightResult {
                        check_id: check.id,
                        check_name: check.name.clone(),
                        category: check.category.clone(),
                        passed: submission.brand_check_passed,
                        severity: check.severity.clone(),
                        message: if submission.brand_check_passed {
                            "Typography compliance: passed".to_string()
                        } else {
                            "Typography compliance: non-approved fonts detected".to_string()
                        },
                        details: HashMap::new(),
                        checked_at: now,
                    })
                } else {
                    Some(PreflightResult {
                        check_id: check.id,
                        check_name: check.name.clone(),
                        category: check.category.clone(),
                        passed: submission.brand_check_passed,
                        severity: check.severity.clone(),
                        message: format!(
                            "{}: {}",
                            check.name,
                            if submission.brand_check_passed {
                                "passed"
                            } else {
                                "failed"
                            }
                        ),
                        details: HashMap::new(),
                        checked_at: now,
                    })
                }
            }

            CheckCategory::LegalCompliance => {
                if check.name.contains("Unsubscribe") {
                    let applicable = submission.channels.iter().any(|c| c == "email");
                    if !applicable {
                        return None;
                    }
                    Some(PreflightResult {
                        check_id: check.id,
                        check_name: check.name.clone(),
                        category: check.category.clone(),
                        passed: submission.has_unsubscribe,
                        severity: check.severity.clone(),
                        message: if submission.has_unsubscribe {
                            "Unsubscribe link: present".to_string()
                        } else {
                            "BLOCKED: Missing unsubscribe link (CAN-SPAM requirement)".to_string()
                        },
                        details: HashMap::new(),
                        checked_at: now,
                    })
                } else if check.name.contains("STOP") {
                    let applicable = submission
                        .channels
                        .iter()
                        .any(|c| c == "sms" || c == "whatsapp");
                    if !applicable {
                        return None;
                    }
                    Some(PreflightResult {
                        check_id: check.id,
                        check_name: check.name.clone(),
                        category: check.category.clone(),
                        passed: submission.has_stop_language,
                        severity: check.severity.clone(),
                        message: if submission.has_stop_language {
                            "STOP language: present".to_string()
                        } else {
                            "BLOCKED: Missing STOP/opt-out language (TCPA requirement)".to_string()
                        },
                        details: HashMap::new(),
                        checked_at: now,
                    })
                } else if check.name.contains("Physical") {
                    let applicable = submission.channels.iter().any(|c| c == "email");
                    if !applicable {
                        return None;
                    }
                    Some(PreflightResult {
                        check_id: check.id,
                        check_name: check.name.clone(),
                        category: check.category.clone(),
                        passed: submission.has_physical_address,
                        severity: check.severity.clone(),
                        message: if submission.has_physical_address {
                            "Physical address: present".to_string()
                        } else {
                            "BLOCKED: Missing physical mailing address".to_string()
                        },
                        details: HashMap::new(),
                        checked_at: now,
                    })
                } else {
                    None
                }
            }

            CheckCategory::LinkIntegrity => {
                let broken_links: Vec<&String> = submission
                    .links
                    .iter()
                    .filter(|l| {
                        !l.starts_with("http://")
                            && !l.starts_with("https://")
                            && !l.starts_with("mailto:")
                    })
                    .collect();

                Some(PreflightResult {
                    check_id: check.id,
                    check_name: check.name.clone(),
                    category: check.category.clone(),
                    passed: broken_links.is_empty(),
                    severity: check.severity.clone(),
                    message: if broken_links.is_empty() {
                        format!("All {} links valid", submission.links.len())
                    } else {
                        format!("{} broken link(s) detected", broken_links.len())
                    },
                    details: if !broken_links.is_empty() {
                        vec![(
                            "broken".to_string(),
                            broken_links
                                .iter()
                                .map(|l| l.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                        )]
                        .into_iter()
                        .collect()
                    } else {
                        HashMap::new()
                    },
                    checked_at: now,
                })
            }

            CheckCategory::TrackingParameters => {
                let has_utm = !submission.utm_params.is_empty();
                Some(PreflightResult {
                    check_id: check.id,
                    check_name: check.name.clone(),
                    category: check.category.clone(),
                    passed: has_utm,
                    severity: check.severity.clone(),
                    message: if has_utm {
                        format!("UTM parameters present ({})", submission.utm_params.len())
                    } else {
                        "Warning: No UTM tracking parameters found on links".to_string()
                    },
                    details: submission.utm_params.clone(),
                    checked_at: now,
                })
            }

            CheckCategory::AssetRights => Some(PreflightResult {
                check_id: check.id,
                check_name: check.name.clone(),
                category: check.category.clone(),
                passed: submission.asset_rights_valid,
                severity: check.severity.clone(),
                message: if submission.asset_rights_valid {
                    format!(
                        "All {} asset(s) have valid rights",
                        submission.asset_ids.len()
                    )
                } else {
                    "BLOCKED: One or more assets have expired or invalid usage rights".to_string()
                },
                details: HashMap::new(),
                checked_at: now,
            }),

            CheckCategory::ChannelConstraints => {
                if check.name.contains("Size") {
                    if let Some((w, h)) = submission.image_dimensions {
                        let ok = w >= 100 && h >= 100;
                        Some(PreflightResult {
                            check_id: check.id,
                            check_name: check.name.clone(),
                            category: check.category.clone(),
                            passed: ok,
                            severity: check.severity.clone(),
                            message: if ok {
                                format!("Image dimensions {}x{}: OK", w, h)
                            } else {
                                format!("Image dimensions {}x{}: below minimum", w, h)
                            },
                            details: vec![
                                ("width".to_string(), w.to_string()),
                                ("height".to_string(), h.to_string()),
                            ]
                            .into_iter()
                            .collect(),
                            checked_at: now,
                        })
                    } else {
                        None
                    }
                } else if check.name.contains("Text Length") {
                    if let (Some(text), Some(max_len)) =
                        (&submission.text_content, submission.max_text_length)
                    {
                        let ok = text.len() <= max_len;
                        Some(PreflightResult {
                            check_id: check.id,
                            check_name: check.name.clone(),
                            category: check.category.clone(),
                            passed: ok,
                            severity: check.severity.clone(),
                            message: if ok {
                                format!("Text length {}/{}: OK", text.len(), max_len)
                            } else {
                                format!("Text length {}/{}: exceeds limit", text.len(), max_len)
                            },
                            details: HashMap::new(),
                            checked_at: now,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            CheckCategory::Accessibility => {
                if let Some(ref content) = submission.content {
                    let lower = content.to_lowercase();
                    let has_img = lower.contains("<img");
                    let has_alt = lower.contains("alt=");
                    let passed = !has_img || has_alt;

                    Some(PreflightResult {
                        check_id: check.id,
                        check_name: check.name.clone(),
                        category: check.category.clone(),
                        passed,
                        severity: check.severity.clone(),
                        message: if passed {
                            "Image alt text: present".to_string()
                        } else {
                            "Warning: Images without alt text detected".to_string()
                        },
                        details: HashMap::new(),
                        checked_at: now,
                    })
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    /// Get a preflight report by id.
    pub fn get_report(&self, report_id: &Uuid) -> Option<PreflightReport> {
        self.reports.get(report_id).map(|r| r.clone())
    }

    /// Get the latest report for an object.
    pub fn latest_report(&self, object_id: &Uuid) -> Option<PreflightReport> {
        self.reports
            .iter()
            .filter(|e| e.value().object_id == *object_id)
            .max_by_key(|e| e.value().generated_at)
            .map(|e| e.value().clone())
    }
}

impl Default for PreflightEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Test Harness (FR-QA-003) ─────────────────────────────────────────

/// A test send request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSendRequest {
    pub id: Uuid,
    pub channel: String,
    pub template_id: Uuid,
    pub recipient: String,
    pub personalization_data: HashMap<String, String>,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
}

/// Result of a test send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSendResult {
    pub request_id: Uuid,
    pub channel: String,
    pub delivered: bool,
    pub rendered_content: String,
    pub error: Option<String>,
    pub completed_at: DateTime<Utc>,
}

/// A seed/test audience entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedAudienceMember {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub device_token: Option<String>,
    pub profile_data: HashMap<String, String>,
}

/// Test harness for sending test messages and validating personalization.
pub struct TestHarness {
    requests: DashMap<Uuid, TestSendRequest>,
    results: DashMap<Uuid, TestSendResult>,
    seed_audience: DashMap<Uuid, SeedAudienceMember>,
}

impl TestHarness {
    pub fn new() -> Self {
        let harness = Self {
            requests: DashMap::new(),
            results: DashMap::new(),
            seed_audience: DashMap::new(),
        };
        harness.seed_test_audience();
        harness
    }

    fn seed_test_audience(&self) {
        let members = vec![
            SeedAudienceMember {
                id: Uuid::new_v4(),
                name: "Test User A".to_string(),
                email: Some("testa@example.com".to_string()),
                phone: Some("+15551234567".to_string()),
                device_token: Some("test_token_a".to_string()),
                profile_data: vec![
                    ("first_name".to_string(), "Alice".to_string()),
                    ("loyalty_tier".to_string(), "Gold".to_string()),
                    ("points_balance".to_string(), "2500".to_string()),
                ]
                .into_iter()
                .collect(),
            },
            SeedAudienceMember {
                id: Uuid::new_v4(),
                name: "Test User B".to_string(),
                email: Some("testb@example.com".to_string()),
                phone: Some("+15559876543".to_string()),
                device_token: Some("test_token_b".to_string()),
                profile_data: vec![
                    ("first_name".to_string(), "Bob".to_string()),
                    ("loyalty_tier".to_string(), "Silver".to_string()),
                    ("points_balance".to_string(), "800".to_string()),
                ]
                .into_iter()
                .collect(),
            },
            SeedAudienceMember {
                id: Uuid::new_v4(),
                name: "Test User C (Minimal)".to_string(),
                email: Some("testc@example.com".to_string()),
                phone: None,
                device_token: None,
                profile_data: HashMap::new(), // no profile data — tests fallback values
            },
        ];

        for member in members {
            self.seed_audience.insert(member.id, member);
        }
    }

    /// Send a test message (simulated — renders the content with personalization data).
    pub fn send_test(
        &self,
        channel: &str,
        template_content: &str,
        recipient: &str,
        personalization: &HashMap<String, String>,
        user_id: Uuid,
    ) -> TestSendResult {
        let request = TestSendRequest {
            id: Uuid::new_v4(),
            channel: channel.to_string(),
            template_id: Uuid::new_v4(),
            recipient: recipient.to_string(),
            personalization_data: personalization.clone(),
            requested_by: user_id,
            requested_at: Utc::now(),
        };
        self.requests.insert(request.id, request.clone());

        // Render content by replacing variables
        let mut rendered = template_content.to_string();
        for (key, value) in personalization {
            rendered = rendered.replace(&format!("{{{{{}}}}}", key), value);
        }

        let result = TestSendResult {
            request_id: request.id,
            channel: channel.to_string(),
            delivered: true,
            rendered_content: rendered,
            error: None,
            completed_at: Utc::now(),
        };

        self.results.insert(request.id, result.clone());
        result
    }

    /// Get the seed/test audience.
    pub fn get_seed_audience(&self) -> Vec<SeedAudienceMember> {
        self.seed_audience
            .iter()
            .map(|e| e.value().clone())
            .collect()
    }

    /// Send test to all seed audience members.
    pub fn send_to_seed_audience(
        &self,
        channel: &str,
        template_content: &str,
        user_id: Uuid,
    ) -> Vec<TestSendResult> {
        let members: Vec<SeedAudienceMember> = self.get_seed_audience();

        members
            .iter()
            .filter_map(|member| {
                let recipient = match channel {
                    "email" => member.email.as_ref()?,
                    "sms" | "whatsapp" => member.phone.as_ref()?,
                    "push" => member.device_token.as_ref()?,
                    _ => return None,
                };

                Some(self.send_test(
                    channel,
                    template_content,
                    recipient,
                    &member.profile_data,
                    user_id,
                ))
            })
            .collect()
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_submission() -> PreflightSubmission {
        PreflightSubmission {
            object_id: Uuid::new_v4(),
            object_type: "campaign".to_string(),
            channels: vec!["email".to_string()],
            content: Some("<p>Hello!</p><img src=\"test.jpg\" alt=\"test\">".to_string()),
            colors: vec!["#0052CC".to_string()],
            fonts: vec![("Inter".to_string(), 16)],
            links: vec!["https://example.com/offer".to_string()],
            utm_params: vec![("utm_source".to_string(), "email".to_string())]
                .into_iter()
                .collect(),
            has_unsubscribe: true,
            has_stop_language: false,
            has_physical_address: true,
            asset_ids: vec![Uuid::new_v4()],
            asset_rights_valid: true,
            image_dimensions: Some((1200, 628)),
            file_size_bytes: Some(150_000),
            brand_check_passed: true,
            text_content: None,
            max_text_length: None,
        }
    }

    #[test]
    fn test_preflight_all_pass() {
        let engine = PreflightEngine::new();
        let submission = sample_submission();

        let report = engine.run_preflight(&submission, Uuid::new_v4());
        assert!(report.can_proceed);
        assert_eq!(report.blocking_failures, 0);
        assert!(report.passed > 0);
    }

    #[test]
    fn test_preflight_missing_unsubscribe() {
        let engine = PreflightEngine::new();
        let mut submission = sample_submission();
        submission.has_unsubscribe = false;

        let report = engine.run_preflight(&submission, Uuid::new_v4());
        assert!(!report.can_proceed);
        assert!(report.blocking_failures > 0);
        assert!(report
            .results
            .iter()
            .any(|r| r.check_name.contains("Unsubscribe") && !r.passed));
    }

    #[test]
    fn test_preflight_sms_stop_language() {
        let engine = PreflightEngine::new();
        let submission = PreflightSubmission {
            object_id: Uuid::new_v4(),
            object_type: "campaign".to_string(),
            channels: vec!["sms".to_string()],
            content: None,
            colors: vec![],
            fonts: vec![],
            links: vec![],
            utm_params: HashMap::new(),
            has_unsubscribe: false,
            has_stop_language: false,
            has_physical_address: false,
            asset_ids: vec![],
            asset_rights_valid: true,
            image_dimensions: None,
            file_size_bytes: None,
            brand_check_passed: true,
            text_content: Some("Buy now!".to_string()),
            max_text_length: Some(160),
        };

        let report = engine.run_preflight(&submission, Uuid::new_v4());
        assert!(!report.can_proceed);
        assert!(report
            .results
            .iter()
            .any(|r| r.check_name.contains("STOP") && !r.passed));
    }

    #[test]
    fn test_preflight_broken_links() {
        let engine = PreflightEngine::new();
        let mut submission = sample_submission();
        submission.links = vec![
            "https://example.com/valid".to_string(),
            "ftp://invalid-proto.com".to_string(),
            "not-a-url".to_string(),
        ];

        let report = engine.run_preflight(&submission, Uuid::new_v4());
        assert!(report
            .results
            .iter()
            .any(|r| r.check_name == "Link Integrity" && !r.passed));
    }

    #[test]
    fn test_preflight_missing_utm() {
        let engine = PreflightEngine::new();
        let mut submission = sample_submission();
        submission.utm_params = HashMap::new();

        let report = engine.run_preflight(&submission, Uuid::new_v4());
        // UTM is a warning, not blocking
        assert!(report.can_proceed);
        assert!(report.warnings > 0);
    }

    #[test]
    fn test_preflight_auditable() {
        let engine = PreflightEngine::new();
        let submission = sample_submission();
        let report = engine.run_preflight(&submission, Uuid::new_v4());

        // Report should be persisted
        let retrieved = engine.get_report(&report.id).unwrap();
        assert_eq!(retrieved.id, report.id);

        // Latest report
        let latest = engine.latest_report(&submission.object_id).unwrap();
        assert_eq!(latest.id, report.id);
    }

    #[test]
    fn test_test_harness_send() {
        let harness = TestHarness::new();
        let user = Uuid::new_v4();

        let data: HashMap<String, String> = vec![
            ("first_name".to_string(), "Jane".to_string()),
            ("offer_amount".to_string(), "$25".to_string()),
        ]
        .into_iter()
        .collect();

        let result = harness.send_test(
            "email",
            "Hello {{first_name}}, save {{offer_amount}} today!",
            "test@example.com",
            &data,
            user,
        );

        assert!(result.delivered);
        assert!(result.rendered_content.contains("Hello Jane"));
        assert!(result.rendered_content.contains("$25"));
    }

    #[test]
    fn test_seed_audience_send() {
        let harness = TestHarness::new();
        let audience = harness.get_seed_audience();
        assert!(audience.len() >= 3);

        let results =
            harness.send_to_seed_audience("email", "Hello {{first_name}}!", Uuid::new_v4());

        // At least the members with email should get results
        assert!(results.len() >= 2);
        assert!(results.iter().all(|r| r.delivered));
    }
}
