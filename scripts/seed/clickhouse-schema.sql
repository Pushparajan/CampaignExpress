-- =============================================================================
-- Campaign Express — ClickHouse Schema + Seed Data
-- Creates all tables and populates them with realistic test data.
-- Run: clickhouse-client --host localhost --query "$(cat scripts/seed/clickhouse-schema.sql)"
--   or: curl 'http://localhost:8123/' --data-binary @scripts/seed/clickhouse-schema.sql
-- =============================================================================

-- ── Database ────────────────────────────────────────────────────────────────

CREATE DATABASE IF NOT EXISTS campaign_express;

-- ── Bid Events Table ────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS campaign_express.bid_events
(
    event_id       String,
    request_id     String,
    impression_id  String,
    timestamp      DateTime64(3) DEFAULT now64(3),
    event_type     Enum8('request' = 1, 'bid' = 2, 'win' = 3, 'impression' = 4, 'click' = 5),
    campaign_id    String,
    offer_id       String,
    user_id        String,
    bid_price      Float64,
    win_price      Float64 DEFAULT 0.0,
    site_domain    String,
    device_os      String,
    device_type    UInt8 DEFAULT 0,
    geo_country    String DEFAULT '',
    geo_region     String DEFAULT '',
    geo_city       String DEFAULT '',
    banner_w       UInt16 DEFAULT 0,
    banner_h       UInt16 DEFAULT 0,
    latency_ms     Float64 DEFAULT 0.0,
    node_id        String DEFAULT '',
    currency       String DEFAULT 'USD'
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (timestamp, request_id, event_type)
TTL timestamp + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;

-- ── Campaign Analytics Aggregates ───────────────────────────────────────────

CREATE TABLE IF NOT EXISTS campaign_express.campaign_stats_daily
(
    date           Date,
    campaign_id    String,
    requests       UInt64 DEFAULT 0,
    bids           UInt64 DEFAULT 0,
    wins           UInt64 DEFAULT 0,
    impressions    UInt64 DEFAULT 0,
    clicks         UInt64 DEFAULT 0,
    spend          Float64 DEFAULT 0.0,
    revenue        Float64 DEFAULT 0.0,
    avg_bid_price  Float64 DEFAULT 0.0,
    avg_win_price  Float64 DEFAULT 0.0,
    avg_latency_ms Float64 DEFAULT 0.0,
    currency       String DEFAULT 'USD'
) ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, campaign_id)
SETTINGS index_granularity = 8192;

-- ── User Segments ───────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS campaign_express.user_segments
(
    user_id        String,
    segment_id     String,
    segment_name   String,
    score          Float64 DEFAULT 1.0,
    updated_at     DateTime64(3) DEFAULT now64(3)
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY (user_id, segment_id)
SETTINGS index_granularity = 8192;

-- ── Offer Performance ───────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS campaign_express.offer_performance
(
    offer_id       String,
    campaign_id    String,
    creative_id    String,
    date           Date,
    impressions    UInt64 DEFAULT 0,
    clicks         UInt64 DEFAULT 0,
    conversions    UInt64 DEFAULT 0,
    spend          Float64 DEFAULT 0.0,
    ctr            Float64 DEFAULT 0.0,
    cvr            Float64 DEFAULT 0.0,
    cpm            Float64 DEFAULT 0.0,
    roas           Float64 DEFAULT 0.0
) ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, offer_id, campaign_id)
SETTINGS index_granularity = 8192;

