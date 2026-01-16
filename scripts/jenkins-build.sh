#!/bin/bash
set -e

IMAGE_NAME=${1:-kamachess}
IMAGE_TAG=${2:-latest}
BUILD_DIR=${3:-.}

echo "Building ${IMAGE_NAME}:${IMAGE_TAG} in ${BUILD_DIR}"

cd "${BUILD_DIR}"

echo "Building Docker image..."
docker build \
    --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
    --tag "${IMAGE_NAME}:latest" \
    -f Dockerfile \
    .

if [ "${RUN_TESTS}" = "true" ]; then
    echo "Running tests..."
    docker run --rm \
        -v "$(pwd):/workspace" \
        -w /workspace \
        "${IMAGE_NAME}:${IMAGE_TAG}" \
        cargo test --all-features --verbose
fi

echo "Build complete: ${IMAGE_NAME}:${IMAGE_TAG}"
