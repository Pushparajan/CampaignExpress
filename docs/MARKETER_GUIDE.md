# Campaign Express — Marketer User Guide

**Version:** 1.0  
**Last Updated:** 2026-02-13  
**Audience:** Campaign Managers, Brand Managers, Marketing Teams

---

## Table of Contents

1. [Getting Started](#1-getting-started)
2. [Dashboard Overview](#2-dashboard-overview)
3. [Campaign Management](#3-campaign-management)
4. [Creative Management](#4-creative-management)
5. [Journey Orchestration](#5-journey-orchestration)
6. [Audience Segmentation](#6-audience-segmentation)
7. [Loyalty Program](#7-loyalty-program)
8. [Multi-Channel Delivery](#8-multi-channel-delivery)
9. [Budget Tracking and Pacing](#9-budget-tracking-and-pacing)
10. [Reporting and Analytics](#10-reporting-and-analytics)
11. [Workflows and Approvals](#11-workflows-and-approvals)
12. [A/B Testing and Experimentation](#12-ab-testing-and-experimentation)
13. [Integrations](#13-integrations)
14. [Best Practices](#14-best-practices)
15. [FAQ](#15-faq)
16. [Glossary](#16-glossary)

---

## 1. Getting Started

### 1.1 Accessing Campaign Express

Campaign Express is accessible through:

- **Web Dashboard:** http://localhost:3000 (development) or your organization's URL
- **REST API:** http://localhost:8080/api/v1
- **Mobile SDK:** Integration via iOS/Android/React Native/Flutter

### 1.2 Login and Authentication

1. Navigate to the login page at `/login`
2. Enter your credentials provided by your administrator
3. Upon successful login, you'll be redirected to the main dashboard

**Authentication Token:** All API requests require a Bearer token:
```bash
Authorization: Bearer campaign-express-demo-token
```

### 1.3 User Roles

Campaign Express supports role-based access control (RBAC):

| Role | Permissions |
|------|-------------|
| **Campaign Manager** | Create and manage campaigns, view reports |
| **Brand Manager** | Manage brand guidelines, approve creatives |
| **Analyst** | View reports, export data, track budgets |
| **Director** | Approve high-budget campaigns, access all features |
| **Compliance** | Review regulated channels, manage suppression lists |

### 1.4 Quick Start Checklist

Before creating your first campaign, ensure:

- [ ] You have appropriate role permissions
- [ ] Brand guidelines are configured
- [ ] Asset library contains necessary creative assets
- [ ] Audience segments are defined
- [ ] Channel integrations are configured (email, SMS, push)
- [ ] Budget allocations are approved

---

## 2. Dashboard Overview

### 2.1 Main Dashboard

The main dashboard provides an at-a-glance view of your marketing operations:

**Key Metrics Cards:**
- Active Campaigns
- Total Impressions (last 7 days)
- Click-Through Rate (CTR)
- Conversion Rate (CVR)
- Budget Utilization

**Quick Actions:**
- Create New Campaign
- View Pending Approvals
- Generate Report
- Manage Journeys

### 2.2 Navigation

The sidebar provides access to all major features:

- **Campaigns** — Campaign management and creation
- **Journeys** — Journey orchestration and state machines
- **DCO** — Dynamic Creative Optimization and templates
- **CDP** — Customer Data Platform integrations
- **Experiments** — A/B/n testing setup and results
- **Billing** — Usage tracking and billing information
- **Platform** — User management and system settings
- **Ops** — Operations dashboard and SLA tracking

---

## 3. Campaign Management

### 3.1 Campaign Lifecycle

Campaigns in Campaign Express follow a 9-stage lifecycle:

1. **Draft** — Initial creation and configuration
2. **InReview** — Submitted for approval
3. **Approved** — Ready to schedule
4. **Scheduled** — Set to launch at specific time
5. **Live** — Currently running
6. **Paused** — Temporarily stopped
7. **Completed** — Finished execution
8. **Archived** — Historical record
9. **Rejected** — Did not pass approval (can be resubmitted)

### 3.2 Creating a Campaign

**Step 1: Navigate to Campaigns**
1. Click **Campaigns** in the sidebar
2. Click **Create Campaign** button

**Step 2: Basic Information**
```
Campaign Name: Spring Product Launch 2026
Campaign Type: Promotional Offer
Start Date: 2026-03-01 09:00
End Date: 2026-03-31 23:59
Budget: $50,000
```

**Step 3: Target Audience**
- Select from existing segments or create new segment
- Choose targeting criteria:
  - Demographics (age, location, gender)
  - Behavioral (purchase history, engagement)
  - Predictive (churn risk, lifetime value)
  - Lifecycle stage
  - Lookalike audiences

**Step 4: Offer Configuration**
```
Offer Type: Discount
Discount Value: 20%
Minimum Purchase: $50
Maximum Redemptions: 10,000
Expiration: 30 days from delivery
```

**Step 5: Creative Assignment**
- Select creative template from DCO library
- Assign brand-compliant assets
- Configure dynamic variants

**Step 6: Channel Selection**
Choose one or more channels:
- Email (SendGrid)
- Push Notifications
- SMS (Twilio)
- In-App Messages
- WhatsApp
- Web Push
- Webhooks

**Step 7: Delivery Schedule**
- **Immediate:** Send as soon as campaign goes live
- **Scheduled:** Specify exact date/time
- **Triggered:** Based on user behavior or segment entry
- **Recurring:** Daily, weekly, or monthly cadence

**Step 8: Review and Submit**
- Review all campaign settings
- Check brand guideline compliance
- Submit for approval workflow

### 3.3 Campaign Actions

Available actions depend on campaign stage:

| Stage | Available Actions |
|-------|-------------------|
| Draft | Submit, Delete, Duplicate |
| InReview | (Wait for approval) |
| Approved | Schedule, Go Live, Edit |
| Scheduled | Cancel Schedule, Edit Schedule |
| Live | Pause, Complete |
| Paused | Resume, Complete |
| Completed | Archive, View Report |

### 3.4 Campaign Calendar

View all campaigns on a calendar interface:

1. Navigate to **Campaigns** → **Calendar**
2. Filter by:
   - Campaign status
   - Channel type
   - Date range
   - Budget range
3. Click any campaign for quick actions

### 3.5 Campaign Templates

Save time with reusable campaign templates:

**Creating a Template:**
1. Create a campaign with desired settings
2. Click **Save as Template**
3. Name your template
4. Select which settings to include

**Using a Template:**
1. Click **Create from Template**
2. Select template
3. Customize specific values
4. Submit

---

## 4. Creative Management

### 4.1 Brand Guidelines

Campaign Express enforces brand compliance automatically.

**Brand Guideline Categories:**

**A. Color Palette**
```
Primary: #1B4FDB (Campaign Express Blue)
Secondary: #FF6B35 (Energetic Orange)
Accent: #00D4AA (Success Green)
Neutral: #F5F7FA (Background Gray)
Text: #1A1A1A (Primary Text)
Error: #DC2626 (Alert Red)
```

**B. Typography Rules**
```
Body Text: Inter, 14-20px
Headings: Inter, 18-72px
Code/Monospace: Roboto Mono, 12-16px
```

**C. Tone of Voice**
- **Approved Keywords:** innovative, effortless, powerful, scalable, intelligent
- **Forbidden Terms:** cheap, guaranteed (without context), revolutionary, magic

**D. Logo Usage**
- Minimum size: 32px height
- Clear space: 16px on all sides
- Approved backgrounds: white, light gray, primary blue

**Validation Process:**
All creatives are automatically validated against brand guidelines before launch. Non-compliant creatives will be flagged for review.

### 4.2 Asset Library

The asset library is your centralized repository for all marketing assets.

**Asset Types:**
- Images (JPG, PNG, WebP)
- Videos (MP4, MOV)
- Logos (SVG, PNG)
- Fonts (TTF, WOFF, WOFF2)
- Color Palettes (JSON)
- Templates (HTML, JSON)
- Documents (PDF, DOCX)
- Audio (MP3, WAV)
- Icons (SVG)
- Animations (GIF, Lottie)

**Uploading Assets:**
1. Navigate to **Brand** → **Asset Library**
2. Click **Upload Asset**
3. Select file and fill in metadata:
   ```
   Asset Name: spring-banner-hero
   Type: Image
   Tags: spring, seasonal, hero
   Folder: Campaigns/Spring2026
   ```
4. Click **Upload**

**Asset Organization:**
- Use folders to organize by campaign, season, or category
- Tag assets for easy discovery
- Maintain version history for all assets

**Asset Lifecycle:**
- **Active** — Available for use in campaigns
- **Archived** — Historical reference, not available for new campaigns
- **PendingReview** — Awaiting brand approval
- **Rejected** — Did not meet brand guidelines

**Searching Assets:**
```bash
Search by: name, type, tags, folder
Filter by: lifecycle status, date uploaded, file type
Sort by: name, date, popularity
```

### 4.3 Dynamic Creative Optimization (DCO)

DCO allows you to automatically test and optimize creative variants.

**Creating a DCO Template:**

1. Navigate to **DCO** → **Templates**
2. Click **Create Template**
3. Define component slots:
   ```
   Template: Email Hero Banner
   
   Slots:
   - Headline (text, max 50 chars)
   - Hero Image (image, 600x400px)
   - CTA Text (text, max 20 chars)
   - CTA Color (color from brand palette)
   ```

4. Add variants for each slot:
   ```
   Headline Variants:
   - "Spring Sale: 20% Off Everything"
   - "Limited Time: Big Savings Inside"
   - "Your Spring Style Awaits"
   
   Hero Image Variants:
   - spring-flowers.jpg
   - spring-fashion.jpg
   - spring-lifestyle.jpg
   
   CTA Text Variants:
   - "Shop Now"
   - "Get Started"
   - "Claim Offer"
   ```

5. Configure optimization:
   ```
   Strategy: Thompson Sampling
   Exploration Rate: 10%
   Success Metric: Click-Through Rate (50%) + Conversion Rate (30%) + Segment Affinity (20%)
   Max Combinations: 1,000
   ```

**How DCO Works:**

1. Campaign Express generates all possible combinations from your variants
2. Thompson Sampling algorithm selects variants to show each user
3. Performance is tracked: impressions, clicks, conversions
4. Algorithm learns which combinations perform best
5. Traffic gradually shifts to winning variants
6. 10% exploration rate ensures continued testing

**Viewing DCO Performance:**
1. Navigate to your campaign
2. Click **DCO Performance** tab
3. View metrics for each variant and combination
4. Identify top performers and underperformers

---

## 5. Journey Orchestration

### 5.1 What is Journey Orchestration?

Journey orchestration allows you to create personalized, multi-step customer experiences that adapt based on user behavior.

**Key Concepts:**
- **State Machine** — Defines possible states and transitions
- **Trigger** — Event that starts or advances the journey
- **Action** — What happens at each state (send message, wait, etc.)
- **Branch** — Conditional path based on user behavior
- **Delay** — Time-based waiting between states

### 5.2 Creating a Journey

**Step 1: Define Journey**
1. Navigate to **Journeys** → **Create Journey**
2. Name your journey: "Onboarding Flow Q1 2026"
3. Set journey type: Onboarding, Retention, Win-back, or Custom

**Step 2: Configure Trigger**

Choose how users enter the journey:

**Event-Based Triggers:**
```
Trigger: User signs up
Conditions: 
- Account created in last 24 hours
- Email verified
- No prior purchase
```

**Segment-Based Triggers:**
```
Trigger: User enters segment
Segment: New Users - High Intent
Re-entry: Not allowed
```

**Schedule-Based Triggers:**
```
Trigger: Every Monday at 9:00 AM
Target Segment: Active Users
Time Zone: User's local time zone
```

**Step 3: Build Journey Flow**

**Example: Welcome Journey**
```
State 1: Welcome Email
├─ Action: Send email "welcome-series-01"
├─ Delay: 24 hours
└─ Next: Check Engagement

State 2: Check Engagement
├─ Branch: If opened email
│   └─ Next: Feature Tour
└─ Branch: If not opened
    └─ Next: Re-engagement Email

State 3a: Feature Tour (Engaged Path)
├─ Action: Send email "feature-tour"
├─ Delay: 48 hours
└─ Next: Check First Action

State 3b: Re-engagement Email (Not Engaged Path)
├─ Action: Send email "welcome-reminder"
├─ Delay: 48 hours
└─ Next: Check Engagement

State 4: Check First Action
├─ Branch: If completed action
│   └─ Next: Success Celebration
└─ Branch: If not completed
    └─ Next: Helpful Tips

State 5a: Success Celebration
├─ Action: Send email "congrats-first-action"
└─ End Journey

State 5b: Helpful Tips
├─ Action: Send email "getting-started-tips"
├─ Delay: 24 hours
└─ Next: Check First Action (retry)
```

**Step 4: Configure Journey Settings**

```
Journey Settings:
- Max journey duration: 30 days
- Re-entry allowed: No
- Quiet hours: 10 PM - 8 AM local time
- Frequency cap: Max 1 message per day
- Exit conditions: User makes purchase, unsubscribes
```

**Step 5: Test and Launch**

1. Click **Test Journey**
2. Enter test user ID
3. Review journey path for test user
4. When satisfied, click **Submit for Approval**

### 5.3 Journey Analytics

Track journey performance:

**Key Metrics:**
- Total users entered
- Completion rate
- Average time to completion
- Drop-off by state
- Conversion rate
- Revenue attributed

**Optimization Tips:**
- Identify states with high drop-off
- Test different message variants
- Adjust delays based on engagement patterns
- Add alternative paths for different user behaviors

---

## 6. Audience Segmentation

### 6.1 Segment Types

Campaign Express supports five segment types:

**1. Behavioral Segments**
```
Example: High-Value Customers
Rules:
- Total purchases > $1,000
- Purchase frequency > 3 times/year
- Last purchase within 90 days
```

**2. Demographic Segments**
```
Example: Bay Area Professionals
Rules:
- Age: 25-45
- Location: San Francisco Bay Area
- Income: $100k+
```

**3. Predictive Segments**
```
Example: Churn Risk
Rules:
- ML churn score > 0.7
- Days since last visit > 30
- Decreasing engagement trend
```

**4. Lifecycle Segments**
```
Segments:
- New Users (0-30 days)
- Active Users (30-90 days, engaged)
- At-Risk Users (90-180 days, declining)
- Dormant Users (180+ days, inactive)
- Churned Users (360+ days, no activity)
```

**5. Lookalike Segments**
```
Example: Similar to Top Customers
Seed Segment: Top 10% customers by LTV
Similarity Threshold: 80%
Max Size: 50,000 users
```

### 6.2 Creating a Segment

**Method 1: Rule Builder (UI)**

1. Navigate to **Audiences** → **Segments**
2. Click **Create Segment**
3. Name segment: "Spring Campaign Target"
4. Add rules using visual builder:
   ```
   Rule Group 1 (AND):
   ├─ Age: between 25 and 55
   ├─ Location: in [US, CA, UK]
   └─ Email Subscribed: equals true
   
   Rule Group 2 (OR):
   ├─ Purchase Category: contains "Apparel"
   └─ Viewed Category: contains "Fashion"
   ```
5. Preview segment size
6. Save segment

**Method 2: Advanced (Expression)**

For complex segments, use expression syntax:
```rust
(age >= 25 AND age <= 55) AND
(country IN ["US", "CA", "UK"]) AND
(email_subscribed = true) AND
(
  (purchase_categories CONTAINS "Apparel") OR
  (viewed_categories CONTAINS "Fashion")
)
```

### 6.3 Segment Management

**Refreshing Segments:**
- Real-time segments update continuously
- Batch segments refresh every 1-24 hours (configurable)
- Manual refresh available for batch segments

**Segment Analytics:**
- View segment size over time
- Track segment growth/decline
- Compare segment performance
- Export segment for analysis

**CDP Sync:**
Segments can be synced to your CDP platforms:
1. Select segment
2. Click **Sync to CDP**
3. Choose destination: Salesforce, Segment, Tealium, etc.
4. Configure field mappings
5. Set sync schedule (hourly, daily, real-time)

---

## 7. Loyalty Program

### 7.1 Loyalty Tiers

Campaign Express includes a 3-tier loyalty program:

| Tier | Qualification | Benefits |
|------|---------------|----------|
| **Green** | 0-999 stars | 5% boost on offers, standard support |
| **Gold** | 1,000-4,999 stars | 10% boost on offers, priority support, birthday rewards |
| **Reserve** | 5,000+ stars | 15% boost on offers, concierge support, exclusive offers, early access |

### 7.2 Earning Stars

**Earning Rules:**
```
Purchase: 1 star per $1 spent
Referral: 100 stars per successful referral
Review: 50 stars per product review
Social Share: 25 stars per share
Birthday Bonus: 200 stars on birthday
```

**Bonus Multipliers:**
- First purchase: 2x stars
- Holiday periods: 1.5x stars
- Special promotions: up to 5x stars

### 7.3 Redeeming Stars

**Redemption Options:**
```
Rewards:
- $5 discount: 500 stars
- $10 discount: 900 stars (10% bonus)
- $25 discount: 2,000 stars (20% bonus)
- Free shipping: 300 stars
- Gift with purchase: 1,500 stars
```

**Redemption Rules:**
- Minimum order value: $20
- Cannot combine multiple rewards
- Rewards expire after 12 months of inactivity

### 7.4 Tier Progression

**Upgrade Path:**
```
Green → Gold: Earn 1,000 stars
Gold → Reserve: Earn 5,000 stars
```

**Tier Benefits Activation:**
- Immediate upon reaching threshold
- Notification sent via email and push
- Celebration message in next app open

**Tier Maintenance:**
- Review period: Annual
- Grace period: 90 days to re-qualify
- Downgrade if stars drop below threshold

### 7.5 Loyalty Campaign Integration

**Targeting by Tier:**
When creating campaigns, filter by loyalty tier:
```
Audience: Gold + Reserve tiers only
Offer: Exclusive 48-hour early access
Message: "Thank you for being a valued member"
```

**Tier-Specific Offers:**
```
Green Tier: 10% off
Gold Tier: 15% off + free shipping
Reserve Tier: 20% off + free shipping + gift
```

**Loyalty Boost in Bidding:**
When serving offers in real-time bidding:
- Green tier: 1.05x bid multiplier
- Gold tier: 1.10x bid multiplier
- Reserve tier: 1.15x bid multiplier

---

## 8. Multi-Channel Delivery

### 8.1 Supported Channels

Campaign Express delivers messages across 7 channels:

| Channel | Provider | Use Case |
|---------|----------|----------|
| Email | SendGrid | Newsletters, promotions, transactional |
| Push Notifications | Native | Mobile app engagement, time-sensitive offers |
| SMS | Twilio | Urgent alerts, two-factor auth, order updates |
| In-App Messages | Native | Contextual guidance, feature announcements |
| WhatsApp | Twilio | Conversational marketing, customer support |
| Web Push | Native | Browser notifications, abandoned cart |
| Webhooks | Custom | Integration with external systems |

### 8.2 Channel Configuration

**Email (SendGrid)**
```
Configuration:
- API Key: (from SendGrid dashboard)
- From Email: marketing@yourcompany.com
- From Name: Your Company Team
- Reply-To: support@yourcompany.com
- Tracking: Opens, clicks enabled
- Unsubscribe: Auto-handle unsubscribe requests
```

**SMS (Twilio)**
```
Configuration:
- Account SID: (from Twilio)
- Auth Token: (from Twilio)
- Phone Number: +1-555-CAMPAIGN
- Message Length: Max 160 chars (or MMS for longer)
- Opt-Out: AUTO handle STOP keyword
```

**Push Notifications**
```
iOS Configuration:
- APNs Certificate: upload .p12 file
- Bundle ID: com.yourcompany.app
- Environment: Production

Android Configuration:
- FCM Server Key: (from Firebase Console)
- Package Name: com.yourcompany.app
```

### 8.3 Message Templates

**Creating Email Templates:**

1. Navigate to **Channels** → **Email Templates**
2. Click **Create Template**
3. Choose editor type:
   - Visual Editor (drag-and-drop)
   - HTML Editor (code)
   - Hybrid (both)

**Template Variables:**
```html
<!DOCTYPE html>
<html>
<head>
  <title>{{campaign_name}}</title>
</head>
<body>
  <h1>Hello {{user.first_name}}!</h1>
  
  <p>Special offer just for you:</p>
  <h2>{{offer.discount}}% off {{offer.product_category}}</h2>
  
  <a href="{{offer.redeem_url}}">
    Claim Your Offer
  </a>
  
  <p>This offer expires on {{offer.expiry_date}}.</p>
  
  <p>You're a {{loyalty.tier}} member with {{loyalty.stars}} stars!</p>
</body>
</html>
```

**Available Variables:**
- `{{user.*}}` — User profile fields
- `{{offer.*}}` — Offer details
- `{{campaign.*}}` — Campaign information
- `{{loyalty.*}}` — Loyalty program data
- `{{product.*}}` — Product recommendations

### 8.4 Channel Rules and Compliance

**Frequency Capping:**
```
Global Rules:
- Max 3 emails per day
- Max 1 SMS per day
- Max 2 push notifications per day
- Max 5 total messages per day (all channels)

Per-Channel Rules:
- Email: Max 10 per week
- SMS: Max 3 per week
- Push: Max 14 per week
```

**Quiet Hours:**
```
Default: 10 PM - 8 AM local time
Configurable per channel:
- Email: Any time (less intrusive)
- SMS: 9 AM - 8 PM only
- Push: 8 AM - 10 PM only
```

**Suppression Lists:**

Campaign Express maintains global suppression lists:

1. **Unsubscribed Users** — Auto-managed per channel
2. **Hard Bounces** — Email addresses that permanently failed
3. **Spam Complaints** — Users who marked as spam
4. **Manual Suppression** — Admin-added exclusions

**Checking Suppression:**
```bash
GET /api/v1/channels/suppression/check
{
  "channel": "email",
  "identifier": "user@example.com"
}
```

---

## 9. Budget Tracking and Pacing

### 9.1 Budget Allocation

**Setting Campaign Budget:**
```
Budget Configuration:
- Total Budget: $50,000
- Currency: USD
- Budget Type: Total (not daily)
- Pacing Strategy: Even
- Start Date: 2026-03-01
- End Date: 2026-03-31
```

**Budget Distribution:**
```
Daily Budget = Total Budget ÷ Campaign Days
$50,000 ÷ 31 days = $1,612.90 per day
```

**Pacing Strategies:**
- **Even** — Spend budget evenly across campaign duration
- **ASAP** — Spend as quickly as possible (no pacing)
- **Accelerated** — Front-load spending in first 50% of duration
- **Custom** — Define custom spending curve

### 9.2 Spend Tracking

**Real-Time Tracking:**
Campaign Express tracks spend in real-time:

```
Campaign: Spring Launch
Total Budget: $50,000
Spent to Date: $32,450
Remaining: $17,550
Pace: On track (Day 20 of 31)
Projected Final: $50,313 (within 1% of budget)
```

**Budget Metrics:**
- **Burn Rate** — Average daily spend
- **Pace** — Ahead, on track, or behind schedule
- **Utilization** — Percentage of budget used
- **Projection** — Expected final spend

### 9.3 Pacing Alerts

Automatic alerts when thresholds are crossed:

**Alert Levels:**
```
Warning (80%): Budget is 80% depleted
├─ Notify: Campaign Manager
└─ Action: Review pacing, consider extension

Critical (100%): Budget fully depleted
├─ Notify: Campaign Manager, Director
└─ Action: Campaign auto-pauses

Daily Overspend (20%): Spent >120% of daily budget
├─ Notify: Campaign Manager
└─ Action: Review targeting, bid adjustments
```

**Alert Delivery:**
- Email notification
- In-app notification
- Slack/Teams integration (if configured)
- SMS for critical alerts (optional)

### 9.4 ROI and ROAS Tracking

**Return on Ad Spend (ROAS):**
```
ROAS = Revenue Generated ÷ Ad Spend
Example: $125,000 ÷ $50,000 = 2.5x ROAS
```

**Return on Investment (ROI):**
```
ROI = (Revenue - Cost) ÷ Cost × 100%
Example: ($125,000 - $50,000) ÷ $50,000 × 100% = 150% ROI
```

**Viewing Financial Metrics:**
1. Navigate to campaign details
2. Click **Financial Performance** tab
3. View:
   - Total spend
   - Revenue attributed
   - ROAS
   - ROI
   - Cost per acquisition (CPA)
   - Customer lifetime value (LTV)

---

## 10. Reporting and Analytics

### 10.1 Report Types

Campaign Express offers 10 pre-built report types:

1. **Campaign Performance** — Impressions, clicks, conversions by campaign
2. **Channel Performance** — Compare performance across email, SMS, push, etc.
3. **Cohort Analysis** — Track user behavior over time by acquisition cohort
4. **Funnel Report** — Conversion funnel from impression to purchase
5. **Attribution Report** — Multi-touch attribution across channels
6. **Budget Report** — Spend, pacing, ROI by campaign
7. **Audience Report** — Segment performance and overlap
8. **DCO Performance** — Creative variant performance
9. **Journey Report** — Journey completion, drop-offs, time-to-convert
10. **Executive Dashboard** — High-level KPIs for leadership

### 10.2 Creating a Report

**Using Report Templates:**

1. Navigate to **Reports** → **Create Report**
2. Select template: "Campaign Performance"
3. Configure parameters:
   ```
   Date Range: Last 30 days
   Campaigns: All active campaigns
   Group By: Campaign, Channel
   Metrics:
   - Impressions
   - Clicks
   - CTR
   - Conversions
   - CVR
   - Revenue
   - ROAS
   ```
4. Click **Generate Report**

**Custom Report Builder:**

1. Click **Custom Report**
2. Select data source: Campaigns, Users, Events, Revenue
3. Choose dimensions (group by):
   - Campaign ID/Name
   - Channel
   - Segment
   - Day/Week/Month
   - Device Type
   - Location
4. Choose metrics:
   - Count, Sum, Average, Min, Max, Percentiles
   - Calculated metrics (CTR, CVR, ROAS)
5. Add filters:
   ```
   Filters:
   - Campaign Status = Live OR Completed
   - Spend > $1,000
   - Channel IN [email, push, sms]
   ```
6. Save and run

### 10.3 Scheduled Reports

**Automating Report Delivery:**

1. Create or open existing report
2. Click **Schedule**
3. Configure schedule:
   ```
   Frequency: Weekly
   Day: Monday
   Time: 9:00 AM
   Time Zone: Pacific Time
   Recipients:
   - marketing-team@company.com
   - director@company.com
   Format: PDF + Excel
   ```
4. Save schedule

**Export Formats:**
- **CSV** — Raw data for analysis
- **Excel** — Formatted spreadsheet with charts
- **PDF** — Presentation-ready report
- **JSON** — API integration

### 10.4 Dashboard Widgets

**Adding Widgets:**

1. Navigate to main dashboard
2. Click **Customize Dashboard**
3. Select widgets to add:
   - Campaign Performance (bar chart)
   - Budget Utilization (gauge)
   - Top Performing Segments (table)
   - Channel Comparison (pie chart)
   - Recent Alerts (list)
4. Drag to arrange
5. Save layout

**Sharing Dashboards:**
- Save dashboard as template
- Share view-only link
- Export as PDF

---

## 11. Workflows and Approvals

### 11.1 Approval Workflow

All campaigns requiring approval follow this process:

**Workflow Stages:**
```
Draft
  ↓ [Submit Action]
InReview
  ↓ [Approve Action] or [Reject Action]
Approved (or Rejected)
  ↓ [Schedule Action]
Scheduled
  ↓ [GoLive Action]
Live
  ↓ [Pause or Complete Action]
Paused or Completed
  ↓ [Archive Action]
Archived
```

### 11.2 Approval Rules

Campaigns are routed to approvers based on rules:

**Standard Campaign Rule:**
```
Rule: Standard Campaign
Applies When:
- Budget < $1,000
- Channel = email OR push
Required Approvals:
- 1 approval from "manager" role
Auto-Approve: Yes (if budget < $1,000)
```

**High Budget Campaign Rule:**
```
Rule: High Budget Campaign
Applies When:
- Budget ≥ $1,000
Required Approvals:
- 2 approvals from "director" role
Auto-Approve: No
```

**Regulated Channel Rule:**
```
Rule: Regulated Channel
Applies When:
- Channel = SMS OR WhatsApp
Required Approvals:
- 2 approvals from "compliance" role
Requires:
- Legal review flag
- Creative review flag
Auto-Approve: No
```

### 11.3 Submitting for Approval

**Step 1: Review Campaign**
Before submitting, verify:
- All required fields completed
- Budget approved by finance
- Creative assets uploaded
- Brand guidelines met
- Target audience defined
- Channel configuration correct

**Step 2: Submit**
1. Click **Submit for Review**
2. Add comments for reviewers:
   ```
   Comments:
   "Spring campaign targeting new users in CA, NY, TX.
   Budget approved by Jane Smith (CFO).
   Creative reviewed by Brand team.
   Launch date: March 1, 2026."
   ```
3. Click **Submit**

**Step 3: Track Approval**
- Campaign status changes to "InReview"
- Approvers notified via email
- View pending approvals in **Workflows** dashboard

### 11.4 Reviewing Campaigns

**For Approvers:**

1. Navigate to **Workflows** → **Pending Approvals**
2. Click campaign to review
3. Review details:
   - Campaign configuration
   - Budget and ROI projections
   - Creative preview
   - Target audience
   - Brand compliance check
4. Take action:

**Approve:**
```
Action: Approve
Comments: "Approved. Looks great!"
Effect: Campaign moves to Approved stage (if threshold met)
```

**Reject:**
```
Action: Reject
Reason: Budget exceeds Q1 allocation
Comments: "Please reduce budget to $40k and resubmit"
Effect: Campaign moves to Rejected stage
```

**Request Changes:**
```
Action: RequestChanges
Comments: "Please update CTA to 'Learn More' per brand guidelines"
Effect: Campaign returns to Draft for edits
```

### 11.5 Approval History

View complete audit trail:
1. Open campaign
2. Click **Approval History** tab
3. View timeline:
   ```
   2026-02-15 10:30 AM — Submitted by Alice (Campaign Manager)
   2026-02-15 02:15 PM — Approved by Bob (Marketing Director)
   2026-02-15 02:45 PM — Approved by Carol (Brand Manager)
   2026-02-15 03:00 PM — Auto-transitioned to Approved
   2026-02-16 09:00 AM — Scheduled by Alice for March 1
   ```

---

## 12. A/B Testing and Experimentation

### 12.1 Experiment Types

**A/B Test:**
Compare two versions (A vs. B)
```
Variant A: Control (existing creative)
Variant B: Treatment (new creative)
Traffic Split: 50% / 50%
```

**A/B/n Test:**
Compare multiple versions (A vs. B vs. C vs. D...)
```
Variant A: Control (current offer)
Variant B: 15% discount
Variant C: 20% discount
Variant D: Free shipping
Traffic Split: 25% / 25% / 25% / 25%
```

**Multivariate Test:**
Test multiple elements simultaneously
```
Elements:
- Subject Line: 3 variants
- Hero Image: 2 variants
- CTA Text: 2 variants
Combinations: 3 × 2 × 2 = 12 total variants
```

### 12.2 Creating an Experiment

**Step 1: Define Experiment**
1. Navigate to **Experiments** → **Create Experiment**
2. Name: "Spring Campaign - Discount Test"
3. Type: A/B Test
4. Primary Metric: Click-Through Rate
5. Secondary Metrics: Conversion Rate, Revenue

**Step 2: Configure Variants**
```
Control (A):
- Name: "Standard 10% Off"
- Description: Current offer
- Traffic: 50%

Treatment (B):
- Name: "Enhanced 20% Off"
- Description: Higher discount test
- Traffic: 50%
```

**Step 3: Set Sample Size and Duration**
```
Confidence Level: 95%
Minimum Detectable Effect: 10%
Expected Baseline CTR: 5%
Required Sample Size: 15,422 per variant
Estimated Duration: 7 days
```

**Step 4: Assignment Strategy**
```
Assignment Method: User ID hash (deterministic)
Sticky: Yes (users stay in same variant)
Traffic Allocation: Equal
```

**Step 5: Launch**
- Review experiment setup
- Click **Start Experiment**
- Monitor in real-time

### 12.3 Analyzing Results

**Statistical Significance:**
Campaign Express automatically calculates:
- Confidence intervals
- P-values
- Statistical significance (α = 0.05)

**Results Dashboard:**
```
Experiment: Spring Campaign - Discount Test
Status: Completed
Duration: 7 days
Total Users: 31,440

Results:
┌──────────┬──────────┬───────┬─────┬────────────┬──────────┐
│ Variant  │ Users    │ CTR   │ CVR │ Revenue    │ Winner?  │
├──────────┼──────────┼───────┼─────┼────────────┼──────────┤
│ Control  │ 15,720   │ 4.8%  │ 2.1%│ $15,430    │          │
│ Treatment│ 15,720   │ 6.2%  │ 2.8%│ $21,840    │ ✓ Yes    │
└──────────┴──────────┴───────┴─────┴────────────┴──────────┘

Statistical Significance: Yes (p < 0.001)
Confidence: 99.9%
Lift: +29% CTR, +33% CVR, +42% Revenue
Recommendation: Roll out Treatment to 100% traffic
```

### 12.4 Implementing Winners

**After Experiment Concludes:**

1. Review results
2. Click **Declare Winner**
3. Choose option:
   - **Roll Out Winner** — Apply to 100% traffic
   - **Continue Testing** — Extend experiment
   - **Archive** — End without rolling out

**Rollout Process:**
1. Select winning variant
2. Click **Roll Out**
3. Campaign Express:
   - Updates campaign configuration
   - Gradually shifts traffic (0% → 100% over 24 hours)
   - Monitors for anomalies
   - Completes rollout

---

## 13. Integrations

### 13.1 DSP Integrations

Connect to demand-side platforms for programmatic advertising:

**Supported DSPs:**
- The Trade Desk (TTD)
- Google DV360
- Xandr (Microsoft Advertising)
- Amazon DSP

**Configuration:**
1. Navigate to **Integrations** → **DSP**
2. Click **Connect DSP**
3. Select provider: "The Trade Desk"
4. Enter credentials:
   ```
   Partner ID: ttd-partner-12345
   API Key: ••••••••••••••••
   Seat ID: seat-67890
   ```
5. Configure sync settings:
   ```
   Sync Frequency: Every 15 minutes
   Sync Segments: Yes
   Sync Conversions: Yes
   ```
6. Test connection
7. Save

**Using DSP Integration:**
When creating campaigns, select DSP distribution:
```
Distribution:
☑ Internal Campaign Express bidding
☑ The Trade Desk
☐ Google DV360
☐ Amazon DSP
```

### 13.2 CDP Integrations

Sync customer data with your CDP platforms:

**Supported CDPs:**
- Salesforce Data Cloud
- Adobe Experience Platform
- Segment
- Tealium
- mParticle
- Zeotap
- Hightouch

**Setting Up CDP Sync:**

1. Navigate to **CDP** → **Platforms**
2. Click **Add Platform**
3. Select: "Salesforce Data Cloud"
4. Configure connection:
   ```
   Instance URL: https://yourorg.salesforce.com
   Client ID: ••••••••
   Client Secret: ••••••••
   ```
5. Map fields:
   ```
   Campaign Express → Salesforce
   user_id → Contact.ExternalId__c
   email → Contact.Email
   first_name → Contact.FirstName
   last_name → Contact.LastName
   loyalty_tier → Contact.LoyaltyTier__c
   ```
6. Choose sync direction:
   - **Inbound** — Import from CDP to Campaign Express
   - **Outbound** — Export from Campaign Express to CDP
   - **Bidirectional** — Two-way sync
7. Set schedule: Real-time, Hourly, Daily
8. Save and test

**Syncing Segments:**
1. Select segment
2. Click **Sync to CDP**
3. Choose destination platform
4. Monitor sync status

### 13.3 DAM Integrations

Connect to Digital Asset Management systems:

**Supported DAMs:**
- Adobe AEM Assets
- Bynder
- Aprimo

**Configuration:**
1. Navigate to **Integrations** → **DAM**
2. Select provider: "Bynder"
3. Enter credentials
4. Enable asset search

**Using DAM Assets:**
When selecting creative assets:
1. Click **Browse DAM**
2. Search Bynder library
3. Select asset
4. Asset automatically imported to Campaign Express

### 13.4 Project Management Integrations

Sync campaigns with project management tools:

**Asana Integration:**
```
Feature: Create Asana task when campaign submitted for approval
Configuration:
- Workspace: Marketing Team
- Project: Q1 2026 Campaigns
- Assignee: Marketing Director
- Due Date: 2 days from submission
```

**Jira Integration:**
```
Feature: Create Jira ticket for campaign approval
Configuration:
- Project: MKTG
- Issue Type: Campaign Review
- Priority: Medium
- Assignee: Auto-assign to team
```

### 13.5 BI Tool Integrations

Export data to business intelligence platforms:

**Power BI:**
1. Navigate to **Integrations** → **BI Tools**
2. Click **Connect Power BI**
3. Generate API key for Power BI access
4. In Power BI:
   - Add Campaign Express data source
   - Enter API endpoint and key
   - Import campaign data
   - Build dashboards

**Excel Export:**
- Any report can be exported to Excel
- Schedule automated Excel deliveries
- Direct export from UI

---

## 14. Best Practices

### 14.1 Campaign Planning

**Before Launch:**
- [ ] Define clear objectives and KPIs
- [ ] Identify target audience and create segments
- [ ] Estimate budget based on expected reach and CPA
- [ ] Create and test creative assets
- [ ] Set up tracking and attribution
- [ ] Configure frequency caps and quiet hours
- [ ] Plan for A/B testing
- [ ] Schedule approval workflow
- [ ] Prepare for post-campaign analysis

**Campaign Naming Convention:**
```
Format: [Year]-[Quarter]-[Type]-[Audience]-[Version]
Example: 2026-Q1-PROMO-NewUsers-v1
```

### 14.2 Audience Segmentation Tips

**Start Broad, Then Refine:**
1. Begin with large segments
2. Analyze performance
3. Create refined sub-segments
4. Test refined segments

**Segment Overlap:**
- Monitor segment overlap
- Avoid conflicting messages to same users
- Use exclusion rules when necessary

**Refresh Frequency:**
- High-value segments: Real-time
- Campaign targets: Daily
- Reporting segments: Weekly

### 14.3 Creative Best Practices

**Email:**
- Subject line: 30-50 characters
- Preview text: Complement subject, don't repeat
- CTA: Above the fold, clear action verb
- Mobile optimization: 600px max width
- Alt text for all images
- Test across email clients

**Push Notifications:**
- Title: 65 characters max
- Body: 240 characters max
- Clear value proposition
- Time-sensitive content performs best
- Rich media increases engagement

**SMS:**
- 160 characters or less
- Include clear CTA with link
- Opt-out instructions required
- Urgent/time-sensitive only

### 14.4 Testing Strategy

**What to Test:**
- Subject lines and preview text
- Offer values (discount %, free shipping)
- CTA text and design
- Creative imagery and layout
- Sending time
- Audience segments

**Testing Cadence:**
- Always have 2-3 active experiments
- Test one element at a time (unless MVT)
- Run tests for full week (accounts for day-of-week effects)
- Archive and document all test results

**Sample Size:**
- Minimum 1,000 users per variant
- Run until statistical significance
- Don't stop early (avoid false positives)

### 14.5 Budget Management

**Pacing:**
- Review daily for first 3 days
- Adjust bids if spending too fast/slow
- Use even pacing for most campaigns
- Use ASAP only for time-sensitive offers

**Budget Allocation:**
```
Recommended Split:
- 60% to proven channels/audiences
- 30% to optimization tests
- 10% to experimental channels/audiences
```

**Contingency:**
- Build 10-15% buffer into budgets
- Monitor for unexpected overspend
- Have approval for budget increases ready

### 14.6 Performance Optimization

**Campaign Health Checks:**
- Review every 3 days for first 2 weeks
- Check: CTR, CVR, ROAS, frequency, pacing
- Compare to benchmarks
- Adjust if underperforming

**Benchmarks:**
```
Email:
- Open Rate: 15-25%
- CTR: 2-5%
- CVR: 1-3%

Push Notifications:
- Open Rate: 10-20%
- CTR: 1-3%

SMS:
- Open Rate: 95%+
- CTR: 15-30%
```

**When to Pause:**
- ROAS < 1.0 after 7 days
- CTR < 50% of benchmark
- High unsubscribe rate (>1%)
- Budget exhausted
- Negative feedback/complaints

---

## 15. FAQ

### Q1: How long does it take to create a campaign?

**A:** Standard campaigns can be created in 10-15 minutes. Complex campaigns with custom segments, DCO, and journey orchestration may take 1-2 hours.

### Q2: Can I duplicate an existing campaign?

**A:** Yes. Open the campaign and click **Duplicate**. All settings are copied; you can modify as needed before launching.

### Q3: What happens if my campaign exceeds budget?

**A:** Campaign Express automatically pauses the campaign when budget is reached. You'll receive alerts at 80% and 100% utilization.

### Q4: Can I edit a live campaign?

**A:** Minor edits (name, description) can be made anytime. Major changes (audience, budget, creative) require pausing the campaign first.

### Q5: How do I handle unsubscribes?

**A:** Unsubscribes are handled automatically. Users are added to suppression lists and won't receive future messages on that channel.

### Q6: Can I target users across multiple devices?

**A:** Yes. Campaign Express uses cross-device identity resolution via your CDP integration to target users consistently.

### Q7: What's the difference between segments and campaigns?

**A:** Segments define WHO to target (audiences). Campaigns define WHAT to send and WHEN to send it.

### Q8: How do I integrate with my existing tech stack?

**A:** Use the Integrations section to connect CDP, DSP, DAM, and BI tools. API documentation available for custom integrations.

### Q9: Can I schedule campaigns in different time zones?

**A:** Yes. Campaigns can be delivered in users' local time zones. Configure in delivery settings.

### Q10: What if I need help?

**A:** Contact support:
- Email: support@campaign-express.com
- Chat: Available 9 AM - 5 PM PT
- Documentation: https://docs.campaign-express.com
- Community: https://community.campaign-express.com

---

## 16. Glossary

**A/B Test:** Experiment comparing two versions to determine which performs better.

**Approval Workflow:** Multi-step review process requiring designated approvers before campaign launch.

**Asset Library:** Centralized repository for creative assets (images, videos, templates, etc.).

**Attribution:** Process of crediting conversions to the marketing touchpoints that influenced them.

**Brand Guidelines:** Rules governing use of colors, fonts, logos, and tone to maintain brand consistency.

**Campaign:** Coordinated marketing effort to deliver offers across one or more channels to a target audience.

**CDP (Customer Data Platform):** System that unifies customer data from multiple sources into a single view.

**Churn:** When a user stops engaging or using your product/service.

**Click-Through Rate (CTR):** Percentage of users who click on a call-to-action (clicks ÷ impressions).

**Conversion Rate (CVR):** Percentage of users who complete a desired action (conversions ÷ clicks).

**CPA (Cost Per Acquisition):** Average cost to acquire one customer (spend ÷ conversions).

**Creative:** Marketing asset such as image, video, or email template used in campaigns.

**DAM (Digital Asset Management):** System for storing, organizing, and retrieving digital assets.

**DCO (Dynamic Creative Optimization):** Automated testing and optimization of creative variants.

**DSP (Demand-Side Platform):** Platform for buying digital ad inventory programmatically.

**Frequency Cap:** Limit on how many times a user sees a message within a time period.

**Impression:** Single instance of an ad being displayed to a user.

**Journey:** Multi-step customer experience with conditional branching based on user behavior.

**Lifecycle Stage:** User's position in customer lifecycle (new, active, at-risk, churned).

**Lookalike Audience:** Segment of users similar to a seed audience based on shared characteristics.

**Loyalty Tier:** Level in loyalty program (Green, Gold, Reserve) based on earned stars.

**Pacing:** Rate of budget spend over campaign duration.

**Personalization:** Customizing messages based on individual user attributes and behavior.

**Quiet Hours:** Time period during which messages won't be sent (typically evening/night).

**ROAS (Return on Ad Spend):** Revenue generated per dollar spent (revenue ÷ spend).

**ROI (Return on Investment):** Profitability of campaign ((revenue - cost) ÷ cost × 100%).

**Segment:** Group of users defined by shared characteristics or behaviors.

**SLA (Service Level Agreement):** Commitment to specific performance and uptime standards.

**Suppression List:** List of users who should not receive messages (unsubscribed, bounced, etc.).

**Thompson Sampling:** Bayesian algorithm for balancing exploration and exploitation in A/B tests.

**Variant:** Different version of creative or offer in an experiment.

**Webhook:** HTTP callback that sends real-time data to external systems.

---

## Next Steps

Now that you understand how to use Campaign Express, here's what to do next:

1. **Create Your First Campaign**
   - Start with a simple email campaign
   - Target a small segment
   - Set a conservative budget
   - Monitor closely for first 24 hours

2. **Set Up Your Brand Guidelines**
   - Upload brand assets
   - Define color palette
   - Configure typography rules
   - Train team on compliance

3. **Connect Integrations**
   - CDP (sync customer data)
   - DSP (expand reach)
   - DAM (access creative assets)
   - BI tools (advanced analytics)

4. **Build Core Segments**
   - High-value customers
   - At-risk users
   - New user onboarding
   - Win-back dormant users

5. **Plan Your Journey**
   - Map customer lifecycle
   - Design welcome journey
   - Create retention journeys
   - Build win-back flows

6. **Establish Testing Culture**
   - Run continuous A/B tests
   - Document learnings
   - Share insights with team
   - Iterate based on data

**Need Help?** Your Customer Success Manager is here to support you. Reach out for:
- Onboarding assistance
- Strategic planning
- Technical troubleshooting
- Best practices guidance

**Ready to launch?** Let's create amazing customer experiences together! 🚀
