#!/bin/bash
set -e

DEPLOY_HOST=${1}
DEPLOY_USER=${2:-ubuntu}
DEPLOY_PATH=${3:-/opt/kamachess}
IMAGE_TAG=${4}

if [ -z "${DEPLOY_HOST}" ] || [ -z "${IMAGE_TAG}" ]; then
    echo "Usage: $0 <host> <user> <deploy_path> <image_tag>"
    exit 1
fi

echo "Deploying ${IMAGE_TAG} to ${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}"

TEMP_DIR=$(mktemp -d)
IMAGE_TAR="${TEMP_DIR}/kamachess-${IMAGE_TAG}.tar"
echo "Exporting Docker image to ${IMAGE_TAR}..."
docker save "${IMAGE_TAG}" -o "${IMAGE_TAR}"
if [ -n "${SSH_KEY}" ] && [ -f "${SSH_KEY}" ]; then
    SSH_CMD="ssh -i ${SSH_KEY}"
    SCP_CMD="scp -i ${SSH_KEY}"
elif [ -n "${SSH_KEY}" ]; then
    SSH_KEY_FILE=$(mktemp)
    echo "${SSH_KEY}" > "${SSH_KEY_FILE}"
    chmod 600 "${SSH_KEY_FILE}"
    SSH_CMD="ssh -i ${SSH_KEY_FILE}"
    SCP_CMD="scp -i ${SSH_KEY_FILE}"
else
    SSH_CMD="ssh"
    SCP_CMD="scp"
fi

echo "Copying image to server..."
${SCP_CMD} "${IMAGE_TAR}" "${DEPLOY_USER}@${DEPLOY_HOST}:/tmp/"

echo "Deploying on remote server..."
${SSH_CMD} -o StrictHostKeyChecking=no "${DEPLOY_USER}@${DEPLOY_HOST}" <<EOF
set -e
cd ${DEPLOY_PATH}

echo "Loading Docker image..."
docker load -i /tmp/kamachess-${IMAGE_TAG}.tar
rm -f /tmp/kamachess-${IMAGE_TAG}.tar

if [ -d ".git" ]; then
    echo "Pulling latest code..."
    git pull || true
fi

echo "Restarting services..."
docker-compose pull || true
docker-compose up -d --no-deps bot

echo "Waiting for service to be healthy..."
sleep 10

if docker-compose ps bot | grep -q "Up"; then
    echo "Deployment successful!"
    docker-compose ps
else
    echo "Deployment failed - service not running"
    docker-compose logs bot
    exit 1
fi
EOF
rm -f "${IMAGE_TAR}"
rmdir "${TEMP_DIR}" || true
if [ -n "${SSH_KEY_FILE}" ]; then
    rm -f "${SSH_KEY_FILE}"
fi

echo "Deployment complete!"
