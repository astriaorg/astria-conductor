#!/bin/bash

set -o errexit -o nounset

DEFAULT_ACCOUNT_ID="0xb0E31D878F49Ec0403A25944d6B1aE1bf05D17E1"
# use default account id if evm_address envar is not set
ACCOUNT_ID=${evm_address:-$DEFAULT_ACCOUNT_ID}

mv genesis.json genesis.bak.json

# TODO - use dasel instead of jq?
# use jq to replace alloc value in genesis.json with ACCOUNT_ID envar
jq --arg accountId "$ACCOUNT_ID" \
  '.alloc |= with_entries( if .key | startswith("0x") then .key = $accountId else . end )' \
  genesis.bak.json > genesis.json
