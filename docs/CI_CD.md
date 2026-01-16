# CI/CD Pipeline Guide

This guide covers setting up and using the Jenkins CI/CD pipeline for automated builds, tests, and deployments.

## Overview

The CI/CD pipeline automates:
- Code checkout
- Building Rust application
- Running tests
- Linting (Clippy, format checks)
- Building Docker images
- Deployment to production

## Jenkins Deployment Architecture

### Deployment Modes

**Local Deployment (Same EC2 as Jenkins):**
- Jenkins and bot application run on the same EC2 instance
- No SSH needed for deployment
- Faster deployments
- Set `DEPLOY_HOST` to empty string or `localhost` in Jenkins credentials
- Recommended for single-server setups

**Remote Deployment (Different EC2):**
- Jenkins runs on one EC2 instance, bot on another
- Uses SSH for deployment
- Better separation of concerns
- Set `DEPLOY_HOST` to the remote EC2 IP/hostname in Jenkins credentials
- Recommended for production environments with multiple servers

## Jenkins Setup

### Option 1: Docker-based Jenkins (Recommended)

```bash
# Create Jenkins directory
mkdir -p jenkins

# Start Jenkins container
docker run -d \
  --name jenkins \
  -p 8080:8080 \
  -p 50000:50000 \
  -v jenkins_home:/var/jenkins_home \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v $(pwd):/workspace \
  jenkins/jenkins:lts

# Get initial admin password
docker exec jenkins cat /var/jenkins_home/secrets/initialAdminPassword
```

Access Jenkins at `http://localhost:8080`

### Option 2: Install Jenkins on EC2

```bash
# Add Jenkins repository
curl -fsSL https://pkg.jenkins.io/debian-stable/jenkins.io-2023.key | sudo tee \
  /usr/share/keyrings/jenkins-keyring.asc > /dev/null
echo deb [signed-by=/usr/share/keyrings/jenkins-keyring.asc] \
  https://pkg.jenkins.io/debian-stable binary/ | sudo tee \
  /etc/apt/sources.list.d/jenkins.list > /dev/null

# Install Jenkins
sudo apt-get update
sudo apt-get install -y jenkins

# Start Jenkins
sudo systemctl start jenkins
sudo systemctl enable jenkins

# Get initial password
sudo cat /var/lib/jenkins/secrets/initialAdminPassword
```

## Jenkins Configuration

### 1. Install Required Plugins

In Jenkins UI (Manage Jenkins → Plugins), install:
- Docker Pipeline
- SSH Pipeline Steps
- GitHub Integration (if using GitHub)
- AnsiColor (for better log display)

### 2. Configure Credentials

**SSH Credentials for Deployment:**
- Go to Manage Jenkins → Credentials
- Add new SSH credentials:
  - ID: `deploy-ssh-key`
  - Username: `ubuntu`
  - Private Key: Your EC2 SSH private key

**String Credentials:**
- `deploy-host`: EC2 hostname or IP (leave empty or `localhost` for local deployment)
- `deploy-user`: SSH username (usually `ubuntu`, not needed for local deployment)
- `deploy-path`: Deployment path (e.g., `/opt/kamachess`)

**For Local Deployment (Jenkins on same EC2):**
- Set `deploy-host` to empty string or `localhost`
- `deploy-path` should point to where your docker-compose.yml is located

**For Remote Deployment:**
- Set `deploy-host` to your EC2 IP or hostname (e.g., `13.49.238.213`)
- Set `deploy-user` to your SSH username (usually `ubuntu`)
- Configure `deploy-ssh-key` with your EC2 SSH private key

**Docker Registry (Optional):**
- `docker-registry-creds`: Docker Hub or registry credentials

### 3. Create Jenkins Pipeline Job

1. New Item → Pipeline
2. Name: `kamachess-pipeline`
3. Pipeline → Definition: Pipeline script from SCM
4. SCM: Git
5. Repository URL: Your repository URL
6. Script Path: `Jenkinsfile`
7. Save

## Pipeline Stages

### 1. Checkout
- Clones repository from Git
- Records commit hash

### 2. Build
- Builds Rust application using Docker
- Creates Docker image with build number tag

### 3. Test
- Runs all unit and integration tests
- Uses Docker-based Rust toolchain (no local Rust needed)

### 4. Lint
- Runs Clippy linter
- Checks code formatting with `cargo fmt`

### 5. Build Docker Image
- Creates production-ready Docker image
- Tags with build number and `latest`

### 6. Push to Registry (Optional)
- Pushes image to Docker registry if configured
- Requires `DOCKER_REGISTRY` environment variable

### 7. Deploy
- Only runs on `main` branch
- Supports two deployment modes:
  - **Local Deployment**: If `DEPLOY_HOST` is empty or `localhost`, deploys directly on the same EC2 instance as Jenkins (no SSH needed)
  - **Remote Deployment**: If `DEPLOY_HOST` is set, deploys to remote EC2 instance using SSH
