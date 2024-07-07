#!/bin/bash

ARCHITECTURE=$(uname -m)
if [ "$ARCHITECTURE" == "aarch64" ] || [ "$ARCHITECTURE" == "arm64" ]; then

else
    echo "The architecture is not arm64. Current architecture: $ARCHITECTURE. Please run script on arm64 machine"
    exit
fi

VERSION=v$(cargo pkgid | cut -d "#" -f2)

docker build -t dssysolyatin/telegram2photoprism:${VERSION}-arm64 .

docker manifest create dssysolyatin/telegram2photoprism:${VERSION} \
dssysolyatin/telegram2photoprism:${VERSION}-arm64 \
dssysolyatin/telegram2photoprism:${VERSION}-amd64

docker manifest push dssysolyatin/telegram2photoprism:${VERSION}

docker manifest create dssysolyatin/telegram2photoprism:latest \
dssysolyatin/telegram2photoprism:${VERSION}-arm64 \
dssysolyatin/telegram2photoprism:${VERSION}-amd64

docker manifest push dssysolyatin/telegram2photoprism:latest