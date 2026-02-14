#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Redis Seed Data
# Populates Redis with test campaigns, user profiles, and offer cache.
# Usage: ./scripts/seed/redis-seed.sh [REDIS_URL]
# =============================================================================
set -euo pipefail

REDIS_URL="${1:-redis://localhost:6379}"
HOST=$(echo "$REDIS_URL" | sed 's|redis://||' | cut -d: -f1)
PORT=$(echo "$REDIS_URL" | sed 's|redis://||' | cut -d: -f2)
PORT="${PORT:-6379}"

echo "Seeding Redis at $HOST:$PORT"

r() { redis-cli -h "$HOST" -p "$PORT" "$@" >/dev/null; }

# ── Campaigns (10 active) ───────────────────────────────────────────────────

echo "  Seeding campaigns..."
r SET "campaign:camp-001" '{"id":"camp-001","name":"TechDeals Summer","status":"active","budget_daily":500.00,"budget_total":15000.00,"bid_strategy":"maximize_clicks","targeting":{"geo":["US"],"segments":["seg-tech","seg-premium"],"devices":["mobile","desktop"],"os":["iOS","Android"]},"creatives":["cre-A1-banner"],"start_date":"2026-01-01","end_date":"2026-03-31"}'
r SET "campaign:camp-002" '{"id":"camp-002","name":"GameZone Retarget","status":"active","budget_daily":350.00,"budget_total":10000.00,"bid_strategy":"target_cpa","targeting":{"geo":["US","DE","UK"],"segments":["seg-gamers","seg-youth"],"devices":["mobile"],"os":["iOS"]},"creatives":["cre-B1-banner"],"start_date":"2026-01-15","end_date":"2026-04-15"}'
r SET "campaign:camp-003" '{"id":"camp-003","name":"BreakingNews Premium","status":"active","budget_daily":1200.00,"budget_total":36000.00,"bid_strategy":"maximize_impressions","targeting":{"geo":["US"],"segments":["seg-news","seg-commute"],"devices":["mobile","desktop","tablet"],"os":["Android","iOS","Windows"]},"creatives":["cre-C1-video"],"start_date":"2026-01-01","end_date":"2026-06-30"}'
r SET "campaign:camp-004" '{"id":"camp-004","name":"SmallBiz Starter","status":"active","budget_daily":200.00,"budget_total":6000.00,"bid_strategy":"minimize_cost","targeting":{"geo":["US"],"segments":["seg-local"],"devices":["mobile"],"os":["iOS","Android"]},"creatives":["cre-D1-banner"],"start_date":"2026-02-01","end_date":"2026-04-30"}'
r SET "campaign:camp-005" '{"id":"camp-005","name":"ShopSmart Q1","status":"active","budget_daily":800.00,"budget_total":24000.00,"bid_strategy":"target_roas","targeting":{"geo":["US","UK","CA"],"segments":["seg-shoppers","seg-premium"],"devices":["desktop","mobile"],"os":["Windows","macOS","iOS"]},"creatives":["cre-E1-native"],"start_date":"2026-01-01","end_date":"2026-03-31"}'
r SET "campaign:camp-006" '{"id":"camp-006","name":"FitLife Spring","status":"active","budget_daily":450.00,"budget_total":13500.00,"bid_strategy":"maximize_clicks","targeting":{"geo":["US","CA"],"segments":["seg-fitness","seg-mobile"],"devices":["mobile"],"os":["iOS","Android"]},"creatives":["cre-F1-banner"],"start_date":"2026-02-01","end_date":"2026-05-31"}'
r SET "campaign:camp-007" '{"id":"camp-007","name":"TravelMax Global","status":"active","budget_daily":1500.00,"budget_total":45000.00,"bid_strategy":"maximize_conversions","targeting":{"geo":["US","UK","DE","FR","JP"],"segments":["seg-travel","seg-premium","seg-high-value"],"devices":["desktop","mobile","tablet"],"os":["macOS","Windows","iOS","Android"]},"creatives":["cre-G1-rich"],"start_date":"2026-01-01","end_date":"2026-12-31"}'
r SET "campaign:camp-008" '{"id":"camp-008","name":"Local Eats Pilot","status":"active","budget_daily":150.00,"budget_total":4500.00,"bid_strategy":"minimize_cost","targeting":{"geo":["US"],"segments":["seg-food","seg-local"],"devices":["mobile"],"os":["iOS","Android"]},"creatives":["cre-H1-banner"],"start_date":"2026-02-01","end_date":"2026-04-30"}'
r SET "campaign:camp-009" '{"id":"camp-009","name":"AutoDrive Intenders","status":"active","budget_daily":600.00,"budget_total":18000.00,"bid_strategy":"target_cpa","targeting":{"geo":["US"],"segments":["seg-auto","seg-suburban"],"devices":["desktop","mobile"],"os":["Windows","macOS","iOS","Android"]},"creatives":["cre-I1-native"],"start_date":"2026-01-15","end_date":"2026-06-30"}'
r SET "campaign:camp-010" '{"id":"camp-010","name":"FinServ Premium","status":"active","budget_daily":1000.00,"budget_total":30000.00,"bid_strategy":"maximize_conversions","targeting":{"geo":["US","UK"],"segments":["seg-finance","seg-high-value"],"devices":["desktop"],"os":["Windows","macOS"]},"creatives":["cre-J1-video"],"start_date":"2026-01-01","end_date":"2026-12-31"}'

