# Kamachess Documentation

Welcome to the Kamachess Telegram Bot documentation. This directory contains comprehensive guides for deployment, development, CI/CD, monitoring, and infrastructure.

## Documentation Index

### Getting Started
- [Development Guide](DEVELOPMENT.md) - Local development setup and workflow
- [Deployment Guide](DEPLOYMENT.md) - Production deployment on EC2

### Infrastructure
- [CI/CD Guide](CI_CD.md) - Jenkins pipeline setup and usage
- [Monitoring Guide](MONITORING.md) - Prometheus and Grafana setup
- [Nginx Guide](NGINX.md) - Nginx configuration and SSL setup

## Quick Links

### For Developers
- Setup local environment: [DEVELOPMENT.md](DEVELOPMENT.md)
- Run tests: `cargo test`
- Build Docker image: `docker build -t kamachess .`

### For DevOps
- Deploy to EC2: [DEPLOYMENT.md](DEPLOYMENT.md)
- Setup Jenkins: [CI_CD.md](CI_CD.md)
- Configure monitoring: [MONITORING.md](MONITORING.md)
- Setup nginx/SSL: [NGINX.md](NGINX.md)

### For System Administrators
- Production deployment: [DEPLOYMENT.md](DEPLOYMENT.md)
- Monitoring setup: [MONITORING.md](MONITORING.md)
- SSL certificate management: [NGINX.md](NGINX.md)

## Architecture Overview

```
┌─────────────┐
│   Telegram  │
│     Bot     │
└──────┬──────┘
       │ HTTPS
       ▼
┌─────────────┐
│    Nginx    │ ← SSL/TLS termination
│  (Port 443) │
└──────┬──────┘
       │ HTTP
       ▼
┌─────────────┐
│     Bot     │ ← Rust application
│  (Port 8080)│
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  PostgreSQL │ ← Database
└─────────────┘

Monitoring Stack:
┌─────────────┐     ┌─────────────┐
│ Prometheus  │◄────│ Grafana     │
│  (Metrics)  │     │(Dashboards) │
└──────┬──────┘     └─────────────┘
       │
       ▼
┌─────────────┐
│node-exporter│
│ (System)    │
└─────────────┘
```

## Support

For issues, questions, or contributions, please see the main [README.md](../README.md) file.
