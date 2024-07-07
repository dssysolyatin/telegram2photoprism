#!/bin/bash

ARCHITECTURE=$(uname -m)

# Check if the architecture is arm64
if [ "$ARCHITECTURE" != "aarch64" ] && [ "$ARCHITECTURE" != "arm64" ]; then
    echo "The architecture is not arm64. Current architecture: $ARCHITECTURE. Please run the script on an arm64 machine."
    exit 1
fi

VERSION=v$(cargo pkgid | cut -d "#" -f2)

docker build -t dssysolyatin/telegram2photoprism:${VERSION}-arm64 --push .

docker manifest create dssysolyatin/telegram2photoprism:${VERSION} dssysolyatin/telegram2photoprism:${VERSION}-arm64 dssysolyatin/telegram2photoprism:${VERSION}-amd64
docker manifest push dssysolyatin/telegram2photoprism:${VERSION}

docker manifest rm dssysolyatin/telegram2photoprism:latest
docker manifest create  dssysolyatin/telegram2photoprism:latest dssysolyatin/telegram2photoprism:${VERSION}-arm64 dssysolyatin/telegram2photoprism:${VERSION}-amd64
docker manifest push dssysolyatin/telegram2photoprism:latest