# Campaign index
r SADD "campaigns:active" camp-001 camp-002 camp-003 camp-004 camp-005 camp-006 camp-007 camp-008 camp-009 camp-010

# ── User Profiles (pre-cached for fast lookup) ──────────────────────────────

echo "  Seeding user profiles..."
r SET "user:user-101" '{"id":"user-101","segments":["seg-tech","seg-premium","seg-mobile"],"geo":"US:CA:San Francisco","device":"iOS","last_seen":"2026-02-14","bid_count":42,"click_count":3}'
r SET "user:user-202" '{"id":"user-202","segments":["seg-news","seg-commute"],"geo":"US:NY:New York","device":"Android","last_seen":"2026-02-14","bid_count":28,"click_count":2}'
r SET "user:user-303" '{"id":"user-303","segments":["seg-shoppers","seg-premium","seg-intl"],"geo":"UK::London","device":"Windows","last_seen":"2026-02-13","bid_count":55,"click_count":5}'
r SET "user:user-404" '{"id":"user-404","segments":["seg-gamers","seg-youth"],"geo":"DE::Berlin","device":"iOS","last_seen":"2026-02-14","bid_count":18,"click_count":4}'
r SET "user:user-505" '{"id":"user-505","segments":["seg-travel","seg-premium","seg-high-value"],"geo":"US:FL:Miami","device":"macOS","last_seen":"2026-02-14","bid_count":67,"click_count":8}'
r SET "user:user-606" '{"id":"user-606","segments":["seg-fitness","seg-mobile"],"geo":"CA:ON:Toronto","device":"iOS","last_seen":"2026-02-13","bid_count":33,"click_count":2}'
r SET "user:user-707" '{"id":"user-707","segments":["seg-auto","seg-suburban"],"geo":"US:TX:Austin","device":"Android","last_seen":"2026-02-12","bid_count":15,"click_count":1}'
r SET "user:user-808" '{"id":"user-808","segments":["seg-finance","seg-high-value"],"geo":"US:MA:Boston","device":"Windows","last_seen":"2026-02-14","bid_count":80,"click_count":6}'
r SET "user:user-909" '{"id":"user-909","segments":["seg-food","seg-local"],"geo":"US:IL:Chicago","device":"iOS","last_seen":"2026-02-13","bid_count":22,"click_count":3}'
r SET "user:user-010" '{"id":"user-010","segments":["seg-tech","seg-early"],"geo":"US:WA:Seattle","device":"macOS","last_seen":"2026-02-14","bid_count":95,"click_count":12}'

# ── Offer Cache (pre-warmed) ────────────────────────────────────────────────

