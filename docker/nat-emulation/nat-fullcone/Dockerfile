# Full Cone NAT Emulator
# Copyright (c) 2025 Saorsa Labs Limited

FROM alpine:3.19

RUN apk add --no-cache iptables bash conntrack-tools

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["sleep", "infinity"]
