# Deployment Guide

This guide covers deploying the Kamachess Telegram bot to production using Docker Compose on EC2.

## Prerequisites

- EC2 instance (Ubuntu 22.04 LTS recommended)
- Domain name pointing to EC2 instance
- Docker and Docker Compose installed
- SSH access to EC2 instance
- Security group configured (ports 22, 80, 443)

## Quick Start

### 1. EC2 Instance Setup

```bash
# Connect to EC2 instance
ssh -i your-key.pem ubuntu@your-ec2-ip

# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker ubuntu

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Log out and back in for group changes
exit
```

### 2. Clone Repository

```bash
# On EC2 instance
cd /opt
sudo git clone https://github.com/yourusername/kamachess.git
sudo chown -R ubuntu:ubuntu kamachess
cd kamachess
```

### 3. Configure Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit with your values
nano .env
```

Required variables:
- `TELEGRAM_BOT_TOKEN` - Your bot token from @BotFather
- `TELEGRAM_BOT_USERNAME` - Your bot username
- `WEBHOOK_URL` - Public URL (e.g., https://yourdomain.com/webhook)

### 4. Setup SSL Certificates

```bash
# Install certbot
sudo apt install -y certbot

# Get certificate (replace with your domain)
sudo certbot certonly --standalone -d yourdomain.com

# Copy certificates to nginx volume location
sudo mkdir -p /var/lib/docker/volumes/kamachess_nginx_ssl/_data
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem /var/lib/docker/volumes/kamachess_nginx_ssl/_data/
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem /var/lib/docker/volumes/kamachess_nginx_ssl/_data/

# Setup auto-renewal
sudo certbot renew --dry-run
```

### 5. Start Services

```bash
# Build and start all services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f bot
```

### 6. Verify Deployment

```bash
# Check bot is running
curl http://localhost:8080/health

# Check nginx is proxying
curl http://localhost/health

# Check webhook is set
# The bot automatically sets the webhook on startup
```

## Detailed Configuration

### Environment Variables

See [.env.example](../.env.example) for all available environment variables.

### Docker Compose Services

- **postgres** - PostgreSQL database
- **bot** - Rust application (webhook server)
- **nginx** - Reverse proxy with SSL
- **prometheus** - Metrics collection
- **grafana** - Metrics visualization
- **node-exporter** - System metrics

### Networking

All services run on the `kamachess-network` Docker network:
- Internal communication only
- External access via nginx (ports 80, 443)
- Bot exposed internally on port 8080

### Volumes

- `postgres_data` - Database persistence
- `bot_logs` - Application logs
- `nginx_ssl` - SSL certificates
- `nginx_certbot` - Let's Encrypt challenge files
- `prometheus_data` - Metrics data
- `grafana_data` - Grafana dashboards and data

## Maintenance

### Viewing Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f bot
docker-compose logs -f nginx
```

### Updating Application

```bash
# Pull latest code
git pull

# Rebuild and restart bot
docker-compose up -d --build bot

# Or restart all services
docker-compose up -d --build
```

### Database Backups

```bash
# Backup PostgreSQL database
docker-compose exec postgres pg_dump -U kamachess kamachess > backup_$(date +%Y%m%d).sql

# Restore from backup
docker-compose exec -T postgres psql -U kamachess kamachess < backup_20260116.sql
```

### SSL Certificate Renewal

```bash
# Certificates auto-renew, but you can manually renew:
sudo certbot renew

# Copy renewed certificates to Docker volume
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem /var/lib/docker/volumes/kamachess_nginx_ssl/_data/
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem /var/lib/docker/volumes/kamachess_nginx_ssl/_data/

# Reload nginx
docker-compose restart nginx
```

## Troubleshooting

### Service Won't Start

```bash
# Check service status
docker-compose ps

# Check logs
docker-compose logs bot

# Check environment variables
docker-compose config
```

### Webhook Not Working

1. Verify `WEBHOOK_URL` is set correctly
2. Check nginx is running: `docker-compose ps nginx`
3. Verify SSL certificates are valid
4. Check bot logs: `docker-compose logs bot`
5. Test webhook endpoint: `curl -X POST https://yourdomain.com/webhook`

### Database Connection Issues

```bash
# Check PostgreSQL is healthy
docker-compose ps postgres

# Check database logs
docker-compose logs postgres

# Verify connection string in .env
```

### Nginx SSL Errors

```bash
# Check certificate files exist
ls -la /var/lib/docker/volumes/kamachess_nginx_ssl/_data/

# Check nginx configuration
docker-compose exec nginx nginx -t

# View nginx logs
docker-compose logs nginx
```

## Security Best Practices

1. **Secrets Management**
   - Never commit `.env` file
   - Use Docker secrets or AWS Secrets Manager for production
   - Rotate credentials regularly

2. **Network Security**
   - Only expose nginx ports (80, 443) externally
   - Use security groups to restrict access
   - Enable SSH key authentication only

3. **SSL/TLS**
   - Use Let's Encrypt certificates
   - Enable HSTS headers
   - Regular certificate renewal

4. **Updates**
   - Keep system packages updated
   - Regularly update Docker images
   - Monitor security advisories

## Monitoring

See [MONITORING.md](MONITORING.md) for detailed monitoring setup and usage.

## Next Steps

- Setup CI/CD pipeline: [CI_CD.md](CI_CD.md)
- Configure monitoring: [MONITORING.md](MONITORING.md)
- Customize nginx: [NGINX.md](NGINX.md)
