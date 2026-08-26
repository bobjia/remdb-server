#!/bin/bash
# Script to create a full snapshot via remdbcli

echo "Creating full snapshot..."
echo "snapshot full" | cargo run --bin remdbcli 2>&1 | grep -E "(Full snapshot|Error)"
echo ""
echo "Checking snapshots directory:"
ls -lh db/snapshot/ | grep -E "(full_|incremental_)" | tail -5