- Restarts services with docker-compose

## Pipeline Configuration

### Jenkinsfile

The `Jenkinsfile` uses declarative pipeline syntax:

```groovy
pipeline {
    agent any
    environment {
        IMAGE_NAME = 'kamachess'
        IMAGE_TAG = "${env.BUILD_NUMBER}"
    }
    stages {
        // ... stages ...
    }
}
```

### Environment Variables

Set in Jenkins → Manage Jenkins → Configure System → Global properties:

- `DOCKER_REGISTRY` (optional): Docker registry URL
- `DEPLOY_HOST`: EC2 hostname (leave empty or set to `localhost` for local deployment on same EC2 as Jenkins, or use credentials)
- `DEPLOY_USER`: SSH username (or use credentials)
- `DEPLOY_PATH`: Deployment path (or use credentials)

### Credentials

Configure in Jenkins → Credentials:

- `deploy-ssh-key`: SSH private key for EC2 access
- `deploy-host`: EC2 hostname (string credential)
- `deploy-user`: SSH username (string credential)
- `deploy-path`: Deployment path (string credential)
- `docker-registry-creds`: Docker registry credentials (optional)

## Deployment Scripts

### jenkins-build.sh

Build script executed by Jenkins:
- Builds Docker image
- Optionally runs tests
- Tags image appropriately

Usage:
```bash
./scripts/jenkins-build.sh [IMAGE_NAME] [IMAGE_TAG] [BUILD_DIR]
```

### jenkins-deploy.sh

Deployment script executed by Jenkins:
- Exports Docker image to tar file
- Copies to EC2 via SCP
- Loads image on remote server
- Restarts services with docker-compose
- Verifies deployment

Usage:
```bash
./scripts/jenkins-deploy.sh <host> <user> <deploy_path> <image_tag>
```

## Manual Pipeline Execution

### Run Pipeline Manually

1. Go to Jenkins dashboard
2. Click on `kamachess-pipeline`
3. Click "Build Now"

### View Pipeline Progress

- Blue Ocean view: Click "Open Blue Ocean"
- Classic view: Click on build number
- Console Output: View real-time logs

### Build Artifacts

- Docker images stored in Jenkins workspace
- Artifacts archived after successful builds
- Access via Build → Artifacts

## Troubleshooting

### Build Fails at Build Stage

- Check Docker is installed and running
- Verify Docker socket is accessible
- Check disk space: `df -h`

### Tests Fail

- Check test logs in console output
- Verify test environment is correct
- Run tests locally: `cargo test`

### Deployment Fails

- Verify SSH credentials are correct
- Check EC2 security group allows SSH
- Verify deployment path exists
- Check docker-compose is installed on EC2

### Permission Issues

```bash
# Ensure Jenkins user can use Docker
sudo usermod -aG docker jenkins
sudo systemctl restart jenkins

# Ensure Jenkins can access workspace
sudo chown -R jenkins:jenkins /path/to/workspace
```

## Best Practices

1. **Branch Strategy**
   - `main` branch: Auto-deploys to production
   - Feature branches: Only build/test, no deployment
   - Configure branch restrictions in Jenkins

2. **Build Artifacts**
   - Keep last 10 builds (configured in Jenkinsfile)
   - Clean up old Docker images
   - Archive important builds

3. **Notifications**
   - Configure email notifications for failures
   - Integrate with Slack/Teams (optional)
   - Set up alerts for critical failures

4. **Security**
   - Use Jenkins credentials for all secrets
   - Never hardcode credentials in Jenkinsfile
   - Rotate SSH keys regularly
   - Limit Jenkins user permissions

## Advanced Configuration

### Parallel Test Execution

Modify Jenkinsfile to run tests in parallel:

```groovy
stage('Test') {
    parallel {
        stage('Unit Tests') {
            steps { /* run unit tests */ }
        }
        stage('Integration Tests') {
            steps { /* run integration tests */ }
        }
    }
}
```

### Multi-Environment Deployment

Deploy to staging before production:

```groovy
stage('Deploy Staging') {
    steps {
        sh './scripts/jenkins-deploy.sh staging-host ...'
    }
}
stage('Deploy Production') {
    when {
        branch 'main'
    }
    steps {
        sh './scripts/jenkins-deploy.sh prod-host ...'
    }
}
```

### Automated Rollback

Add rollback stage in post-failure:

```groovy
post {
    failure {
        script {
            // Rollback to previous version
            sh './scripts/rollback.sh'
        }
    }
}
```

## Next Steps

- Setup monitoring: [MONITORING.md](MONITORING.md)
- Configure deployment: [DEPLOYMENT.md](DEPLOYMENT.md)
