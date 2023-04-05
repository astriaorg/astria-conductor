#!/bin/bash

# use jq to replace alloc value in genesis.json with ACCOUNT_ID envar
mv genesis.json genesis.bak.json
jq --arg accountId "$ACCOUNT_ID" \
  '.alloc |= with_entries( if .key | startswith("0x") then .key = $accountId else . end )' \
  genesis.bak.json > genesis.json

geth --datadir ~/.astriageth/ init genesis.json
geth --datadir ~/.astriageth/ --http --http.port=8545 \
  --ws --ws.port=8545 --networkid=1337 --http.corsdomain='*' --ws.origins='*' \
  --grpc --grpc.addr "0.0.0.0" --grpc.port 50051
