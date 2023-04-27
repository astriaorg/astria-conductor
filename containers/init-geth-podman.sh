#!/bin/bash

set -o errexit -o nounset

geth --datadir $home_dir/.astriageth/ init genesis.json
