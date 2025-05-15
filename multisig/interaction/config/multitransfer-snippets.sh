deployMultiTransfer() {
    CHECK_VARIABLES MULTI_TRANSFER_WASM

    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${MULTI_TRANSFER_WASM} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Multi transfer contract address: ${CONTRACT_ADDRESS}"
    update-config MULTI_TRANSFER ${CONTRACT_ADDRESS}
}

setBridgedTokensWrapperOnMultiTransfer() {
    CHECK_VARIABLES MULTI_TRANSFER BRIDGED_TOKENS_WRAPPER

    operator sc invoke ${MULTI_TRANSFER} setWrappingContractAddress --key-file=${ALICE} \
    --args A:${BRIDGED_TOKENS_WRAPPER} \
    --await --sign --node ${PROXY}
}

deployMultiTransferForUpgrade() {
    CHECK_VARIABLES MULTI_TRANSFER_WASM

    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${MULTI_TRANSFER_WASM} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "New multi transfer contract address: ${CONTRACT_ADDRESS}"
    
    # Store for later use in upgradeMultiTransferContract
    NEW_MULTI_TRANSFER_ADDR=${CONTRACT_ADDRESS}
}

upgradeMultiTransferContract() {
    CHECK_VARIABLES MULTISIG MULTI_TRANSFER NEW_MULTI_TRANSFER_ADDR

    operator sc invoke ${MULTISIG} upgradeChildContractFromSource --key-file=${ALICE} \
    --args A:${MULTI_TRANSFER} --args A:${NEW_MULTI_TRANSFER_ADDR} --args bool:false \
    --await --sign --node ${PROXY}

    update-config MULTI_TRANSFER ${NEW_MULTI_TRANSFER_ADDR}
}
