# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Campaign Express, please report it responsibly. **Do not open a public issue.**

### How to Report

1. **Email**: security@campaign-express.io
2. **Subject**: `[SECURITY] <brief description>`
3. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Suggested fix (if applicable)

### Response Timeline

| Phase | SLA |
|-------|-----|
| Acknowledgment | Within 24 hours |
| Initial assessment | Within 48 hours |
| Fix development | Based on severity (see below) |
| Disclosure | Coordinated with reporter |

### Severity Levels

| Severity | Response Time | Examples |
|----------|--------------|---------|
| **Critical** | Fix within 24 hours | RCE, auth bypass, data exfiltration |
| **High** | Fix within 72 hours | Privilege escalation, SQL injection, XSS |
| **Medium** | Fix within 1 week | Information disclosure, CSRF |
| **Low** | Fix within 2 weeks | Minor info leak, configuration issue |

---

## Security Architecture

### Authentication & Authorization

- **Bearer Token Authentication**: All management API endpoints require `Authorization: Bearer <token>` headers
- **Role-Based Access Control (RBAC)**: Granular permissions per role (admin, manager, viewer, analyst)
- **Multi-Tenancy**: Strict tenant isolation at the data and API layer
- **API Key Management**: Per-tenant API keys with scoped permissions
- **Audit Logging**: All management operations are recorded with user, action, resource, and timestamp

### Network Security

Campaign Express enforces defense-in-depth at the Kubernetes network layer:

- **Default Deny**: All ingress and egress traffic is denied by default via NetworkPolicies
- **Explicit Allow Rules**: Only required communication paths are permitted:
  - Internal pod-to-pod (ports 8080, 9090, 9091)
  - NATS access (port 4222, 6222, 8222)
  - Redis access (port 6379)
  - ClickHouse access (port 8123, 9000)
  - Prometheus scraping (port 9091)
  - HAProxy ingress (port 8080)
  - DNS resolution (port 53)

### TLS / Certificate Management

- **cert-manager** with Let's Encrypt issuers (production and staging)
- All external traffic terminates TLS at the HAProxy ingress layer
- Internal cluster communication uses mTLS where supported

### Secrets Management

- **External Secrets Operator** syncs secrets from Azure Key Vault into Kubernetes
- Secrets managed externally: Redis password, ClickHouse credentials, NATS auth token, Twilio API key, SendGrid API key, Stripe API key, JWT signing key
- No secrets are stored in source code, environment files, or ConfigMaps
- Azure Key Vault access via Kubernetes workload identity

### Rate Limiting

- HAProxy enforces request rate limiting: **10,000 requests per 10 seconds** per source IP
- Exceeding the limit returns HTTP 429 (Too Many Requests)

### Data Protection

- **Encryption at Rest**: Azure managed disk encryption for all persistent volumes
- **Encryption in Transit**: TLS 1.2+ for all external connections
- **Data Retention**: ClickHouse analytics data retention configurable per deployment
- **Log Retention**: Loki retains logs for 7 days with automatic compaction
- **Trace Retention**: Tempo retains distributed traces for 72 hours

### Compliance

- **DSR Support**: Data Subject Request (DSR) API for GDPR/CCPA compliance
- **Privacy Controls**: Per-tenant privacy configuration
- **Audit Trail**: Immutable audit log for all data access and modifications

---

## Security Checklist for Contributors

When contributing to Campaign Express, ensure:

- [ ] No secrets, credentials, API keys, or tokens are committed to the repository
- [ ] No `.env` files are included in commits
- [ ] Input validation is applied at all API boundaries
- [ ] SQL/NoSQL injection is prevented (use parameterized queries)
- [ ] XSS protection is in place for any user-rendered content
- [ ] Authentication checks are present on all management endpoints
- [ ] RBAC permissions are enforced for sensitive operations
- [ ] Error messages do not leak internal implementation details
- [ ] Dependencies are reviewed for known vulnerabilities
- [ ] Network policies are updated if new services or ports are introduced

---

## Dependency Security

- Run `cargo audit` regularly to check for known vulnerabilities in Rust dependencies
- Review new dependency additions in pull requests
- Prefer well-maintained crates with active security disclosure practices
- Pin dependency versions in `Cargo.lock` (committed to repository)

---

## Incident Response

In the event of a security incident:

1. **Contain**: Isolate affected systems immediately
2. **Assess**: Determine scope and impact
3. **Notify**: Alert the security team and affected stakeholders
4. **Remediate**: Apply fixes and patches
5. **Review**: Conduct post-incident review and update procedures

For incident escalation, contact: security@campaign-express.io
