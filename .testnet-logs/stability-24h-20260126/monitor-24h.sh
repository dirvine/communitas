#!/bin/bash

# 24-Hour Stability Monitoring Script for Communitas TestNet
# Duration: 24 hours (1440 minutes)
# Interval: 30 minutes
# Total snapshots: 48

LOG_DIR="/Users/davidirvine/Desktop/Devel/projects/communitas/.testnet-logs/stability-24h-20260126"
MONITOR_LOG="$LOG_DIR/monitoring.log"
DURATION_MINUTES=1440
INTERVAL_MINUTES=30

NODES=(
    "saorsa-2:142.93.199.50"
    "saorsa-3:147.182.234.192"
    "saorsa-4:206.189.7.117"
    "saorsa-5:144.126.230.161"
    "saorsa-6:65.21.157.229"
    "saorsa-7:116.203.101.172"
    "saorsa-8:149.28.156.231"
    "saorsa-9:45.77.176.184"
    "saorsa-10:77.42.39.239"
)

echo "╔══════════════════════════════════════════════════════════════╗" | tee "$MONITOR_LOG"
echo "║  COMMUNITAS 24-HOUR STABILITY TEST                           ║" | tee -a "$MONITOR_LOG"
echo "╠══════════════════════════════════════════════════════════════╣" | tee -a "$MONITOR_LOG"
echo "║  Start: $(date -u +"%Y-%m-%dT%H:%M:%SZ")                              ║" | tee -a "$MONITOR_LOG"
echo "║  Duration: 24 hours (1440 minutes)                           ║" | tee -a "$MONITOR_LOG"
echo "║  Interval: 30 minutes                                        ║" | tee -a "$MONITOR_LOG"
echo "║  Expected snapshots: 48                                      ║" | tee -a "$MONITOR_LOG"
echo "╚══════════════════════════════════════════════════════════════╝" | tee -a "$MONITOR_LOG"
echo "" | tee -a "$MONITOR_LOG"

snapshot_num=0
end_time=$(($(date +%s) + DURATION_MINUTES * 60))
start_time=$(date +%s)

while [ $(date +%s) -lt $end_time ]; do
    snapshot_num=$((snapshot_num + 1))
    timestamp=$(date -u +"%Y%m%d_%H%M%S")
    snapshot_dir="$LOG_DIR/snapshot_${snapshot_num}_${timestamp}"
    mkdir -p "$snapshot_dir"

    elapsed=$(( ($(date +%s) - start_time) / 60 ))
    remaining=$(( (end_time - $(date +%s)) / 60 ))

    echo "=== SNAPSHOT $snapshot_num at $(date -u +"%Y-%m-%dT%H:%M:%SZ") (${elapsed}m elapsed, ${remaining}m remaining) ===" | tee -a "$MONITOR_LOG"

    nodes_up=0
    nodes_down=0
    total_memory=0

    echo "| Node | Status | PID | Memory | Uptime |" > "$snapshot_dir/status.md"
    echo "|------|--------|-----|--------|--------|" >> "$snapshot_dir/status.md"

    for entry in "${NODES[@]}"; do
        node="${entry%%:*}"
        ip="${entry##*:}"

        proc_count=$(ssh -o ConnectTimeout=10 -o BatchMode=yes root@$ip 'pgrep -c communitas-head 2>/dev/null' 2>/dev/null || echo "0")

        if [ "$proc_count" -gt "0" ] && [ "$proc_count" != "0" ]; then
            nodes_up=$((nodes_up + 1))
            pid=$(ssh -o ConnectTimeout=10 -o BatchMode=yes root@$ip 'pgrep -f communitas-headless | head -1' 2>/dev/null)
            mem=$(ssh -o ConnectTimeout=10 -o BatchMode=yes root@$ip "ps -p $pid -o rss= 2>/dev/null" 2>/dev/null | tr -d ' ')
            mem_mb=$((mem / 1024))
            total_memory=$((total_memory + mem_mb))
            uptime_sec=$(ssh -o ConnectTimeout=10 -o BatchMode=yes root@$ip "ps -p $pid -o etimes= 2>/dev/null" 2>/dev/null | tr -d ' ')
            uptime_min=$((uptime_sec / 60))
            echo "| $node | UP | $pid | ${mem_mb}MB | ${uptime_min}m |" >> "$snapshot_dir/status.md"

            # Collect recent logs (last 30 min)
            ssh -o ConnectTimeout=10 -o BatchMode=yes root@$ip 'journalctl -u communitas --since "30 minutes ago" --no-pager 2>/dev/null | tail -50' > "$snapshot_dir/${node}.log" 2>/dev/null || true
        else
            nodes_down=$((nodes_down + 1))
            echo "| $node | DOWN | - | - | - |" >> "$snapshot_dir/status.md"
        fi
    done

    avg_memory=0
    if [ $nodes_up -gt 0 ]; then
        avg_memory=$((total_memory / nodes_up))
    fi

    echo "Nodes: $nodes_up UP, $nodes_down DOWN | Avg Memory: ${avg_memory}MB" | tee -a "$MONITOR_LOG"

    # UDP connectivity test from saorsa-2 (quick spot check)
    echo "UDP connectivity:" >> "$snapshot_dir/connectivity.log"
    for target_ip in 147.182.234.192 206.189.7.117 144.126.230.161 65.21.157.229; do
        result=$(ssh -o ConnectTimeout=10 -o BatchMode=yes root@142.93.199.50 "timeout 2 bash -c 'echo test | nc -u -w1 $target_ip 11000' && echo OK || echo FAIL" 2>/dev/null)
        echo "$target_ip: $result" >> "$snapshot_dir/connectivity.log"
    done

    # Save summary
    echo "{\"snapshot\": $snapshot_num, \"timestamp\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\", \"nodes_up\": $nodes_up, \"nodes_down\": $nodes_down, \"avg_memory_mb\": $avg_memory}" >> "$LOG_DIR/metrics.jsonl"

    echo "Snapshot $snapshot_num complete" | tee -a "$MONITOR_LOG"
    echo "" | tee -a "$MONITOR_LOG"

    # Wait for next interval
    remaining_sec=$((end_time - $(date +%s)))
    if [ $remaining_sec -gt 0 ]; then
        sleep_time=$((INTERVAL_MINUTES * 60))
        if [ $sleep_time -gt $remaining_sec ]; then
            sleep_time=$remaining_sec
        fi
        sleep $sleep_time
    fi
done

echo "╔══════════════════════════════════════════════════════════════╗" | tee -a "$MONITOR_LOG"
echo "║  24-HOUR MONITORING COMPLETE                                 ║" | tee -a "$MONITOR_LOG"
echo "╠══════════════════════════════════════════════════════════════╣" | tee -a "$MONITOR_LOG"
echo "║  End: $(date -u +"%Y-%m-%dT%H:%M:%SZ")                                ║" | tee -a "$MONITOR_LOG"
echo "║  Total snapshots: $snapshot_num                                       ║" | tee -a "$MONITOR_LOG"
echo "╚══════════════════════════════════════════════════════════════╝" | tee -a "$MONITOR_LOG"
