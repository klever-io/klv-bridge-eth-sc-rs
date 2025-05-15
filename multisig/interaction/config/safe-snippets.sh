deploySafe() {
    CHECK_VARIABLES SAFE_WASM MULTI_TRANSFER AGGREGATOR
    
    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${SAFE_WASM} \
    --args A:${AGGREGATOR} --args A:${MULTI_TRANSFER} --args n:1 \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Safe contract address: ${CONTRACT_ADDRESS}"
    update-config SAFE ${CONTRACT_ADDRESS}
}   

setLocalRolesKdaSafe() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN SAFE
    ADD_ROLE_TRIGGER=6

    operator kda trigger ${ADD_ROLE_TRIGGER} --kdaID=${CHAIN_SPECIFIC_TOKEN} --addRolesMint=${SAFE} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

unsetLocalRolesKdaSafe() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN SAFE
    REMOVE_ROLE_TRIGGER=7

    operator kda trigger ${REMOVE_ROLE_TRIGGER} --kdaID=${CHAIN_SPECIFIC_TOKEN} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

setBridgedTokensWrapperOnKdaSafe() {
    CHECK_VARIABLES SAFE BRIDGED_TOKENS_WRAPPER

    operator sc invoke ${SAFE} setBridgedTokensWrapperAddress --key-file=${ALICE} \
    --args A:${BRIDGED_TOKENS_WRAPPER} \
    --await --sign --node ${PROXY}
}

deploySafeForUpgrade() {
    CHECK_VARIABLES SAFE_WASM MULTI_TRANSFER AGGREGATOR

    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${SAFE_WASM} \
    --args A:${AGGREGATOR} --args A:${MULTI_TRANSFER} --args n:1 \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "New safe contract address: ${CONTRACT_ADDRESS}"
    
    # Store for later use in upgradeSafeContract
    NEW_SAFE_ADDR=${CONTRACT_ADDRESS}
}

upgradeSafeContract() {
    CHECK_VARIABLES MULTISIG SAFE NEW_SAFE_ADDR AGGREGATOR MULTI_TRANSFER

    operator sc invoke ${MULTISIG} upgradeChildContractFromSource --key-file=${ALICE} \
    --args A:${SAFE} --args A:${NEW_SAFE_ADDR} --args bool:true \
    --args A:${AGGREGATOR} --args A:${MULTI_TRANSFER} --args n:1 \
    --await --sign --node ${PROXY}

    update-config SAFE ${NEW_SAFE_ADDR}
}
