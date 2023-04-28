#!/bin/bash

set -o errexit -o nounset

DEFAULT_ACCOUNT_ID="0xb0E31D878F49Ec0403A25944d6B1aE1bf05D17E1"
# use default account id if genesis_address envar is not set
ACCOUNT_ID=${genesis_address:-$DEFAULT_ACCOUNT_ID}

echo "Modifying genesis.json to allocate funds to $ACCOUNT_ID"

mv /genesis.json $home_dir/genesis.bak.json

# use jq to replace alloc value in genesis.json with ACCOUNT_ID envar
jq --arg accountId "$ACCOUNT_ID" \
  '.alloc |= with_entries( if .key | startswith("0x") then .key = $accountId else . end )' \
  $home_dir/genesis.bak.json > $home_dir/genesis.json
