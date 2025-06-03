deployAggregator() {
    CHECK_VARIABLES AGGREGATOR_WASM CHAIN_SPECIFIC_TOKEN ORACLE_ADDR_0 ORACLE_ADDR_1 ORACLE_ADDR_2

    STAKE=$(echo "$ORACLE_REQUIRED_STAKE*10^6" | bc)

    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${AGGREGATOR_WASM} \
    --args String:KLV --args u64:${STAKE} --args u64:1 --args u64:2 --args u64:3 \
    --args A:${ORACLE_ADDR_0} --args A:${ORACLE_ADDR_1} --args A:${ORACLE_ADDR_2} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')
    
    echo ""
    echo "Price agregator: ${CONTRACT_ADDRESS}"
    update-config AGGREGATOR ${CONTRACT_ADDRESS}
}

stakeOracles() {
    CHECK_VARIABLES AGGREGATOR

    STAKE=$(echo "$ORACLE_REQUIRED_STAKE*10^6" | bc)
    echo "---------------------------------------------------------"
    operator sc invoke ${AGGREGATOR} stake --key-file=${ORACLE_WALLET0} \
    --values KLV=${STAKE} --await --sign --node ${PROXY}

    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    operator sc invoke ${AGGREGATOR} stake --key-file=${ORACLE_WALLET1} \
    --values KLV=${STAKE} --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    operator sc invoke ${AGGREGATOR} stake --key-file=${ORACLE_WALLET2} \
    --values KLV=${STAKE} --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
}

submitAggregatorBatch() {
    CHECK_VARIABLES AGGREGATOR CHAIN_SPECIFIC_TOKEN FEE_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC

    FEE=$(echo "scale=0; $FEE_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)

    CURRENT_TIME=$(date +%s)
    operator sc invoke ${AGGREGATOR} submitBatch --key-file=${ORACLE_WALLET0} \
    --args String:GWEI --args String:${CHAIN_SPECIFIC_TOKEN_TICKER} --args u64:${CURRENT_TIME} --args n:${FEE} --args u8:0 \
    --await --sign --node ${PROXY}

    CURRENT_TIME=$(date +%s)
    operator sc invoke ${AGGREGATOR} submitBatch --key-file=${ORACLE_WALLET1} \
    --args String:GWEI --args String:${CHAIN_SPECIFIC_TOKEN_TICKER} --args u64:${CURRENT_TIME} --args n:${FEE} --args u8:0 \
    --await --sign --node ${PROXY}

    CURRENT_TIME=$(date +%s)
    operator sc invoke ${AGGREGATOR} submitBatch --key-file=${ORACLE_WALLET2} \
    --args String:GWEI --args String:${CHAIN_SPECIFIC_TOKEN_TICKER} --args u64:${CURRENT_TIME} --args n:${FEE} --args u8:0 \
    --await --sign --node ${PROXY}
}

setPairDecimals() {
    CHECK_VARIABLES AGGREGATOR

    operator sc invoke ${AGGREGATOR} setPairDecimals --key-file=${ALICE} \
    --args String:GWEI --args String:${CHAIN_SPECIFIC_TOKEN_TICKER} --args u8:0 \
    --await --sign --node ${PROXY}
}

pauseAggregator() {
    CHECK_VARIABLES AGGREGATOR

    operator sc invoke ${AGGREGATOR} pause --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

unpauseAggregator() {
    CHECK_VARIABLES AGGREGATOR

    operator sc invoke ${AGGREGATOR} unpause --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

aggregator-upgrade() {
    CHECK_VARIABLES AGGREGATOR AGGREGATOR_WASM

    operator sc upgrade ${AGGREGATOR} --key-file=${ALICE} \
    --wasm ${AGGREGATOR_WASM} \
    --await --sign --node ${PROXY}
    
    echo ""
    echo "Price aggregator upgraded successfully at address: ${AGGREGATOR}"
}