-- ── Node Metrics ────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS campaign_express.node_metrics
(
    node_id           String,
    timestamp         DateTime64(3) DEFAULT now64(3),
    requests_per_sec  Float64 DEFAULT 0.0,
    avg_latency_ms    Float64 DEFAULT 0.0,
    p99_latency_ms    Float64 DEFAULT 0.0,
    active_agents     UInt16 DEFAULT 0,
    cpu_percent       Float64 DEFAULT 0.0,
    memory_mb         Float64 DEFAULT 0.0,
    cache_hit_rate    Float64 DEFAULT 0.0
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (timestamp, node_id)
TTL timestamp + INTERVAL 30 DAY
SETTINGS index_granularity = 8192;

-- =============================================================================
-- SEED DATA
-- =============================================================================

-- ── Campaigns (10 test campaigns) ───────────────────────────────────────────

INSERT INTO campaign_express.campaign_stats_daily
    (date, campaign_id, requests, bids, wins, impressions, clicks, spend, revenue, avg_bid_price, avg_win_price, avg_latency_ms)
VALUES
    (today(), 'camp-001', 50000, 12000, 8000,  7500,  225,  375.00,  1125.00, 1.50, 0.98, 8.2),
    (today(), 'camp-002', 35000,  9000, 5500,  5200,  156,  260.00,   780.00, 1.25, 0.85, 7.5),
    (today(), 'camp-003', 80000, 25000, 18000, 17000,  680,  850.00,  3400.00, 2.10, 1.35, 9.1),
    (today(), 'camp-004', 20000,  5000, 3200,  3000,   60,  150.00,   300.00, 0.90, 0.62, 6.8),
    (today(), 'camp-005', 60000, 18000, 12000, 11500,  460,  575.00,  2300.00, 1.80, 1.10, 8.7),
    (today(), 'camp-006', 45000, 11000, 7500,  7000,  280,  350.00,  1400.00, 1.40, 0.95, 7.9),
    (today(), 'camp-007', 90000, 30000, 22000, 21000, 1050,  1050.00, 5250.00, 2.50, 1.60, 10.3),
    (today(), 'camp-008', 15000,  3500, 2200,  2000,   40,  100.00,   160.00, 0.75, 0.52, 6.2),
    (today(), 'camp-009', 55000, 14000, 9500,  9000,  360,  450.00,  1800.00, 1.65, 1.05, 8.4),
    (today(), 'camp-010', 70000, 22000, 15000, 14000,  700,  700.00,  3500.00, 2.20, 1.40, 9.5),
    -- Yesterday's data
    (today() - 1, 'camp-001', 48000, 11500, 7600, 7100, 213, 355.00, 1065.00, 1.48, 0.95, 8.0),
    (today() - 1, 'camp-002', 33000,  8500, 5200, 4900, 147, 245.00,  735.00, 1.22, 0.82, 7.3),
    (today() - 1, 'camp-003', 77000, 24000, 17000, 16000, 640, 800.00, 3200.00, 2.05, 1.30, 8.9),
    (today() - 1, 'camp-005', 58000, 17000, 11500, 10800, 432, 540.00, 2160.00, 1.75, 1.07, 8.5),
    (today() - 1, 'camp-007', 87000, 29000, 21000, 20000, 1000, 1000.00, 5000.00, 2.45, 1.55, 10.1);

-- ── Bid Events (50 recent sample events) ────────────────────────────────────

INSERT INTO campaign_express.bid_events
    (event_id, request_id, impression_id, event_type, campaign_id, offer_id, user_id, bid_price, win_price, site_domain, device_os, device_type, geo_country, geo_region, geo_city, banner_w, banner_h, latency_ms, node_id)
VALUES
    ('evt-001', 'req-001', 'imp-1', 'request',    'camp-001', '',         'user-101', 0.00, 0.00, 'techblog.com',   'iOS',     2, 'US', 'CA', 'San Francisco', 300, 250, 5.2, 'dev-01'),
    ('evt-002', 'req-001', 'imp-1', 'bid',        'camp-001', 'offer-A1', 'user-101', 1.50, 0.00, 'techblog.com',   'iOS',     2, 'US', 'CA', 'San Francisco', 300, 250, 8.1, 'dev-01'),
    ('evt-003', 'req-001', 'imp-1', 'win',        'camp-001', 'offer-A1', 'user-101', 1.50, 0.98, 'techblog.com',   'iOS',     2, 'US', 'CA', 'San Francisco', 300, 250, 8.1, 'dev-01'),
    ('evt-004', 'req-001', 'imp-1', 'impression', 'camp-001', 'offer-A1', 'user-101', 1.50, 0.98, 'techblog.com',   'iOS',     2, 'US', 'CA', 'San Francisco', 300, 250, 8.1, 'dev-01'),
    ('evt-005', 'req-002', 'imp-1', 'request',    'camp-003', '',         'user-202', 0.00, 0.00, 'news24.com',     'Android', 2, 'US', 'NY', 'New York',      728, 90,  4.8, 'dev-01'),
    ('evt-006', 'req-002', 'imp-1', 'bid',        'camp-003', 'offer-C1', 'user-202', 2.10, 0.00, 'news24.com',     'Android', 2, 'US', 'NY', 'New York',      728, 90,  9.5, 'dev-01'),
    ('evt-007', 'req-002', 'imp-1', 'win',        'camp-003', 'offer-C1', 'user-202', 2.10, 1.35, 'news24.com',     'Android', 2, 'US', 'NY', 'New York',      728, 90,  9.5, 'dev-01'),
    ('evt-008', 'req-003', 'imp-1', 'request',    'camp-005', '',         'user-303', 0.00, 0.00, 'shopping.io',    'Windows', 1, 'UK', '',   'London',         160, 600, 6.1, 'dev-01'),
    ('evt-009', 'req-003', 'imp-1', 'bid',        'camp-005', 'offer-E1', 'user-303', 1.80, 0.00, 'shopping.io',    'Windows', 1, 'UK', '',   'London',         160, 600, 8.9, 'dev-01'),
    ('evt-010', 'req-004', 'imp-1', 'request',    'camp-002', '',         'user-404', 0.00, 0.00, 'gamezone.net',   'iOS',     5, 'DE', '',   'Berlin',         320, 50,  5.5, 'dev-01'),
    ('evt-011', 'req-004', 'imp-1', 'bid',        'camp-002', 'offer-B1', 'user-404', 1.25, 0.00, 'gamezone.net',   'iOS',     5, 'DE', '',   'Berlin',         320, 50,  7.8, 'dev-01'),
    ('evt-012', 'req-004', 'imp-1', 'win',        'camp-002', 'offer-B1', 'user-404', 1.25, 0.85, 'gamezone.net',   'iOS',     5, 'DE', '',   'Berlin',         320, 50,  7.8, 'dev-01'),
    ('evt-013', 'req-004', 'imp-1', 'impression', 'camp-002', 'offer-B1', 'user-404', 1.25, 0.85, 'gamezone.net',   'iOS',     5, 'DE', '',   'Berlin',         320, 50,  7.8, 'dev-01'),
    ('evt-014', 'req-004', 'imp-1', 'click',      'camp-002', 'offer-B1', 'user-404', 1.25, 0.85, 'gamezone.net',   'iOS',     5, 'DE', '',   'Berlin',         320, 50,  7.8, 'dev-01'),
    ('evt-015', 'req-005', 'imp-1', 'request',    'camp-007', '',         'user-505', 0.00, 0.00, 'travel.com',     'macOS',   1, 'US', 'FL', 'Miami',          300, 250, 4.3, 'dev-01'),
    ('evt-016', 'req-005', 'imp-1', 'bid',        'camp-007', 'offer-G1', 'user-505', 2.50, 0.00, 'travel.com',     'macOS',   1, 'US', 'FL', 'Miami',          300, 250, 10.2,'dev-01'),
    ('evt-017', 'req-005', 'imp-1', 'win',        'camp-007', 'offer-G1', 'user-505', 2.50, 1.60, 'travel.com',     'macOS',   1, 'US', 'FL', 'Miami',          300, 250, 10.2,'dev-01'),
    ('evt-018', 'req-005', 'imp-1', 'impression', 'camp-007', 'offer-G1', 'user-505', 2.50, 1.60, 'travel.com',     'macOS',   1, 'US', 'FL', 'Miami',          300, 250, 10.2,'dev-01'),
    ('evt-019', 'req-006', 'imp-1', 'request',    'camp-009', '',         'user-606', 0.00, 0.00, 'fitness.app',    'iOS',     2, 'CA', 'ON', 'Toronto',        300, 250, 5.7, 'dev-01'),
    ('evt-020', 'req-006', 'imp-1', 'bid',        'camp-009', 'offer-I1', 'user-606', 1.65, 0.00, 'fitness.app',    'iOS',     2, 'CA', 'ON', 'Toronto',        300, 250, 8.6, 'dev-01');

-- ── User Segments (sample audience data) ────────────────────────────────────

INSERT INTO campaign_express.user_segments (user_id, segment_id, segment_name, score)
VALUES
    ('user-101', 'seg-tech',       'Technology Enthusiasts',     0.92),
    ('user-101', 'seg-premium',    'Premium Users',              0.85),
    ('user-101', 'seg-mobile',     'Mobile First',               0.78),
    ('user-202', 'seg-news',       'News Readers',               0.88),
    ('user-202', 'seg-commute',    'Commuters',                  0.75),
    ('user-303', 'seg-shoppers',   'Online Shoppers',            0.95),
    ('user-303', 'seg-premium',    'Premium Users',              0.80),
    ('user-303', 'seg-intl',       'International Users',        0.70),
    ('user-404', 'seg-gamers',     'Gamers',                     0.91),
    ('user-404', 'seg-youth',      'Youth 18-24',                0.82),
    ('user-505', 'seg-travel',     'Frequent Travelers',         0.90),
    ('user-505', 'seg-premium',    'Premium Users',              0.88),
    ('user-505', 'seg-high-value', 'High Lifetime Value',        0.93),
    ('user-606', 'seg-fitness',    'Health & Fitness',            0.87),
    ('user-606', 'seg-mobile',     'Mobile First',               0.76),
    ('user-707', 'seg-auto',       'Automotive Intenders',        0.84),
    ('user-707', 'seg-suburban',   'Suburban Families',            0.79),
    ('user-808', 'seg-finance',    'Financial Services',           0.86),
    ('user-808', 'seg-high-value', 'High Lifetime Value',          0.91),
    ('user-909', 'seg-food',       'Foodies',                      0.83),
    ('user-909', 'seg-local',      'Local Discovery',              0.77),
    ('user-010', 'seg-tech',       'Technology Enthusiasts',       0.89),
    ('user-010', 'seg-early',      'Early Adopters',               0.94);

-- ── Offer Performance (sample creative metrics) ─────────────────────────────

INSERT INTO campaign_express.offer_performance
    (offer_id, campaign_id, creative_id, date, impressions, clicks, conversions, spend, ctr, cvr, cpm, roas)
VALUES
    ('offer-A1', 'camp-001', 'cre-A1-banner', today(), 7500,  225, 12, 375.00, 3.00, 5.33, 50.00, 3.00),
    ('offer-B1', 'camp-002', 'cre-B1-banner', today(), 5200,  156,  8, 260.00, 3.00, 5.13, 50.00, 3.00),
    ('offer-C1', 'camp-003', 'cre-C1-video',  today(), 17000, 680, 34, 850.00, 4.00, 5.00, 50.00, 4.00),
    ('offer-D1', 'camp-004', 'cre-D1-banner', today(), 3000,   60,  3, 150.00, 2.00, 5.00, 50.00, 2.00),
    ('offer-E1', 'camp-005', 'cre-E1-native', today(), 11500, 460, 23, 575.00, 4.00, 5.00, 50.00, 4.00),
    ('offer-F1', 'camp-006', 'cre-F1-banner', today(), 7000,  280, 14, 350.00, 4.00, 5.00, 50.00, 4.00),
    ('offer-G1', 'camp-007', 'cre-G1-rich',   today(), 21000, 1050, 63, 1050.00, 5.00, 6.00, 50.00, 5.00),
    ('offer-H1', 'camp-008', 'cre-H1-banner', today(), 2000,   40,  2, 100.00, 2.00, 5.00, 50.00, 1.60),
    ('offer-I1', 'camp-009', 'cre-I1-native', today(), 9000,  360, 18, 450.00, 4.00, 5.00, 50.00, 4.00),
    ('offer-J1', 'camp-010', 'cre-J1-video',  today(), 14000, 700, 42, 700.00, 5.00, 6.00, 50.00, 5.00);

-- ── Node Metrics (simulated cluster health) ─────────────────────────────────

INSERT INTO campaign_express.node_metrics
    (node_id, requests_per_sec, avg_latency_ms, p99_latency_ms, active_agents, cpu_percent, memory_mb, cache_hit_rate)
VALUES
    ('dev-01', 2500.0, 8.2, 45.0, 4, 35.0, 512.0, 0.85);
