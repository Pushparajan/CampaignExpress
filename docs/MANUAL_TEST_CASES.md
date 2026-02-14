# Manual Test Cases — CampaignExpress

**Version:** 1.0  
**Date:** 2026-02-14  
**Project:** CampaignExpress - Real-Time Ad Offer Personalization Platform

---

## Table of Contents

1. [Authentication and Authorization](#1-authentication-and-authorization)
2. [Campaign Management](#2-campaign-management)
3. [Creative Management](#3-creative-management)
4. [Journey Orchestration](#4-journey-orchestration)
5. [Audience Segmentation](#5-audience-segmentation)
6. [Loyalty Program](#6-loyalty-program)
7. [Multi-Channel Delivery](#7-multi-channel-delivery)
8. [Budget Tracking and Reporting](#8-budget-tracking-and-reporting)
9. [Workflows and Approvals](#9-workflows-and-approvals)
10. [A/B Testing and Experimentation](#10-ab-testing-and-experimentation)
11. [Integrations](#11-integrations)
12. [Real-Time Bidding (OpenRTB)](#12-real-time-bidding-openrtb)
13. [Dynamic Creative Optimization (DCO)](#13-dynamic-creative-optimization-dco)
14. [CDP Integration](#14-cdp-integration)
15. [Platform Features](#15-platform-features)
16. [API Testing](#16-api-testing)
17. [Performance Testing](#17-performance-testing)
18. [Security Testing](#18-security-testing)
19. [Negative Testing](#19-negative-testing)
20. [Edge Cases and Boundary Testing](#20-edge-cases-and-boundary-testing)

---

## Test Case Conventions

### Priority Levels
- **P0**: Critical - Must pass before release
- **P1**: High - Should pass before release
- **P2**: Medium - Important but not blocking
- **P3**: Low - Nice to have

### Test Types
- **Functional**: Feature functionality testing
- **Integration**: Component integration testing
- **Regression**: Verify existing functionality
- **Smoke**: Quick sanity check
- **E2E**: End-to-end user workflow

---

## 1. Authentication and Authorization

### TC-AUTH-001: Valid User Login
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User account exists with valid credentials
- Application is accessible at the login page

**Test Steps**:
1. Navigate to the login page
2. Enter valid username
3. Enter valid password
4. Click "Login" button

**Expected Result**:
- User successfully logs in
- Redirected to dashboard
- User session created with valid token

**Test Data**:
- Username: `testuser@example.com`
- Password: `ValidPass123!`

---

### TC-AUTH-002: Invalid User Login
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User is on the login page

**Test Steps**:
1. Navigate to the login page
2. Enter invalid username
3. Enter invalid password
4. Click "Login" button

**Expected Result**:
- Login fails with appropriate error message
- User remains on login page
- No session token created

**Test Data**:
- Username: `invalid@example.com`
- Password: `WrongPassword`

---

### TC-AUTH-003: Role-Based Access Control (RBAC)
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User logged in with "Campaign Manager" role

**Test Steps**:
1. Log in as Campaign Manager
2. Attempt to access "Create Campaign" page
3. Attempt to access "System Administration" page

**Expected Result**:
- Campaign Manager can access campaign creation
- Campaign Manager cannot access admin functions
- Appropriate "Access Denied" message shown for unauthorized pages

---

### TC-AUTH-004: Session Timeout
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- User logged in successfully
- Session timeout configured to 30 minutes

**Test Steps**:
1. Log in successfully
2. Remain idle for 30+ minutes
3. Attempt to perform any action

**Expected Result**:
- Session expires after timeout period
- User redirected to login page
- Message indicates session has expired

---

### TC-AUTH-005: API Token Authentication
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Valid API token available

**Test Steps**:
1. Make API request to `/api/v1/management/campaigns` with valid Bearer token
2. Make API request with invalid token
3. Make API request without token

**Expected Result**:
- Request with valid token succeeds (200 OK)
- Request with invalid token fails (401 Unauthorized)
- Request without token fails (401 Unauthorized)

**Test Data**:
```bash
Authorization: Bearer campaign-express-demo-token
```

---

### TC-AUTH-006: Multi-Tenancy Isolation
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Two tenant accounts exist (Tenant A and Tenant B)
- Campaigns created for both tenants

**Test Steps**:
1. Log in as Tenant A user
2. View campaigns list
3. Log out and log in as Tenant B user
4. View campaigns list

**Expected Result**:
- Tenant A user sees only Tenant A campaigns
- Tenant B user sees only Tenant B campaigns
- No cross-tenant data leakage

---

## 2. Campaign Management

### TC-CAMP-001: Create New Campaign
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User logged in with campaign creation permissions
- At least one creative asset available

**Test Steps**:
1. Navigate to Campaigns page
2. Click "Create New Campaign" button
3. Enter campaign details:
   - Name: "Summer Sale 2026"
   - Type: "Promotional"
   - Start Date: "2026-06-01"
   - End Date: "2026-08-31"
   - Budget: "$50,000"
4. Select target audience segment
5. Assign creative assets
6. Click "Save Campaign"

**Expected Result**:
- Campaign created successfully
- Campaign appears in campaigns list with status "Draft"
- Success message displayed
- Campaign ID generated

---

### TC-CAMP-002: Edit Existing Campaign
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Campaign exists in "Draft" status

**Test Steps**:
1. Navigate to Campaigns page
2. Select existing campaign
3. Click "Edit" button
4. Modify campaign name from "Summer Sale 2026" to "Summer Mega Sale 2026"
5. Update budget from "$50,000" to "$75,000"
6. Click "Save Changes"

**Expected Result**:
- Campaign updated successfully
- Changes reflected in campaign details
- Audit log records the modification
- Updated timestamp shown

---

### TC-CAMP-003: Delete Campaign
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Campaign exists in "Draft" status
- No active journeys associated with campaign

**Test Steps**:
1. Navigate to Campaigns page
2. Select campaign to delete
3. Click "Delete" button
4. Confirm deletion in dialog

**Expected Result**:
- Campaign deleted successfully
- Campaign removed from list
- Confirmation message displayed
- Associated data archived (not hard deleted)

---

### TC-CAMP-004: Campaign Status Workflow
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Campaign exists in "Draft" status

**Test Steps**:
1. Open campaign in "Draft" status
2. Click "Submit for Review"
3. Campaign moves to "Under Review"
4. Approve campaign (as approver)
5. Campaign moves to "Approved"
6. Click "Activate Campaign"
7. Campaign moves to "Active"

**Expected Result**:
- Campaign progresses through all stages correctly
- Status transitions logged in audit trail
- Email notifications sent at each stage
- Campaign cannot skip required stages

**Valid Stages**: Draft → Under Review → Approved → Scheduled → Active → Paused → Completed → Archived

---

### TC-CAMP-005: Search and Filter Campaigns
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Multiple campaigns exist with different statuses and dates

**Test Steps**:
1. Navigate to Campaigns page
2. Enter search term in search box: "Summer"
3. Apply filter: Status = "Active"
4. Apply filter: Date Range = "Last 30 days"
5. Sort by: "Budget (Descending)"

**Expected Result**:
- Only matching campaigns displayed
- Filters applied cumulatively
- Sort order correct
- Result count shown
- Clear filters button works

---

### TC-CAMP-006: Campaign Duplication
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Existing campaign "Summer Sale 2026" exists

**Test Steps**:
1. Navigate to Campaigns page
2. Select campaign "Summer Sale 2026"
3. Click "Duplicate" button
4. Modify name to "Fall Sale 2026"
5. Update dates appropriately
6. Save duplicated campaign

**Expected Result**:
- New campaign created with all settings copied
- New campaign has unique ID
- Name clearly indicates it's a copy
- All associations (creatives, segments) copied

---

### TC-CAMP-007: Bulk Campaign Operations
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Multiple campaigns exist in "Draft" status

**Test Steps**:
1. Navigate to Campaigns page
2. Select multiple campaigns using checkboxes
3. Click "Bulk Actions" dropdown
4. Select "Submit for Review"
5. Confirm action

**Expected Result**:
- All selected campaigns move to "Under Review" status
- Bulk operation success message shown
- Individual notifications sent for each campaign
- Audit log updated for all campaigns

---

## 3. Creative Management

### TC-CREA-001: Upload Creative Asset
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User logged in with creative management permissions
- Valid image file available (PNG, JPG)

**Test Steps**:
1. Navigate to Creative Management page
2. Click "Upload Asset" button
3. Select image file (banner.png, 300x250px)
4. Enter asset details:
   - Name: "Summer Banner 2026"
   - Type: "Banner"
   - Dimensions: "300x250"
5. Click "Upload"

**Expected Result**:
- Asset uploaded successfully
- Asset visible in asset library
- Thumbnail generated
- File size and format validated
- Success message displayed

**Test Data**:
- File: banner.png (< 5MB)
- Format: PNG
- Dimensions: 300x250px

---

### TC-CREA-002: Brand Guideline Validation
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Brand guidelines configured with approved colors
- User attempting to upload creative

**Test Steps**:
1. Upload creative with non-approved brand colors
2. System analyzes creative
3. Review validation warnings

**Expected Result**:
- System detects color violations
- Warning message lists non-compliant colors
- Option to override with justification
- Compliance report generated

---

### TC-CREA-003: Creative Versioning
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Creative asset "Summer Banner v1" exists

**Test Steps**:
1. Select existing creative
2. Click "Upload New Version"
3. Upload updated version (banner-v2.png)
4. Add version notes: "Updated CTA button"
5. Save new version

**Expected Result**:
- New version created (v2)
- Previous version archived but accessible
- Version history displayed
- Ability to revert to previous version

---

### TC-CREA-004: Creative Search and Filtering
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Multiple creative assets exist

**Test Steps**:
1. Navigate to Asset Library
2. Search for "banner"
3. Filter by Type: "Image"
4. Filter by Dimensions: "300x250"
5. Filter by Date: "Last 30 days"

**Expected Result**:
- Only matching assets displayed
- Filters work correctly
- Search highlights matching terms
- Can clear all filters

---

### TC-CREA-005: Assign Creative to Campaign
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Campaign exists
- Creative assets available

**Test Steps**:
1. Open campaign editor
2. Go to "Creatives" section
3. Click "Add Creative"
4. Select creative from library
5. Set creative priority and weight
6. Save assignment

**Expected Result**:
- Creative assigned to campaign
- Creative appears in campaign's creative list
- Can assign multiple creatives
- Can set rotation rules

---

### TC-CREA-006: Delete Creative Asset
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Creative asset exists
- Creative not assigned to any active campaigns

**Test Steps**:
1. Navigate to Asset Library
2. Select creative to delete
3. Click "Delete" button
4. Confirm deletion

**Expected Result**:
- Creative deleted successfully
- Removed from library
- If assigned to campaigns, warning shown
- Archived rather than hard deleted

---

## 4. Journey Orchestration

### TC-JOUR-001: Create Customer Journey
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User has journey orchestration permissions
- Segments and channels configured

**Test Steps**:
1. Navigate to Journey Orchestration page
2. Click "Create New Journey"
3. Enter journey details:
   - Name: "Welcome Journey"
   - Trigger: "User Registration"
4. Add journey steps:
   - Step 1: Send welcome email (immediate)
   - Step 2: Wait 24 hours
   - Step 3: Send onboarding tips (push notification)
   - Step 4: Wait 3 days
   - Step 5: Send survey request (email)
5. Save journey

**Expected Result**:
- Journey created successfully
- Visual flow diagram displayed
- All steps configured correctly
- Journey in "Draft" status

---

### TC-JOUR-002: Test Journey Flow
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Journey "Welcome Journey" created

**Test Steps**:
1. Open journey editor
2. Click "Test Journey" button
3. Enter test user ID
4. Execute journey for test user
5. Monitor execution in real-time

**Expected Result**:
- Test mode executes journey without sending real messages
- All steps execute in sequence
- Logs show execution details
- Can verify timing and conditions

---

### TC-JOUR-003: Journey Branching Logic
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Journey with conditional branching exists

**Test Steps**:
1. Create journey with branch:
   - If user opens email → send follow-up
   - If user does not open email → send reminder
2. Activate journey
3. Test with user who opens email
4. Test with user who doesn't open email

**Expected Result**:
- Correct path taken based on condition
- Only appropriate messages sent
- Branch logic evaluated correctly
- User can be in only one branch at a time

---

### TC-JOUR-004: Journey State Machine
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- User enrolled in active journey

**Test Steps**:
1. Enroll user in journey
2. Verify initial state
3. Trigger state transition (e.g., email opened)
4. Verify state updated
5. Complete journey
6. Verify final state

**Expected Result**:
- User state tracked correctly
- State transitions logged
- Can query user's current state
- State persists across system restarts

---

### TC-JOUR-005: Schedule-Based Journey Trigger
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Journey configured with schedule trigger

**Test Steps**:
1. Create journey with trigger:
   - Type: "Schedule"
   - Frequency: "Daily at 9:00 AM"
   - Target: "Users in segment 'Active Customers'"
2. Activate journey
3. Wait for scheduled time
4. Verify execution

**Expected Result**:
- Journey executes at scheduled time
- All users in segment enrolled
- Execution logged
- Can view scheduled runs in calendar

---

### TC-JOUR-006: Journey Exit Conditions
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Journey with exit conditions configured

**Test Steps**:
1. Create journey with exit condition: "User makes purchase"
2. Enroll user in journey
3. User completes purchase during journey
4. Verify journey exits

**Expected Result**:
- User immediately exits journey
- No further messages sent
- Exit reason logged
- User can be re-enrolled if configured

---

## 5. Audience Segmentation

### TC-SEG-001: Create Audience Segment
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User has segmentation permissions
- User data available

**Test Steps**:
1. Navigate to Segmentation page
2. Click "Create Segment"
3. Define segment rules:
   - Name: "High-Value Customers"
   - Condition 1: Total purchases > $1000
   - Condition 2: Last purchase within 90 days
   - Condition 3: Email engagement rate > 30%
4. Save segment

**Expected Result**:
- Segment created successfully
- Rule engine validates syntax
- Estimated segment size shown
- Can preview sample users

---

### TC-SEG-002: Real-Time Segment Evaluation
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Segment "High-Value Customers" exists
- User profile data available

**Test Steps**:
1. Create test user with attributes below threshold
2. Verify user NOT in segment
3. Update user to meet criteria (make purchase > $1000)
4. Verify user NOW in segment
5. Check evaluation latency

**Expected Result**:
- Segment membership evaluated in real-time
- User added to segment immediately after qualifying
- Evaluation takes < 1 second
- Audit log shows membership change

---

### TC-SEG-003: Segment Overlap Analysis
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Multiple segments exist

**Test Steps**:
1. Navigate to Segmentation page
2. Select "Segment Analysis" tool
3. Choose segments to compare:
   - "High-Value Customers"
   - "Email Subscribers"
4. Click "Analyze Overlap"

**Expected Result**:
- Venn diagram showing overlap
- Count of users in each segment
- Count of users in overlap
- Can export overlap list

---

### TC-SEG-004: Dynamic vs Static Segments
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- User can create both segment types

**Test Steps**:
1. Create dynamic segment with rules
2. Create static segment by uploading user list
3. Verify dynamic segment updates automatically
4. Verify static segment remains fixed

**Expected Result**:
- Dynamic segment membership changes as data updates
- Static segment requires manual refresh
- Both types usable in campaigns
- Clear indication of segment type

---

### TC-SEG-005: Segment Export
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Segment "High-Value Customers" exists with members

**Test Steps**:
1. Open segment details
2. Click "Export Segment"
3. Select format: CSV
4. Choose fields to include
5. Download export

**Expected Result**:
- Export file generated successfully
- Contains all segment members
- Includes selected fields only
- PII properly masked if required

---

## 6. Loyalty Program

### TC-LOY-001: User Enrollment in Loyalty Program
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Loyalty program configured with 3 tiers
- User not enrolled in loyalty program

**Test Steps**:
1. Navigate to Loyalty page
2. Enter user ID to enroll
3. System assigns initial tier: "Green"
4. Verify enrollment

**Expected Result**:
- User enrolled successfully
- Assigned to Green tier
- Initial star balance: 0
- Welcome bonus applied (if configured)

**Loyalty Tiers**: Green (entry) → Gold (5000 stars) → Reserve (25000 stars)

---

### TC-LOY-002: Earn Stars on Purchase
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User enrolled in Green tier
- Earning rate: $1 = 10 stars

**Test Steps**:
1. User makes purchase of $100
2. System calculates stars: 100 × 10 = 1000 stars
3. Stars credited to user account
4. Verify balance

**Expected Result**:
- 1000 stars added to balance
- Transaction logged in history
- User receives notification
- Balance updates in real-time

---

### TC-LOY-003: Redeem Stars for Rewards
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User has 5000 stars in account
- Reward catalog available

**Test Steps**:
1. Navigate to Rewards page
2. Browse available rewards
3. Select "$25 Gift Card" (costs 2500 stars)
4. Click "Redeem"
5. Confirm redemption

**Expected Result**:
- 2500 stars deducted from balance
- New balance: 2500 stars
- Reward issued to user
- Redemption recorded in history

---

### TC-LOY-004: Tier Upgrade
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- User in Green tier with 4900 stars
- Gold tier threshold: 5000 stars

**Test Steps**:
1. User earns 150 stars (total: 5050)
2. System checks tier eligibility
3. User upgraded to Gold tier
4. Verify upgrade benefits applied

**Expected Result**:
- User automatically upgraded to Gold tier
- Notification sent about upgrade
- Benefits activated (higher earning rate, exclusive rewards)
- Tier badge displayed on profile

---

### TC-LOY-005: Star Expiration
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Stars configured to expire after 12 months
- User has stars earned 13 months ago

**Test Steps**:
1. Run star expiration job
2. Verify expired stars removed
3. Check user balance
4. Verify notification sent

**Expected Result**:
- Expired stars removed from balance
- User notified 30 days before expiration
- Expiration logged in transaction history
- Can query expiring stars

---

### TC-LOY-006: Loyalty Tier Benefits
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- User in Reserve tier (highest tier)

**Test Steps**:
1. Verify earning rate multiplier (2x)
2. Check access to exclusive rewards
3. Verify priority customer service
4. Test early access to sales

**Expected Result**:
- All tier benefits available
- Benefits automatically applied
- Can view benefit details
- Benefits persist across sessions

---

## 7. Multi-Channel Delivery

### TC-CHAN-001: Send Email Campaign
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- SendGrid integration configured
- Email template available
- Target segment defined

**Test Steps**:
1. Create campaign with channel: Email
2. Select email template
3. Configure sender: "noreply@campaignexpress.com"
4. Set subject: "Exclusive Summer Sale"
5. Select segment: "Email Subscribers"
6. Schedule send or send immediately
7. Click "Send"

**Expected Result**:
- Emails queued for sending
- SendGrid API called successfully
- Delivery status tracked
- Can monitor open/click rates

---

### TC-CHAN-002: Send SMS Campaign
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Twilio integration configured
- SMS opted-in users available

**Test Steps**:
1. Create campaign with channel: SMS
2. Compose message (< 160 characters)
3. Select segment: "SMS Subscribers"
4. Send SMS

**Expected Result**:
- SMS messages sent via Twilio
- Character count validated
- Delivery confirmation received
- Cost tracked per message

---

### TC-CHAN-003: Send Push Notification
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Mobile app users with push enabled
- Push notification template available

**Test Steps**:
1. Create campaign with channel: Push
2. Configure notification:
   - Title: "Flash Sale Alert"
   - Body: "50% off for next 2 hours!"
   - Deep link: "app://sales/flash"
3. Select segment: "Mobile App Users"
4. Send push

**Expected Result**:
- Push notifications delivered
- iOS and Android both supported
- Deep links work correctly
- Can track opens and conversions

---

### TC-CHAN-004: Multi-Channel Campaign
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- User opted in to multiple channels
- Campaign configured for all channels

**Test Steps**:
1. Create campaign with sequence:
   - Send Email immediately
   - If no open after 2 hours, send Push
   - If no action after 24 hours, send SMS
2. Activate campaign
3. Test with user who doesn't open email

**Expected Result**:
- Email sent first
- After 2 hours, push sent (email not opened)
- After 24 hours, SMS sent (no action)
- User receives sequence as configured

---

### TC-CHAN-005: Channel Suppression Lists
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Global suppression lists configured per channel

**Test Steps**:
1. Add user to email suppression list
2. Attempt to send email campaign to segment including suppressed user
3. Verify user excluded

**Expected Result**:
- Suppressed user not contacted
- Suppression reason logged
- Can add/remove from suppression list
- Respects user opt-out preferences

---

### TC-CHAN-006: Webhook Delivery
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Webhook URL configured
- Campaign with webhook channel

**Test Steps**:
1. Create campaign with webhook delivery
2. Configure webhook URL and payload
3. Send campaign
4. Verify webhook received

**Expected Result**:
- Webhook POST request sent
- Payload correct JSON format
- Retry logic on failure (3 attempts)
- Delivery status tracked

---

## 8. Budget Tracking and Reporting

### TC-BUD-001: Set Campaign Budget
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Campaign in Draft status

**Test Steps**:
1. Open campaign editor
2. Navigate to Budget section
3. Set total budget: $50,000
4. Set daily budget cap: $2,000
5. Save budget settings

**Expected Result**:
- Budget limits saved
- Daily pacing calculated
- Can view budget allocation
- Warning if budget too low for date range

---

### TC-BUD-002: Budget Pacing Alerts
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Campaign with $50,000 budget
- 80% threshold alert configured

**Test Steps**:
1. Campaign spends $40,000 (80% of budget)
2. System triggers pacing alert
3. Verify notification sent
4. Campaign continues running

**Expected Result**:
- Alert triggered at 80% threshold
- Email sent to campaign owner
- Dashboard shows warning
- Can adjust budget in response

**Alert Thresholds**: 80%, 100%, daily overrun

---

### TC-BUD-003: Budget Exhaustion
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Campaign with $50,000 budget
- Campaign actively spending

**Test Steps**:
1. Campaign reaches $50,000 spent
2. Verify campaign pauses automatically
3. Check notification sent

**Expected Result**:
- Campaign auto-paused at budget limit
- No overspend beyond limit
- Notification sent to owner
- Can allocate additional budget to resume

---

### TC-BUD-004: Generate Campaign Performance Report
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Campaign has run for at least 7 days
- Performance data collected

**Test Steps**:
1. Navigate to Reports page
2. Select "Campaign Performance" report
3. Choose campaign
4. Select date range: Last 7 days
5. Click "Generate Report"

**Expected Result**:
- Report generated successfully
- Shows key metrics:
  - Impressions
  - Clicks
  - CTR
  - Conversions
  - CVR
  - Spend
  - ROAS
- Can export as CSV/PDF

---

### TC-BUD-005: Schedule Automated Report
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- User has reporting permissions

**Test Steps**:
1. Create new scheduled report
2. Configure:
   - Report type: "Weekly Campaign Summary"
   - Schedule: "Every Monday at 9:00 AM"
   - Recipients: "manager@example.com"
   - Format: "PDF"
3. Save schedule

**Expected Result**:
- Scheduled report created
- Report sent at specified time
- Email contains PDF attachment
- Can modify or delete schedule

---

### TC-BUD-006: ROAS and ROI Calculation
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Campaign with spend and revenue data

**Test Steps**:
1. View campaign analytics
2. Verify ROAS calculation: (Revenue / Spend) × 100
3. Verify ROI calculation: ((Revenue - Spend) / Spend) × 100

**Expected Result**:
- ROAS calculated correctly
- ROI calculated correctly
- Formulas documented
- Metrics update in real-time

**Example**:
- Spend: $10,000
- Revenue: $45,000
- ROAS: 450%
- ROI: 350%

---

## 9. Workflows and Approvals

### TC-WORK-001: Submit Campaign for Approval
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Campaign in Draft status
- Approval workflow configured

**Test Steps**:
1. Complete campaign configuration
2. Click "Submit for Review"
3. Select reviewers from list
4. Add submission notes
5. Submit

**Expected Result**:
- Campaign status changes to "Under Review"
- Approval request sent to reviewers
- Submitter receives confirmation
- Audit log updated

---

### TC-WORK-002: Approve Campaign
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Campaign "Under Review"
- User is designated approver

**Test Steps**:
1. Log in as approver
2. Navigate to "Pending Approvals"
3. Review campaign details
4. Add approval comments
5. Click "Approve"

**Expected Result**:
- Campaign status changes to "Approved"
- Submitter notified of approval
- Campaign can now be activated
- Approval logged with timestamp

---

### TC-WORK-003: Reject Campaign
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Campaign "Under Review"
- User is designated approver

**Test Steps**:
1. Log in as approver
2. Review campaign
3. Identify issues (e.g., budget too high)
4. Add rejection comments
5. Click "Reject"

**Expected Result**:
- Campaign status returns to "Draft"
- Submitter notified with rejection reason
- Campaign owner can revise and resubmit
- Rejection logged

---

### TC-WORK-004: Multi-Level Approval
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Campaign with 3-level approval required
- Budget > $100,000

**Test Steps**:
1. Submit high-budget campaign
2. Level 1 approval (Manager)
3. Level 2 approval (Director)
4. Level 3 approval (VP)
5. Campaign fully approved

**Expected Result**:
- Each approval level processed in sequence
- Cannot skip levels
- All approvers notified at their stage
- Campaign activated only after final approval

---

### TC-WORK-005: Approval Delegation
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Approver going on vacation
- Delegation feature available

**Test Steps**:
1. Log in as approver
2. Navigate to Settings
3. Set up delegation:
   - Delegate to: "backup-approver@example.com"
   - Date range: "2026-07-01 to 2026-07-14"
4. Save delegation

**Expected Result**:
- Approval requests routed to delegate
- Original approver notified
- Delegation expires automatically
- Audit shows delegation details

---

### TC-WORK-006: Workflow Calendar View
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- Multiple campaigns in various workflow stages

**Test Steps**:
1. Navigate to Workflow Calendar
2. View monthly calendar
3. See campaigns by launch date
4. Color-coded by status

**Expected Result**:
- Visual calendar shows all campaigns
- Can click campaign for details
- Drag and drop to reschedule (if draft)
- Export calendar view

---

## 10. A/B Testing and Experimentation

### TC-EXP-001: Create A/B Test
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Campaign ready for testing
- Multiple creative variants available

**Test Steps**:
1. Navigate to Experiments page
2. Click "Create A/B Test"
3. Configure test:
   - Name: "Summer Banner Test"
   - Variants: A (control) vs B (treatment)
   - Traffic split: 50/50
   - Primary metric: CTR
   - Duration: 7 days
4. Assign creatives to variants
5. Start test

**Expected Result**:
- Test created and activated
- Traffic split correctly
- Data collection begins
- Can monitor results in real-time

---

### TC-EXP-002: A/B/n Multivariate Test
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Multiple variants to test (4 variants)

**Test Steps**:
1. Create A/B/n test with 4 variants
2. Set traffic split: 25% each
3. Define success metric
4. Run test for configured duration

**Expected Result**:
- Traffic distributed evenly across variants
- Each variant receives fair exposure
- Statistical significance calculated
- Winner determined at test end

---

### TC-EXP-003: Statistical Significance Check
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- A/B test running with sufficient data

**Test Steps**:
1. View test results
2. Check significance indicator
3. Verify confidence level (95%)
4. Review sample size

**Expected Result**:
- System calculates p-value
- Significance threshold: p < 0.05
- Visual indicator of confidence
- Recommendation to continue or conclude test

---

### TC-EXP-004: Automatic Winner Selection
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- A/B test configured with auto-winner
- Test duration completed

**Test Steps**:
1. Test runs for configured duration
2. System analyzes results
3. Winner determined (variant with highest CTR)
4. Automatic transition to winner

**Expected Result**:
- Winner selected based on metric
- All traffic shifted to winner
- Notification sent
- Test results archived

---

### TC-EXP-005: Manual Test Termination
**Priority**: P2  
**Type**: Functional

**Preconditions**:
- A/B test running

**Test Steps**:
1. Monitor test progress
2. Identify clear winner early
3. Manually stop test
4. Select winning variant

**Expected Result**:
- Test can be stopped early
- Winner selection confirmed
- Traffic shifted to winner
- Early termination logged

---

## 11. Integrations

### TC-INT-001: DSP Integration - The Trade Desk
**Priority**: P0  
**Type**: Integration

**Preconditions**:
- The Trade Desk credentials configured
- Campaign ready for DSP distribution

**Test Steps**:
1. Configure TTD integration
2. Map campaign to DSP
3. Push campaign to TTD
4. Verify campaign created in TTD

**Expected Result**:
- Campaign successfully pushed to TTD
- Campaign ID received from TTD
- Bidirectional sync established
- Can view TTD performance in CampaignExpress

---

### TC-INT-002: CDP Integration - Salesforce
**Priority**: P0  
**Type**: Integration

**Preconditions**:
- Salesforce credentials configured
- User segments defined

**Test Steps**:
1. Configure Salesforce CDC connection
2. Map segment to Salesforce list
3. Sync segment
4. Verify users created/updated in Salesforce

**Expected Result**:
- Segment synced to Salesforce
- User data mapping correct
- Bidirectional sync works
- Can trigger CampaignExpress campaigns from Salesforce

---

### TC-INT-003: Email Provider - SendGrid
**Priority**: P0  
**Type**: Integration

**Preconditions**:
- SendGrid API key configured
- Email template created

**Test Steps**:
1. Configure SendGrid integration
2. Test connection
3. Send test email
4. Check delivery status

**Expected Result**:
- Connection successful
- Email delivered via SendGrid
- Delivery status tracked
- Open/click events received via webhook

---

### TC-INT-004: SMS Provider - Twilio
**Priority**: P0  
**Type**: Integration

**Preconditions**:
- Twilio credentials configured
- Phone numbers provisioned

**Test Steps**:
1. Configure Twilio integration
2. Test connection
3. Send test SMS
4. Verify delivery

**Expected Result**:
- SMS sent successfully
- Delivery status received
- Cost tracked per message
- Error handling for invalid numbers

---

### TC-INT-005: Analytics Export - Power BI
**Priority**: P2  
**Type**: Integration

**Preconditions**:
- Power BI connector configured
- Campaign performance data available

**Test Steps**:
1. Configure Power BI connection
2. Select data to export
3. Schedule automatic sync
4. Open Power BI dashboard

**Expected Result**:
- Data exported to Power BI
- Refresh schedule works
- Dashboards update automatically
- Real-time metrics available

---

### TC-INT-006: DAM Integration - Bynder
**Priority**: P2  
**Type**: Integration

**Preconditions**:
- Bynder API credentials configured
- Assets available in Bynder

**Test Steps**:
1. Configure Bynder integration
2. Search Bynder assets from CampaignExpress
3. Import asset to CampaignExpress library
4. Use asset in campaign

**Expected Result**:
- Can browse Bynder assets
- Import maintains metadata
- Assets synced correctly
- Brand guidelines preserved

---

## 12. Real-Time Bidding (OpenRTB)

### TC-RTB-001: Valid OpenRTB Bid Request
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Bidding service running on port 8080
- CoLaNet inference model loaded

**Test Steps**:
1. Send POST request to `/v1/bid`
2. Include valid OpenRTB 2.6 bid request:
```json
{
  "id": "req-001",
  "imp": [{
    "id": "imp-1",
    "bidfloor": 0.50,
    "banner": {"w": 300, "h": 250}
  }],
  "site": {"domain": "example.com"},
  "user": {"id": "user-123"}
}
```
3. Verify response

**Expected Result**:
- Response received within 10ms (p99)
- Valid OpenRTB bid response
- Bid price calculated
- Response includes seatbid with creative

---

### TC-RTB-002: Multiple Impression Bidding
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- Bidding service running

**Test Steps**:
1. Send bid request with 3 impressions
2. Each impression different size/placement
3. Verify response

**Expected Result**:
- Separate bid for each impression
- Each bid optimized for placement
- Response time still < 10ms
- All impressions included in seatbid

---

### TC-RTB-003: Bid Floor Enforcement
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- Bidding service running

**Test Steps**:
1. Send bid request with bidfloor: 2.00
2. System calculates bid: 1.50
3. Verify no-bid response

**Expected Result**:
- No bid returned (below floor)
- Response status: 204 No Content
- Logged as "below floor"
- Proper OpenRTB compliance

---

### TC-RTB-004: User Targeting
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- User segments configured
- User data available

**Test Steps**:
1. Send bid request with user ID
2. System checks user segments
3. Matches to eligible campaigns
4. Returns personalized bid

**Expected Result**:
- User matched to segments
- Relevant campaigns selected
- Bid personalized based on user data
- ML model scored user-offer pair

---

### TC-RTB-005: Cache Hit Performance
**Priority**: P1  
**Type**: Performance

**Preconditions**:
- Redis cache warmed up
- Same user bidding repeatedly

**Test Steps**:
1. Send 10 bid requests for same user
2. Measure response time
3. Check cache hit rate

**Expected Result**:
- First request: ~8ms (cache miss)
- Subsequent requests: ~2ms (cache hit)
- Cache hit rate > 80%
- L1 (DashMap) used before L2 (Redis)

---

### TC-RTB-006: Invalid Bid Request Handling
**Priority**: P1  
**Type**: Negative

**Preconditions**:
- Bidding service running

**Test Steps**:
1. Send malformed JSON
2. Send request missing required fields
3. Send request with invalid values

**Expected Result**:
- 400 Bad Request response
- Error message indicates issue
- No system crash
- Request logged for analysis

---

## 13. Dynamic Creative Optimization (DCO)

### TC-DCO-001: Create DCO Template
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User has DCO permissions
- Asset library populated

**Test Steps**:
1. Navigate to DCO Templates
2. Click "Create Template"
3. Define template structure:
   - Layout: "Banner 300x250"
   - Slots: headline, image, CTA button
4. Add variants for each slot
5. Save template

**Expected Result**:
- Template created successfully
- All slots configured
- Variants assigned
- Preview shows sample combinations

---

### TC-DCO-002: Thompson Sampling Variant Selection
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- DCO template with 3 variants per slot
- Thompson Sampling enabled

**Test Steps**:
1. Activate DCO campaign
2. Serve 1000 impressions
3. Monitor variant distribution
4. Check performance tracking

**Expected Result**:
- Initial distribution explores all variants
- System learns from performance
- Better variants served more frequently
- Thompson Sampling algorithm applied

---

### TC-DCO-003: Variant Performance Tracking
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- DCO campaign active
- Multiple variants in rotation

**Test Steps**:
1. View DCO performance dashboard
2. Check metrics per variant:
   - Impressions
   - Clicks
   - CTR
   - Conversions
3. Compare variants

**Expected Result**:
- Performance tracked per variant
- Clear winner identified over time
- Can pause underperforming variants
- Visualization of performance trends

---

### TC-DCO-004: Personalized Creative Assembly
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- DCO template configured
- User profile data available

**Test Steps**:
1. Serve DCO creative to user
2. System selects variants based on:
   - User preferences
   - Past behavior
   - Segment membership
3. Creative assembled in real-time

**Expected Result**:
- Creative personalized per user
- Assembly takes < 5ms
- Correct variants selected
- Consistent branding maintained

---

## 14. CDP Integration

### TC-CDP-001: Sync Segment to Salesforce
**Priority**: P0  
**Type**: Integration

**Preconditions**:
- Salesforce Marketing Cloud connected
- Segment "VIP Customers" defined

**Test Steps**:
1. Navigate to CDP Integrations
2. Select Salesforce
3. Map segment to Salesforce Data Extension
4. Configure sync schedule: Daily at 2:00 AM
5. Trigger initial sync

**Expected Result**:
- Segment synced successfully
- Users created in Salesforce DE
- Field mapping correct
- Sync schedule saved

---

### TC-CDP-002: Inbound Event from Segment.io
**Priority**: P1  
**Type**: Integration

**Preconditions**:
- Segment.io webhook configured
- Campaign triggered by event

**Test Steps**:
1. Segment.io sends "Product Viewed" event
2. CampaignExpress receives event
3. Event matched to trigger
4. Campaign activated for user

**Expected Result**:
- Event received and processed
- User entered journey
- Latency < 2 seconds
- Event logged in analytics

---

### TC-CDP-003: Bidirectional Data Sync
**Priority**: P1  
**Type**: Integration

**Preconditions**:
- Adobe Experience Platform connected
- Bidirectional sync configured

**Test Steps**:
1. Update user preference in CampaignExpress
2. Change synced to Adobe
3. Update user attribute in Adobe
4. Change synced to CampaignExpress

**Expected Result**:
- Both directions sync correctly
- No data loss
- Conflict resolution works
- Sync logs available

---

## 15. Platform Features

### TC-PLAT-001: API Key Management
**Priority**: P0  
**Type**: Functional

**Preconditions**:
- User has API key management permissions

**Test Steps**:
1. Navigate to API Keys page
2. Click "Generate New Key"
3. Set key permissions and expiry
4. Generate key
5. Copy and test key

**Expected Result**:
- API key generated successfully
- Key shown only once for security
- Can set granular permissions
- Can revoke key later

---

### TC-PLAT-002: Audit Logging
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- System has audit logging enabled

**Test Steps**:
1. Perform various actions:
   - Create campaign
   - Update user
   - Delete segment
2. Navigate to Audit Logs
3. Search for actions
4. Filter by user/date/action

**Expected Result**:
- All actions logged
- Log includes: user, timestamp, action, details
- Can export logs
- Logs immutable

---

### TC-PLAT-003: Role Management
**Priority**: P1  
**Type**: Functional

**Preconditions**:
- User has admin permissions

**Test Steps**:
1. Create custom role: "Campaign Analyst"
2. Assign permissions:
   - View campaigns (read-only)
   - Create reports
   - View analytics
3. Assign role to user
4. Test permissions

**Expected Result**:
- Custom role created
- Permissions enforced correctly
- User can perform allowed actions only
- Audit log tracks role changes

---

### TC-PLAT-004: Multi-Tenant Isolation
**Priority**: P0  
**Type**: Security

**Preconditions**:
- Multiple tenants configured

**Test Steps**:
1. Create data in Tenant A
2. Log in as Tenant B user
3. Attempt to access Tenant A data via:
   - UI navigation
   - API calls with modified IDs
   - Direct database queries

**Expected Result**:
- Complete data isolation
- Tenant B cannot access Tenant A data
- Access denied errors returned
- Security log records attempts

---

## 16. API Testing

### TC-API-001: REST API Health Check
**Priority**: P0  
**Type**: Functional

**Test Steps**:
```bash
curl http://localhost:8080/health
```

**Expected Result**:
```json
{
  "status": "healthy",
  "timestamp": "2026-02-14T12:00:00Z"
}
```

---

### TC-API-002: List Campaigns API
**Priority**: P0  
**Type**: Functional

**Test Steps**:
```bash
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/management/campaigns
```

**Expected Result**:
- 200 OK response
- JSON array of campaigns
- Each campaign has required fields
- Pagination headers included

---

### TC-API-003: Create Campaign API
**Priority**: P0  
**Type**: Functional

**Test Steps**:
```bash
curl -X POST \
  -H "Authorization: Bearer campaign-express-demo-token" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "API Test Campaign",
    "type": "promotional",
    "budget": 10000,
    "start_date": "2026-06-01",
    "end_date": "2026-06-30"
  }' \
  http://localhost:8080/api/v1/management/campaigns
```

**Expected Result**:
- 201 Created response
- Campaign ID returned
- Location header with new resource URL
- Campaign created in database

---

### TC-API-004: API Rate Limiting
**Priority**: P1  
**Type**: Functional

**Test Steps**:
1. Send 100 API requests rapidly
2. Exceed rate limit threshold
3. Check response

**Expected Result**:
- First N requests succeed (200 OK)
- Subsequent requests: 429 Too Many Requests
- Rate limit headers included:
  - X-RateLimit-Limit
  - X-RateLimit-Remaining
  - X-RateLimit-Reset

---

### TC-API-005: API Error Handling
**Priority**: P1  
**Type**: Negative

**Test Steps**:
1. Send request with invalid JSON
2. Send request with missing required field
3. Send request to non-existent endpoint

**Expected Result**:
- Appropriate HTTP status codes (400, 404)
- Error messages in consistent format
- Error details helpful for debugging
- No sensitive data in error messages

---

### TC-API-006: API Pagination
**Priority**: P2  
**Type**: Functional

**Test Steps**:
```bash
curl -H "Authorization: Bearer campaign-express-demo-token" \
  "http://localhost:8080/api/v1/management/campaigns?page=2&per_page=10"
```

**Expected Result**:
- 10 items returned
- Pagination metadata in response
- Links to next/previous pages
- Total count included

---

## 17. Performance Testing

### TC-PERF-001: Throughput Test
**Priority**: P0  
**Type**: Performance

**Test Objective**: Verify 50M offers/hour throughput

**Test Steps**:
1. Set up 20-node Kubernetes cluster
2. Configure load generators
3. Run 1-hour sustained load test
4. Monitor metrics

**Expected Result**:
- Throughput: ≥ 50M offers/hour
- Even distribution across nodes
- No errors under load
- Resource utilization < 80%

---

### TC-PERF-002: Latency Test
**Priority**: P0  
**Type**: Performance

**Test Objective**: Verify sub-10ms p99 latency

**Test Steps**:
1. Send 10,000 bid requests
2. Measure response times
3. Calculate percentiles

**Expected Result**:
- p50 latency: < 5ms
- p95 latency: < 8ms
- p99 latency: < 10ms
- p99.9 latency: < 15ms

---

### TC-PERF-003: Concurrent Users Test
**Priority**: P1  
**Type**: Performance

**Test Objective**: Support 1000 concurrent users

**Test Steps**:
1. Simulate 1000 concurrent dashboard users
2. Each performing typical actions
3. Monitor system response
4. Check for degradation

**Expected Result**:
- UI remains responsive
- No request timeouts
- Database connection pool sufficient
- Session management scales

---

### TC-PERF-004: Database Query Performance
**Priority**: P1  
**Type**: Performance

**Test Steps**:
1. Execute complex campaign queries
2. Test large segment queries
3. Check analytics aggregations
4. Monitor query execution time

**Expected Result**:
- Simple queries: < 100ms
- Complex queries: < 500ms
- Aggregations: < 2 seconds
- Proper indexes utilized

---

### TC-PERF-005: Cache Performance
**Priority**: P1  
**Type**: Performance

**Test Steps**:
1. Warm up cache with user data
2. Send repeated bid requests
3. Monitor cache hit rate
4. Measure latency improvement

**Expected Result**:
- Cache hit rate: > 80%
- Cache miss latency: ~8ms
- Cache hit latency: ~2ms
- 4x improvement with cache

---

### TC-PERF-006: Memory Leak Test
**Priority**: P1  
**Type**: Performance

**Test Steps**:
1. Run system under sustained load
2. Monitor memory usage over 24 hours
3. Check for gradual increase
4. Verify garbage collection

**Expected Result**:
- Memory usage stable
- No continuous growth
- GC cycles normal
- No memory leaks detected

---

## 18. Security Testing

### TC-SEC-001: SQL Injection Test
**Priority**: P0  
**Type**: Security

**Test Steps**:
1. Attempt SQL injection in login form:
   - Username: `' OR '1'='1`
2. Attempt in search fields
3. Attempt in API parameters

**Expected Result**:
- All injection attempts blocked
- Parameterized queries used
- Input validation effective
- Attempts logged

---

### TC-SEC-002: Cross-Site Scripting (XSS)
**Priority**: P0  
**Type**: Security

**Test Steps**:
1. Attempt to inject script in form fields:
   - `<script>alert('XSS')</script>`
2. Try in campaign name, description
3. Check if script executes

**Expected Result**:
- Scripts not executed
- Input sanitized/escaped
- Content Security Policy enforced
- XSS attempts blocked

---

### TC-SEC-003: Authentication Token Security
**Priority**: P0  
**Type**: Security

**Test Steps**:
1. Inspect JWT token
2. Check token expiry
3. Attempt to use expired token
4. Attempt to tamper with token

**Expected Result**:
- Token properly signed
- Expiry enforced (30 min)
- Tampering detected
- Secure token storage (HttpOnly cookies)

---

### TC-SEC-004: HTTPS Enforcement
**Priority**: P0  
**Type**: Security

**Test Steps**:
1. Attempt HTTP connection
2. Verify redirect to HTTPS
3. Check certificate validity
4. Test TLS version

**Expected Result**:
- HTTP redirects to HTTPS
- Valid SSL certificate
- TLS 1.2+ enforced
- Strong cipher suites only

---

### TC-SEC-005: Sensitive Data Exposure
**Priority**: P0  
**Type**: Security

**Test Steps**:
1. Check API responses for sensitive data
2. Inspect error messages
3. Check logs for PII
4. Verify data masking

**Expected Result**:
- No passwords in responses
- PII properly masked
- Secure error messages
- Logs sanitized

---

### TC-SEC-006: Dependency Vulnerability Scan
**Priority**: P1  
**Type**: Security

**Test Steps**:
1. Run `cargo audit` for Rust dependencies
2. Run `npm audit` for JavaScript dependencies
3. Review vulnerability report
4. Verify no high/critical issues

**Expected Result**:
- No high/critical vulnerabilities
- Dependencies up to date
- Security advisories reviewed
- Patches applied

---

## 19. Negative Testing

### TC-NEG-001: Invalid Input Handling
**Priority**: P1  
**Type**: Negative

**Test Steps**:
1. Submit campaign with negative budget
2. Submit campaign with end date before start date
3. Submit campaign with empty required fields

**Expected Result**:
- Validation errors returned
- Clear error messages
- No system crash
- User can correct and resubmit

---

### TC-NEG-002: Concurrent Modification Conflict
**Priority**: P2  
**Type**: Negative

**Test Steps**:
1. User A opens campaign editor
2. User B opens same campaign
3. User A saves changes
4. User B saves changes

**Expected Result**:
- Optimistic locking detects conflict
- User B warned of conflict
- Can review differences
- Can merge or override

---

### TC-NEG-003: Database Connection Loss
**Priority**: P1  
**Type**: Negative

**Test Steps**:
1. System running normally
2. Disconnect database
3. Attempt operations
4. Reconnect database

**Expected Result**:
- Graceful error handling
- User-friendly error messages
- Auto-reconnect when available
- No data corruption

---

### TC-NEG-004: NATS Queue Unavailable
**Priority**: P1  
**Type**: Negative

**Test Steps**:
1. System running
2. Stop NATS service
3. Attempt to queue message
4. Restart NATS

**Expected Result**:
- Circuit breaker activates
- Retry logic engages
- Messages buffered locally
- Automatic recovery when NATS returns

---

### TC-NEG-005: Redis Cache Failure
**Priority**: P1  
**Type**: Negative

**Test Steps**:
1. System using cache
2. Stop Redis
3. Send bid requests
4. Verify fallback behavior

**Expected Result**:
- System continues without cache
- Fallback to database
- Performance degraded but functional
- Alerts triggered

---

## 20. Edge Cases and Boundary Testing

### TC-EDGE-001: Maximum Campaign Duration
**Priority**: P2  
**Type**: Boundary

**Test Steps**:
1. Create campaign with 10-year duration
2. Verify system accepts
3. Check budget calculations
4. Test date handling

**Expected Result**:
- System accepts long duration
- No overflow errors
- Proper date arithmetic
- Pacing calculations correct

---

### TC-EDGE-002: Zero Budget Campaign
**Priority**: P2  
**Type**: Boundary

**Test Steps**:
1. Create campaign with $0 budget
2. Attempt to activate
3. Verify behavior

**Expected Result**:
- Validation warning shown
- Can save but not activate
- Clear message about limitation
- Use case: testing campaigns

---

### TC-EDGE-003: Maximum Segment Size
**Priority**: P2  
**Type**: Boundary

**Test Steps**:
1. Create segment matching 10M users
2. Attempt operations on large segment
3. Monitor performance
4. Check memory usage

**Expected Result**:
- System handles large segments
- Queries optimized with pagination
- No memory issues
- Operations complete successfully

---

### TC-EDGE-004: Unicode and Special Characters
**Priority**: P2  
**Type**: Boundary

**Test Steps**:
1. Create campaign with Unicode name: "夏季促销 2026"
2. Include emojis: "🎉 Summer Sale 🎉"
3. Test special characters in descriptions

**Expected Result**:
- All Unicode properly stored
- Display correct in UI
- Search works with Unicode
- No encoding issues

---

### TC-EDGE-005: Simultaneous Campaign Start
**Priority**: P2  
**Type**: Boundary

**Test Steps**:
1. Schedule 100 campaigns to start at same time
2. Wait for scheduled time
3. Monitor activation

**Expected Result**:
- All campaigns activate
- No race conditions
- Queue processing efficient
- No campaigns missed

---

### TC-EDGE-006: Campaign End Date in Past
**Priority**: P2  
**Type**: Boundary

**Test Steps**:
1. Attempt to create campaign with end date in past
2. Verify validation
3. Test edge case: end date = current time

**Expected Result**:
- Past end dates rejected
- Clear validation message
- End date = now: allowed but warning shown
- No system errors

---

## Appendices

### Appendix A: Test Execution Summary Template

| Metric | Value |
|--------|-------|
| Total Test Cases | [count] |
| Executed | [count] |
| Passed | [count] |
| Failed | [count] |
| Blocked | [count] |
| Pass Rate | [percentage] |

### Appendix B: Defect Summary Template

| Severity | Open | Closed | Total |
|----------|------|--------|-------|
| Critical | | | |
| High | | | |
| Medium | | | |
| Low | | | |

### Appendix C: Test Environment Details

| Component | Version | Configuration |
|-----------|---------|---------------|
| CampaignExpress | 0.1.0 | 20-node cluster |
| Rust | 1.77+ | Release build |
| Kubernetes | 1.28+ | AKS |
| NATS | 2.10+ | 3-node cluster |
| Redis | 7.0+ | 6-node cluster |
| ClickHouse | 24.0+ | Single node |

### Appendix D: Test Data Sets

- **Users**: 10,000 test users with varied attributes
- **Campaigns**: 100+ sample campaigns
- **Creatives**: 500+ assets
- **Segments**: 50+ audience segments
- **OpenRTB Requests**: 1000+ sample bid requests

### Appendix E: References

- [Test Strategy Document](TEST_STRATEGY.md)
- [Architecture Documentation](ARCHITECTURE.md)
- [API Documentation](../README.md#api)
- [OpenRTB 2.6 Specification](https://www.iab.com/guidelines/openrtb/)

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-02-14 | QA Team | Initial manual test cases |

---

**Test Execution Notes**:
- Execute P0 tests before every release
- Execute P1 tests for major releases
- Execute P2 tests quarterly
- Update test cases as features evolve
- Report any test case issues to QA lead

---

*This document is confidential and proprietary. All rights reserved.*
