deployMultisig() {
    CHECK_VARIABLES RELAYER_ADDR_0 RELAYER_ADDR_1 RELAYER_ADDR_2 RELAYER_ADDR_3 \
    RELAYER_ADDR_4 RELAYER_ADDR_5 RELAYER_ADDR_6 RELAYER_ADDR_7 RELAYER_ADDR_8 \
    RELAYER_ADDR_9 SAFE MULTI_TRANSFER RELAYER_REQUIRED_STAKE SLASH_AMOUNT QUORUM MULTISIG_WASM

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^18" | bc)
    
    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${MULTISIG_WASM} \
    --args A:${SAFE} A:${MULTI_TRANSFER} \
    n:${MIN_STAKE} n:${SLASH_AMOUNT} n:${QUORUM} \
    A:${RELAYER_ADDR_0} A:${RELAYER_ADDR_1} A:${RELAYER_ADDR_2} A:${RELAYER_ADDR_3} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Multisig contract address: ${CONTRACT_ADDRESS}"
    update-config MULTISIG ${CONTRACT_ADDRESS}
}

changeChildContractsOwnershipSafe() {
    CHECK_VARIABLES SAFE MULTISIG

    operator sc invoke ${SAFE} changeOwnerAddress --key-file=${ALICE} \
    --args A:${MULTISIG} \
    --await --sign --node ${PROXY}
}

changeChildContractsOwnershipMultiTransfer() {
    CHECK_VARIABLES MULTI_TRANSFER MULTISIG

    operator sc invoke ${MULTI_TRANSFER} changeOwnerAddress --key-file=${ALICE} \
    --args A:${MULTISIG} \
    --await --sign --node ${PROXY}
}

clearMapping() {
    CHECK_VARIABLES ERC20_TOKEN CHAIN_SPECIFIC_TOKEN MULTISIG

    operator sc invoke ${MULTISIG} clearMapping --key-file=${ALICE} \
    --args A:${ERC20_TOKEN} String:${CHAIN_SPECIFIC_TOKEN} \
    --await --sign --node ${PROXY}
}

addMapping() {
    CHECK_VARIABLES ERC20_TOKEN CHAIN_SPECIFIC_TOKEN MULTISIG

    operator sc invoke ${MULTISIG} addMapping --key-file=${ALICE} \
    --args A:${ERC20_TOKEN} String:${CHAIN_SPECIFIC_TOKEN} \
    --await --sign --node ${PROXY}
}

