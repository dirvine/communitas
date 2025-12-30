#!/bin/bash
# Copyright (c) 2025 Saorsa Labs Limited
# Generate Sparkle appcast.xml from GitHub release

set -euo pipefail

# Configuration
REPO_OWNER="${REPO_OWNER:-saorsa-labs}"
REPO_NAME="${REPO_NAME:-communitas}"
DMG_URL="${DMG_URL:-}"
VERSION="${VERSION:-}"
SIGNATURE="${SIGNATURE:-}"
DMG_SIZE="${DMG_SIZE:-0}"
RELEASE_NOTES="${RELEASE_NOTES:-}"
PUB_DATE="${PUB_DATE:-$(date -u +"%a, %d %b %Y %H:%M:%S %z")}"

# Generate appcast.xml
cat << EOF
<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Communitas Updates</title>
    <link>https://github.com/${REPO_OWNER}/${REPO_NAME}/releases</link>
    <description>Most recent changes with links to updates.</description>
    <language>en</language>
    <item>
      <title>Version ${VERSION}</title>
      <pubDate>${PUB_DATE}</pubDate>
      <sparkle:version>${VERSION}</sparkle:version>
      <sparkle:shortVersionString>${VERSION}</sparkle:shortVersionString>
      <description><![CDATA[
${RELEASE_NOTES}
      ]]></description>
      <enclosure
        url="${DMG_URL}"
        sparkle:edSignature="${SIGNATURE}"
        length="${DMG_SIZE}"
        type="application/octet-stream" />
    </item>
  </channel>
</rss>
EOF
