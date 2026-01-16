# Monitoring Guide

This guide covers setting up and using Prometheus and Grafana for monitoring the Kamachess bot.

## Overview

The monitoring stack consists of:
- **Prometheus** - Metrics collection and storage
- **Grafana** - Metrics visualization and dashboards
- **node-exporter** - System metrics (CPU, memory, disk, network)

## Architecture

```
┌─────────────┐
│  Grafana    │ ← Visualization (Port 3000)
│ Dashboards  │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Prometheus  │ ← Metrics Collection (Port 9090)
│   Scraper   │
└──────┬──────┘
       │
       ├─────────────┐
       ▼             ▼
┌──────────┐   ┌─────────────┐
│   Bot    │   │node-exporter│
│ /metrics │   │  (System)   │
└──────────┘   └─────────────┘
```

## Setup

### Automatic Setup (Docker Compose)

Monitoring services are included in `docker-compose.yml`:

```bash
# Start all services including monitoring
docker-compose up -d

# Check monitoring services
docker-compose ps prometheus grafana node-exporter
```

### Manual Configuration

#### Prometheus

Configuration file: `monitoring/prometheus/prometheus.yml`

**Scrape targets:**
- Bot application: `bot:8080/metrics` (if metrics endpoint added)
- node-exporter: `node-exporter:9100/metrics`
- Prometheus itself: `localhost:9090`

**Retention:** 30 days (configurable)

#### Grafana

**Default credentials:**
- Username: `admin`
- Password: Set via `GRAFANA_ADMIN_PASSWORD` environment variable

**Pre-configured:**
- Prometheus datasource (auto-provisioned)
- Default dashboard (if configured)

## Accessing Monitoring

### Prometheus

Access Prometheus UI:
- Local: `http://localhost:9090`
- Remote: `http://your-server-ip:9090` (if exposed)

**Useful queries:**
```promql
# Container up status
up{job="kamachess-bot"}

# CPU usage
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Memory usage
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100

# Disk usage
(node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100
```

### Grafana

Access Grafana UI:
- Local: `http://localhost:3000`
- Remote: `http://your-server-ip:3000` (if exposed)

**Default dashboard:**
- Name: Kamachess Bot Dashboard
- Path: `/d/kamachess/bot-dashboard`

## Metrics Available

### System Metrics (node-exporter)

- **CPU:** Usage percentage, per-core usage
- **Memory:** Total, available, used, cache, buffers
- **Disk:** Usage, I/O operations, read/write rates
- **Network:** Bytes sent/received, packet rates
- **Load Average:** System load

### Application Metrics (if implemented)

- **Webhook Requests:** Request count, rate, latency
- **Error Rate:** HTTP errors, application errors
- **Active Games:** Current active game count
- **Database Connections:** Pool status, active connections
- **Response Times:** P50, P95, P99 latencies

### Container Metrics

- **Container Status:** Up/down status
- **Container Health:** Health check results
- **Resource Usage:** CPU, memory per container

## Grafana Dashboards

### Default Dashboard

Location: `monitoring/grafana/dashboards/bot-dashboard.json`

**Panels:**
1. System CPU Usage (graph)
2. System Memory Usage (graph)
3. Bot Container Status (stat)
4. Nginx Status (stat)
5. PostgreSQL Status (stat)
6. Disk Usage (graph)

### Creating Custom Dashboards

1. Login to Grafana
2. Create → Dashboard
3. Add panels with Prometheus queries
4. Save dashboard
5. Export JSON and add to `monitoring/grafana/dashboards/`

### Importing Dashboards

```bash
# Copy dashboard JSON to Grafana dashboards directory
cp my-dashboard.json monitoring/grafana/dashboards/

# Restart Grafana to load new dashboard
docker-compose restart grafana
```

## Alerting (Optional)

### Prometheus Alert Rules

Create `monitoring/alertrules.yml`:

```yaml
groups:
  - name: kamachess_alerts
    rules:
      - alert: BotDown
        expr: up{job="kamachess-bot"} == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Bot service is down"

      - alert: HighCPU
        expr: 100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "CPU usage is above 80%"
```

### Grafana Alerts

1. Create alert in Grafana dashboard panel
2. Configure alert conditions
3. Set notification channels (email, Slack, etc.)
4. Test alerts

## Maintenance

### Viewing Metrics

```bash
# Prometheus metrics endpoint
curl http://localhost:9090/api/v1/query?query=up

# Node exporter metrics
curl http://localhost:9100/metrics

# Bot metrics (if implemented)
curl http://localhost:8080/metrics
```

### Data Retention

**Prometheus retention:** 30 days (configurable in `prometheus.yml`)

```yaml
global:
  storage_tsdb_retention_time: 30d
```

**Adjust retention:**
1. Edit `monitoring/prometheus/prometheus.yml`
2. Change `storage_tsdb_retention_time`
3. Restart Prometheus: `docker-compose restart prometheus`

### Backing Up Data

```bash
# Backup Prometheus data
docker-compose exec prometheus tar czf /tmp/prometheus-backup.tar.gz /prometheus

# Backup Grafana data
docker-compose exec grafana tar czf /tmp/grafana-backup.tar.gz /var/lib/grafana
```

### Cleaning Old Data

```bash
# Clean Prometheus old data (if retention exceeded)
docker-compose exec prometheus promtool tsdb clean /prometheus

# Or restart with new retention period
```

## Troubleshooting

### Prometheus Not Scraping

```bash
# Check Prometheus targets
# Visit: http://localhost:9090/targets

# Check Prometheus logs
docker-compose logs prometheus

# Verify scrape configuration
docker-compose exec prometheus cat /etc/prometheus/prometheus.yml
```

### Grafana Can't Connect to Prometheus

1. Verify Prometheus is running: `docker-compose ps prometheus`
2. Check datasource URL: `http://prometheus:9090`
3. Verify network: Both services on `kamachess-network`
4. Check Grafana logs: `docker-compose logs grafana`

### Missing Metrics

- Verify scrape targets are configured
- Check targets are up in Prometheus UI
- Verify metrics endpoint is accessible
- Check scrape interval (default: 15s)

### High Disk Usage

```bash
# Check Prometheus data size
docker-compose exec prometheus du -sh /prometheus

# Reduce retention period
# Edit monitoring/prometheus/prometheus.yml
# storage_tsdb_retention_time: 15d  # Reduce from 30d
```

## Performance Tuning

### Prometheus

- **Scrape interval:** 15s (balance between accuracy and load)
- **Retention:** 30 days (adjust based on disk space)
- **Memory:** Monitor Prometheus memory usage

### Grafana

- **Refresh interval:** 30s (default dashboard)
- **Dashboard complexity:** Limit panels per dashboard
- **Data source timeout:** Adjust if queries timeout

## Security

1. **Change default passwords:** Set `GRAFANA_ADMIN_PASSWORD`
2. **Limit access:** Use nginx reverse proxy with authentication
3. **HTTPS:** Enable SSL/TLS for Grafana (via nginx)
4. **Firewall:** Only expose necessary ports

## Next Steps

- Setup alerts: Configure Prometheus alertmanager
- Custom dashboards: Create application-specific dashboards
- Metrics export: Add Prometheus metrics endpoint to bot
- Log aggregation: Integrate with ELK stack (optional)
