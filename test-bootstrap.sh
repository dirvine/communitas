#!/bin/bash

echo "Testing connection to bootstrap nodes..."

# Test connection to bootstrap node
echo "Testing bootstrap node (port 9001)..."
nc -zv 159.89.81.21 9001

# Test a few p2p nodes
echo "Testing p2p node (port 9100)..."
nc -zv 159.89.81.21 9100

echo "Testing p2p node (port 9110)..."
nc -zv 159.89.81.21 9110

echo "Testing p2p node (port 9120)..."
nc -zv 159.89.81.21 9120

echo "All connection tests completed."