addTokenToWhitelist() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN CHAIN_SPECIFIC_TOKEN_TICKER MULTISIG MINTBURN_WHITELIST NATIVE_TOKEN

    BALANCE=$(echo "$TOTAL_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)
    MINT=$(echo "$MINT_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)
    BURN=$(echo "$BURN_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)

    operator sc invoke ${MULTISIG} kdaSafeAddTokenToWhitelist --key-file=${ALICE} \
    --args String:${CHAIN_SPECIFIC_TOKEN} String:${CHAIN_SPECIFIC_TOKEN_TICKER} A:${MINTBURN_WHITELIST} n:${NATIVE_TOKEN} \
    n:${BALANCE} n:${MINT} n:${BURN} \
    --await --sign --node ${PROXY}
}

removeTokenFromWhitelist() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN CHAIN_SPECIFIC_TOKEN_TICKER MULTISIG

    operator sc invoke ${MULTISIG} kdaSafeRemoveTokenFromWhitelist --key-file=${ALICE} \
    --args String:${CHAIN_SPECIFIC_TOKEN} \
    --await --sign --node ${PROXY}
}

kdaSafeSetMaxTxBatchSize() {
    CHECK_VARIABLES MAX_TX_PER_BATCH MULTISIG

    operator sc invoke ${MULTISIG} kdaSafeSetMaxTxBatchSize --key-file=${ALICE} \
    --args n:${MAX_TX_PER_BATCH} \
    --await --sign --node ${PROXY}
}

kdaSafeSetMaxTxBatchBlockDuration() {
    CHECK_VARIABLES MAX_TX_BLOCK_DURATION_PER_BATCH MULTISIG

    operator sc invoke ${MULTISIG} kdaSafeSetMaxTxBatchBlockDuration --key-file=${ALICE} \
    --args n:${MAX_TX_BLOCK_DURATION_PER_BATCH} \
    --await --sign --node ${PROXY}
}

changeQuorum() {
    CHECK_VARIABLES QUORUM MULTISIG

    operator sc invoke ${MULTISIG} changeQuorum --key-file=${ALICE} \
    --args n:${QUORUM} \
    --await --sign --node ${PROXY}
}

pause() {
    CHECK_VARIABLES MULTISIG

    operator sc invoke ${MULTISIG} pause --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

pauseV2() {
    CHECK_VARIABLES MULTISIG_v2

    operator sc invoke ${MULTISIG_v2} pause --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

pauseKdaSafe() {
    CHECK_VARIABLES MULTISIG

    operator sc invoke ${MULTISIG} pauseKdaSafe --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

pauseKdaSafeV2() {
    CHECK_VARIABLES MULTISIG_v2

    operator sc invoke ${MULTISIG_v2} pauseKdaSafe --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

pauseProxy() {
    CHECK_VARIABLES MULTISIG

    operator sc invoke ${MULTISIG} pauseProxy --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

unpause() {
    CHECK_VARIABLES MULTISIG

    operator sc invoke ${MULTISIG} unpause --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

unpauseKdaSafe() {
    CHECK_VARIABLES MULTISIG

    operator sc invoke ${MULTISIG} unpauseKdaSafe --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

unpauseProxy() {
    CHECK_VARIABLES MULTISIG

    operator sc invoke ${MULTISIG} unpauseProxy --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

kdaSafeSetMaxBridgedAmountForToken() {
    CHECK_VARIABLES MAX_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC CHAIN_SPECIFIC_TOKEN MULTISIG

    MAX=$(echo "scale=0; $MAX_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)
    
    operator sc invoke ${MULTISIG} kdaSafeSetMaxBridgedAmountForToken --key-file=${ALICE} \
    --args String:${CHAIN_SPECIFIC_TOKEN} n:${MAX} \
    --await --sign --node ${PROXY}
}

multiTransferKdaSetMaxBridgedAmountForToken() {
    CHECK_VARIABLES MAX_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC CHAIN_SPECIFIC_TOKEN MULTISIG

    MAX=$(echo "scale=0; $MAX_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)
    
    operator sc invoke ${MULTISIG} multiTransferKdaSetMaxBridgedAmountForToken --key-file=${ALICE} \
    --args String:${CHAIN_SPECIFIC_TOKEN} n:${MAX} \
    --await --sign --node ${PROXY}
}

multiTransferKdaSetMaxBridgedAmountForTokenWithRAWValue() {
    CHECK_VARIABLES ETH_MAX_AMOUNT CHAIN_SPECIFIC_TOKEN MULTISIG

    operator sc invoke ${MULTISIG} multiTransferKdaSetMaxBridgedAmountForToken --key-file=${ALICE} \
    --args String:${CHAIN_SPECIFIC_TOKEN} n:${ETH_MAX_AMOUNT} \
    --await --sign --node ${PROXY}
}

setMultiTransferOnKdaSafeThroughMultisig() {
    CHECK_VARIABLES MULTISIG

    operator sc invoke ${MULTISIG} setMultiTransferOnKdaSafe --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

setKdaSafeOnMultiTransferThroughMultisig() {
    CHECK_VARIABLES MULTISIG

    operator sc invoke ${MULTISIG} setKdaSafeOnMultiTransfer --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

initSupplyMintBurn() {
  CHECK_VARIABLES MULTISIG

  echo -e
  echo "PREREQUIREMENTS: The MINT_BALANCE & BURN_BALANCE values should be defined in configs.cfg file"
  echo "The script automatically apply denomination factors based on the number of the decimals the token has"
  echo -e

  confirmation-with-skip manual-update-config-file

  MINT=$(echo "$MINT_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)
  BURN=$(echo "$BURN_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)

  MINT=$(echo ${MINT%.*}) # trim decimals, if existing
  BURN=$(echo ${BURN%.*}) # trim decimals, if existing

  operator sc invoke ${MULTISIG} initSupplyMintBurnKdaSafe --key-file=${ALICE} \
  --args String:${CHAIN_SPECIFIC_TOKEN} n:${MINT} n:${BURN} \
  --await --sign --node ${PROXY}
}

syncValueWithEthereumDenom() {
  CHECK_VARIABLES MULTISIG SAFE

  read -p "Chain specific token (human readable): " TOKEN
  read -p "Denominated value on Ethereum (should contain all digits): " ETH_VALUE

  # Using operator to query the contract
  SAFE_QUERY_BURN=$(operator sc query ${SAFE} getBurnBalances --args String:$TOKEN --node=${PROXY})
  EXISTING_BURN=$(echo $SAFE_QUERY_BURN | jq '.[0].number')
  
  SAFE_QUERY_MINT=$(operator sc query ${SAFE} getMintBalances --args String:$TOKEN --node=${PROXY})
  EXISTING_MINT=$(echo $SAFE_QUERY_MINT | jq '.[0].number')
  
  NEW_MINT=$(echo "$ETH_VALUE+$EXISTING_BURN" | bc)
  DIFF=$(echo "$EXISTING_MINT-$EXISTING_BURN" | bc)
  NEW_DIFF=$(echo "$NEW_MINT-$EXISTING_BURN" | bc)

  echo "For token ${TOKEN} the existing mint is ${EXISTING_MINT} and existing burn is ${EXISTING_BURN}. The minted value will be replaced with ${NEW_MINT}"
  echo "Existing diff ${DIFF}, new diff will be ${NEW_DIFF}"

  operator sc invoke ${MULTISIG} initSupplyMintBurnKdaSafe --key-file=${ALICE} \
    --args String:${TOKEN} n:${NEW_MINT} n:${EXISTING_BURN} \
    --await --sign --node ${PROXY}
}

upgradeMultisig() {
    CHECK_VARIABLES SAFE MULTI_TRANSFER MULTISIG_WASM

    SC_RESULT=$(eval operator sc upgrade ${MULTISIG} --key-file=${ALICE} \
    --wasm ${MULTISIG_WASM} \
    --args A:${SAFE} A:${MULTI_TRANSFER} \
    --await --result-only --sign --node ${PROXY} \

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Multisig contract upgraded successfully at address: ${CONTRACT_ADDRESS}"
}
