#!/bin/sh

set -o errexit -o nounset

sed -i'.bak' "s#\"tcp://127.0.0.1:26659\"#\"tcp://0.0.0.0:$bridge_host_port\"#g" $home_dir/config/config.toml

./celestia bridge init \
  --node.store "$home_dir/bridge" \
  --core.ip 127.0.0.1
cp -r "$home_dir/keyring-test" "$home_dir/bridge/keys/"
