#!/bin/bash

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <volume_id> [target_directory]"
  echo "Example: $0 nhm9t3owr7ze7ij01uduaiop ../itell/apps/demo/content"
  exit 1
fi

# Check if the release binary exists
if [ ! -f "./target/release/fetch_volume" ]; then
  echo "Error: no compiled target found"
  echo "Please run 'make build' first"
  exit 1
fi

volume_id=$1
target_dir=$2

./target/release/fetch_volume "$volume_id" "$target_dir"