echo "  Seeding offer cache..."
r SET "offer:offer-A1" '{"id":"offer-A1","campaign_id":"camp-001","creative_id":"cre-A1-banner","ad_markup":"<div class=\"ce-ad\"><img src=\"/ads/tech-deals-300x250.jpg\" width=\"300\" height=\"250\"/></div>","w":300,"h":250,"bid_price":1.50,"floor":0.50}'
r SET "offer:offer-B1" '{"id":"offer-B1","campaign_id":"camp-002","creative_id":"cre-B1-banner","ad_markup":"<div class=\"ce-ad\"><img src=\"/ads/gamezone-320x50.jpg\" width=\"320\" height=\"50\"/></div>","w":320,"h":50,"bid_price":1.25,"floor":0.30}'
r SET "offer:offer-C1" '{"id":"offer-C1","campaign_id":"camp-003","creative_id":"cre-C1-video","ad_markup":"<div class=\"ce-ad\"><video src=\"/ads/news-premium.mp4\" width=\"728\" height=\"90\"></video></div>","w":728,"h":90,"bid_price":2.10,"floor":0.80}'
r SET "offer:offer-D1" '{"id":"offer-D1","campaign_id":"camp-004","creative_id":"cre-D1-banner","ad_markup":"<div class=\"ce-ad\"><img src=\"/ads/smallbiz-300x250.jpg\" width=\"300\" height=\"250\"/></div>","w":300,"h":250,"bid_price":0.90,"floor":0.20}'
r SET "offer:offer-E1" '{"id":"offer-E1","campaign_id":"camp-005","creative_id":"cre-E1-native","ad_markup":"<div class=\"ce-ad ce-native\"><h3>Smart Shopping Picks</h3><p>Curated deals for you</p></div>","w":160,"h":600,"bid_price":1.80,"floor":0.60}'
r SET "offer:offer-F1" '{"id":"offer-F1","campaign_id":"camp-006","creative_id":"cre-F1-banner","ad_markup":"<div class=\"ce-ad\"><img src=\"/ads/fitlife-300x250.jpg\" width=\"300\" height=\"250\"/></div>","w":300,"h":250,"bid_price":1.40,"floor":0.45}'
r SET "offer:offer-G1" '{"id":"offer-G1","campaign_id":"camp-007","creative_id":"cre-G1-rich","ad_markup":"<div class=\"ce-ad ce-rich\"><iframe src=\"/ads/travel-rich.html\" width=\"300\" height=\"250\"></iframe></div>","w":300,"h":250,"bid_price":2.50,"floor":1.00}'
r SET "offer:offer-H1" '{"id":"offer-H1","campaign_id":"camp-008","creative_id":"cre-H1-banner","ad_markup":"<div class=\"ce-ad\"><img src=\"/ads/local-eats-300x250.jpg\" width=\"300\" height=\"250\"/></div>","w":300,"h":250,"bid_price":0.75,"floor":0.15}'
r SET "offer:offer-I1" '{"id":"offer-I1","campaign_id":"camp-009","creative_id":"cre-I1-native","ad_markup":"<div class=\"ce-ad ce-native\"><h3>Drive Your Dream</h3><p>Auto offers near you</p></div>","w":300,"h":250,"bid_price":1.65,"floor":0.55}'
r SET "offer:offer-J1" '{"id":"offer-J1","campaign_id":"camp-010","creative_id":"cre-J1-video","ad_markup":"<div class=\"ce-ad\"><video src=\"/ads/finserv-premium.mp4\" width=\"300\" height=\"250\"></video></div>","w":300,"h":250,"bid_price":2.20,"floor":0.90}'

# ── Segment → Campaign Mapping (for fast matching) ──────────────────────────

echo "  Seeding segment indexes..."
r SADD "segment:seg-tech"       camp-001 camp-010
r SADD "segment:seg-premium"    camp-001 camp-005 camp-007
r SADD "segment:seg-mobile"     camp-001 camp-006
r SADD "segment:seg-news"       camp-003
r SADD "segment:seg-commute"    camp-003
r SADD "segment:seg-gamers"     camp-002
r SADD "segment:seg-youth"      camp-002
r SADD "segment:seg-shoppers"   camp-005
r SADD "segment:seg-intl"       camp-005
r SADD "segment:seg-travel"     camp-007
r SADD "segment:seg-high-value" camp-007 camp-010
r SADD "segment:seg-fitness"    camp-006
r SADD "segment:seg-food"       camp-008
r SADD "segment:seg-local"      camp-004 camp-008
r SADD "segment:seg-auto"       camp-009
r SADD "segment:seg-suburban"   camp-009
r SADD "segment:seg-finance"    camp-010
r SADD "segment:seg-early"      camp-010

echo "  Done — $(redis-cli -h "$HOST" -p "$PORT" DBSIZE | awk '{print $2}') keys in Redis"
