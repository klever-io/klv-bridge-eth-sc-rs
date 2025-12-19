#!/bin/bash
set -e

#Make script aware of its location
SCRIPTPATH="$( cd "$(dirname -- "$0")" ; pwd -P )"

source $SCRIPTPATH/config/configs.cfg
source $SCRIPTPATH/config/helper.cfg
source $SCRIPTPATH/config/menu_functions.cfg
source $SCRIPTPATH/release-v3/menu_functions.cfg

case "$1" in

### PART 1

'deploy-bridge-contracts-eth-v3')
  confirmation deploy-bridge-contracts-eth-v3
  ;;

'unpause-contracts-eth-v3')
  confirmation unpause-contracts-eth-v3
  ;;

'set-tokens-on-eth')
  confirmation set-tokens-on-eth
  ;;

'stake-oracles')
  confirmation stake-oracles
  ;;

'submit-aggregation-batches-eth')
  confirmation submit-aggregation-batches-eth
  ;;

'stake-relayers-eth')
  confirmation stake-relayers-eth
  ;;

'set-roles-on-kda-safe-eth')
  confirmation set-roles-on-kda-safe-eth
  ;;

### PART 2

'upgrade-wrapper')
  confirmation upgrade-wrapper
  ;;

'unpause-wrapper')
  confirmation unpause-wrapper
  ;;

'set-token-limits-on-eth')
  confirmation set-token-limits-on-eth
  ;;

*)
  echo "Usage: Invalid choice: '"$1"'"
  echo -e
  echo "Choose from:"
  echo "PART 1 - Ethereum:"
  echo " 1.1 deploy-bridge-contracts-eth-v3"
  echo " 1.2 unpause-contracts-eth-v3"
  echo " 1.3 set-tokens-on-eth"
  echo " -----------"
  echo " 1.4 stake-oracles"
  echo " 1.5 submit-aggregation-batches-eth"
  echo " 1.6 stake-relayers-eth"
  echo " -----------"
  echo " 1.7 set-roles-on-kda-safe-eth"
  echo -e
  echo "PART 2 - Upgrade wrapper:"
  echo " 2.1 upgrade-wrapper"
  echo " 2.2 unpause-wrapper"
  echo -e
  echo "PART 3 - Limits:"
  echo " 3.1 set-token-limits-on-eth"
  echo -e
  ;;

esac