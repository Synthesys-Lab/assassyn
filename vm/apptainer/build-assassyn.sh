#!/bin/bash
# Build script for Assassyn Apptainer containers
# Usage: ./build-assassyn.sh [branch-name]

set -e

BRANCH=${1:-main}
BASE_IMAGE="assassyn-base.sif"
BRANCH_IMAGE="assassyn-${BRANCH}.sif"
BASE_DEF="assassyn-base.def"
BRANCH_DEF="assassyn-branch.def"

echo "=== Building Assassyn Container for branch: $BRANCH ==="

# Check if definition files exist
if [ ! -f "$BASE_DEF" ]; then
    echo "Error: $BASE_DEF not found!"
    exit 1
fi

if [ ! -f "$BRANCH_DEF" ]; then
    echo "Error: $BRANCH_DEF not found!"
    exit 1
fi

# Build base image if it doesn't exist
if [ ! -f "$BASE_IMAGE" ]; then
    echo "Building base image (this may take a while)..."
    apptainer build "$BASE_IMAGE" "$BASE_DEF"
    echo "Base image built successfully!"
else
    echo "Base image already exists, skipping..."
fi

# Build branch-specific image
echo "Building branch image for $BRANCH..."
apptainer build --build-arg ASSASSYN_BRANCH="$BRANCH" "$BRANCH_IMAGE" "$BRANCH_DEF"

echo "=== Build completed successfully! ==="
echo "Container: $BRANCH_IMAGE"
echo "Usage: apptainer exec --no-home $BRANCH_IMAGE python your_script.py"
