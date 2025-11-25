#!/bin/sh

# builds all wasm targets and generates proxies

set -e

KSC=~/klever-sdk/ksc

# Find all contract directories (those containing meta/ folder with Cargo.toml)
CONTRACTS=$(find . -type d -name "meta" -exec dirname {} \; | grep -v target | sort -u)

for contract_path in $CONTRACTS
do
    echo ""
    echo "Building contract: $contract_path"
    (set -x; $KSC all build --path "$contract_path")
    
    echo ""
    echo "Generating proxies: $contract_path"
    (set -x; $KSC all proxy --path "$contract_path")
done

echo ""
echo "All contracts built and proxies generated successfully!"
