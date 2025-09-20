#!/usr/bin/env python3
"""Generate testnet configuration for multiple Communitas nodes"""

import json
import random
import os
from pathlib import Path

# Word lists for generating four-word identities
WORD_LIST = [
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
    "ocean", "forest", "mountain", "river", "valley", "desert", "lake", "meadow",
    "star", "moon", "sun", "comet", "nebula", "galaxy", "meteor", "planet",
    "eagle", "falcon", "hawk", "owl", "raven", "wolf", "bear", "lion",
    "north", "south", "east", "west", "center", "edge", "peak", "base"
]

def generate_four_words():
    """Generate a unique four-word identity"""
    return "-".join(random.sample(WORD_LIST, 4))

def create_node_config(node_num, bootstrap_nodes=None):
    """Create configuration for a single node"""

    # Generate unique identity
    four_words = generate_four_words()

    # Base port assignments (each node gets a range)
    base_port = 9000 + (node_num - 1) * 10
    quic_port = base_port
    dht_port = base_port + 1
    api_port = base_port + 2

    config = {
        "identity": {
            "four_words": four_words,
            "display_name": f"TestNode{node_num}",
            "device_name": f"testnet-node-{node_num}"
        },
        "network": {
            "quic_listen": f"0.0.0.0:{quic_port}",
            "dht_listen": f"0.0.0.0:{dht_port}",
            "api_listen": f"127.0.0.1:{api_port}",
            "bootstrap_nodes": bootstrap_nodes or []
        },
        "storage": {
            "data_dir": f"./data",
            "max_storage_gb": 10,
            "cache_size_mb": 512
        },
        "logging": {
            "level": "info",
            "file": f"./logs/node{node_num}.log",
            "stdout": True
        },
        "performance": {
            "worker_threads": 4,
            "max_connections": 100,
            "connection_timeout_ms": 5000
        }
    }

    return config, four_words

def main():
    """Generate testnet configuration"""

    print("🚀 Generating Communitas testnet configuration...")

    nodes = {}
    bootstrap_nodes = []

    # Generate configurations for 5 nodes
    for i in range(1, 6):
        config, four_words = create_node_config(i, bootstrap_nodes.copy())
        nodes[f"node{i}"] = {
            "config": config,
            "four_words": four_words,
            "quic_port": config["network"]["quic_listen"].split(":")[1],
            "dht_port": config["network"]["dht_listen"].split(":")[1],
            "api_port": config["network"]["api_listen"].split(":")[1]
        }

        # Add this node as a bootstrap for subsequent nodes
        if i <= 2:  # First two nodes are bootstrap nodes
            bootstrap_nodes.append({
                "address": f"127.0.0.1:{config['network']['quic_listen'].split(':')[1]}",
                "identity": four_words
            })

    # Update all nodes to know about bootstrap nodes
    for node_name, node_data in nodes.items():
        node_data["config"]["network"]["bootstrap_nodes"] = bootstrap_nodes

    # Write configuration files
    for node_name, node_data in nodes.items():
        config_path = Path(f"testnet/{node_name}/config/node.json")
        config_path.parent.mkdir(parents=True, exist_ok=True)

        with open(config_path, 'w') as f:
            json.dump(node_data["config"], f, indent=2)

        print(f"✅ Created config for {node_name}:")
        print(f"   Identity: {node_data['four_words']}")
        print(f"   QUIC Port: {node_data['quic_port']}")
        print(f"   DHT Port: {node_data['dht_port']}")
        print(f"   API Port: {node_data['api_port']}")

    # Save summary
    summary = {
        "nodes": nodes,
        "bootstrap_nodes": bootstrap_nodes,
        "total_nodes": len(nodes)
    }

    with open("testnet/testnet-summary.json", 'w') as f:
        json.dump(summary, f, indent=2)

    print("\n📋 Testnet Summary saved to testnet/testnet-summary.json")
    print(f"📊 Total nodes: {len(nodes)}")
    print(f"🔗 Bootstrap nodes: {len(bootstrap_nodes)}")

if __name__ == "__main__":
    main()