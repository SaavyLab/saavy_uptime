# Cost Modeling Guidelines

Saavy Uptime includes an **experimental cost estimator** 
to help users understand how their configuration choices 
affect Cloudflare resource consumption.

## What It Does
- Estimates resource usage (D1/AE writes, Worker requests)
- Applies publicly documented Cloudflare pricing
- Shows directional cost impact of tuning decisions

## What It Doesn't Do
- Replace Cloudflare's billing dashboard
- Account for account-specific pricing
- Guarantee accuracy of estimates

## Data Sources
Pricing data from:
- https://developers.cloudflare.com/workers/platform/pricing/
- https://developers.cloudflare.com/d1/platform/pricing/
- https://developers.cloudflare.com/analytics/analytics-engine/pricing/