# Nginx Configuration Guide

This guide covers nginx reverse proxy configuration, SSL/TLS setup, and troubleshooting.

## Overview

Nginx serves as:
- SSL/TLS termination
- Reverse proxy to bot service
- Rate limiting
- Security headers
- Request forwarding

## Architecture

```
Internet → Nginx (Port 443) → Bot (Port 8080)
           │
           └─ SSL/TLS termination
```

## Configuration Files

### Main Configuration

**File:** `nginx/nginx.conf`

Includes:
- Global settings (worker processes, logging)
- Rate limiting zones
- Upstream configuration for bot
- HTTP to HTTPS redirect
- HTTPS server block

### Site Configuration

**File:** `nginx/conf.d/default.conf`

Additional site-specific configurations (optional)

## SSL/TLS Setup

### Let's Encrypt with Certbot

#### Initial Certificate

```bash
# Stop nginx temporarily
docker-compose stop nginx

# Get certificate
sudo certbot certonly --standalone -d yourdomain.com -d www.yourdomain.com

# Copy certificates to Docker volume
sudo mkdir -p /var/lib/docker/volumes/kamachess_nginx_ssl/_data
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem /var/lib/docker/volumes/kamachess_nginx_ssl/_data/
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem /var/lib/docker/volumes/kamachess_nginx_ssl/_data/

# Start nginx
docker-compose start nginx
```

#### Auto-Renewal

```bash
# Test renewal
sudo certbot renew --dry-run

# Setup automatic renewal
sudo crontab -e

# Add this line (runs twice daily)
0 0,12 * * * certbot renew --quiet && docker cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem kamachess-nginx:/etc/nginx/ssl/ && docker cp /etc/letsencrypt/live/yourdomain.com/privkey.pem kamachess-nginx:/etc/nginx/ssl/ && docker-compose restart nginx
```

### Using Docker Certbot (Alternative)

Add certbot service to docker-compose.yml:

```yaml
certbot:
  image: certbot/certbot
  volumes:
    - nginx_ssl:/etc/letsencrypt
    - nginx_certbot:/var/www/certbot
  command: certonly --webroot --webroot-path=/var/www/certbot --email your@email.com --agree-tos --no-eff-email -d yourdomain.com
```

## Configuration Details

### Upstream Configuration

```nginx
upstream bot_backend {
    server bot:8080;
    keepalive 32;
}
```

- Points to bot service on port 8080
- Keepalive connections for performance

### Webhook Endpoint

```nginx
location /webhook {
    limit_req zone=webhook_limit burst=20 nodelay;
    limit_conn conn_limit 10;
    
    proxy_pass http://bot_backend;
    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header X-Telegram-Bot-Api-Secret-Token $http_x_telegram_bot_api_secret_token;
}
```

**Features:**
- Rate limiting: 10 requests/second (burst 20)
- Connection limiting: 10 concurrent connections
- Header forwarding for proper request handling
- Secret token header forwarding for webhook verification

### Security Headers

```nginx
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
```

**Benefits:**
- HSTS: Forces HTTPS connections
- X-Frame-Options: Prevents clickjacking
- X-Content-Type-Options: Prevents MIME sniffing
- X-XSS-Protection: XSS protection
- Referrer-Policy: Controls referrer information

### SSL/TLS Settings

```nginx
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:...;
ssl_prefer_server_ciphers off;
ssl_session_cache shared:SSL:10m;
ssl_session_timeout 10m;
ssl_session_tickets off;
```

**Configuration:**
- Only TLS 1.2 and 1.3 (secure)
- Modern cipher suites
- Session caching for performance
- Session tickets disabled (security)

## Rate Limiting

### Configuration

```nginx
limit_req_zone $binary_remote_addr zone=webhook_limit:10m rate=10r/s;
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;
```

**Settings:**
- **10r/s:** 10 requests per second
- **burst 20:** Allow 20 additional requests in burst
- **10 connections:** Maximum 10 concurrent connections per IP

### Adjusting Rate Limits

Edit `nginx/nginx.conf`:

```nginx
# Increase to 20 requests/second
limit_req_zone $binary_remote_addr zone=webhook_limit:10m rate=20r/s;

# In location block
limit_req zone=webhook_limit burst=40 nodelay;
```

Then restart nginx: `docker-compose restart nginx`

## Logging

### Access Logs

Location: `/var/lib/docker/volumes/kamachess_nginx_logs/_data/access.log`

Format: Combined log format with request details

### Error Logs

Location: `/var/lib/docker/volumes/kamachess_nginx_logs/_data/error.log`

### Viewing Logs

```bash
# Via docker-compose
docker-compose logs nginx

# Direct file access
sudo tail -f /var/lib/docker/volumes/kamachess_nginx_logs/_data/access.log
sudo tail -f /var/lib/docker/volumes/kamachess_nginx_logs/_data/error.log
```

## Troubleshooting

### Nginx Won't Start

```bash
# Check configuration syntax
docker-compose exec nginx nginx -t

# View nginx logs
docker-compose logs nginx

# Check SSL certificates exist
docker-compose exec nginx ls -la /etc/nginx/ssl/
```

### SSL Certificate Errors

```bash
# Verify certificates are valid
docker-compose exec nginx openssl x509 -in /etc/nginx/ssl/fullchain.pem -text -noout

# Check certificate expiration
docker-compose exec nginx openssl x509 -in /etc/nginx/ssl/fullchain.pem -noout -dates

# Regenerate if needed
sudo certbot renew --force-renewal
```

### 502 Bad Gateway

**Causes:**
- Bot service not running
- Bot service not healthy
- Network connectivity issues

**Solutions:**
```bash
# Check bot is running
docker-compose ps bot

# Check bot health
curl http://localhost:8080/health

# Check network
docker network inspect kamachess-network

# Restart services
docker-compose restart bot nginx
```

### Rate Limiting Issues

If legitimate requests are being rate-limited:

```bash
# Check current limits
grep -A 5 "limit_req" nginx/nginx.conf

# Increase limits (edit nginx.conf)
# Then restart: docker-compose restart nginx
```

### Headers Not Forwarded

Verify proxy headers in nginx.conf:

```nginx
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $scheme;
```

## Performance Tuning

### Worker Processes

```nginx
worker_processes auto;  # Uses number of CPU cores
```

### Keepalive Connections

```nginx
keepalive_timeout 65;
keepalive_requests 100;
```

### Connection Pooling

```nginx
upstream bot_backend {
    server bot:8080;
    keepalive 32;  # Maintain 32 idle connections
}
```

## Customization

### Adding Custom Locations

Edit `nginx/conf.d/default.conf`:

```nginx
# Custom endpoint
location /custom {
    proxy_pass http://bot_backend;
    # ... other settings ...
}
```

### Multiple Domains

Add additional server blocks in `nginx.conf`:

```nginx
server {
    listen 443 ssl http2;
    server_name another-domain.com;
    # ... SSL and proxy config ...
}
```

## Security Hardening

1. **Disable server tokens:**
```nginx
server_tokens off;
```

2. **Limit allowed methods:**
```nginx
location /webhook {
    limit_except POST {
        deny all;
    }
}
```

3. **Restrict access by IP (optional):**
```nginx
location /webhook {
    allow 1.2.3.4;  # Telegram IP ranges
    deny all;
}
```

## Next Steps

- SSL setup: Configure Let's Encrypt certificates
- Monitoring: Integrate nginx metrics with Prometheus (optional)
- Log analysis: Setup ELK stack for log analysis (optional)
