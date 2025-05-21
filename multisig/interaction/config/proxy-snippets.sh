deployBridgeProxy() {
    CHECK_VARIABLES PROXY_WASM MULTI_TRANSFER

    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${PROXY_WASM} \
    --args A:${MULTI_TRANSFER} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Proxy contract address: ${CONTRACT_ADDRESS}"
    update-config BRIDGE_PROXY ${CONTRACT_ADDRESS}
}

setBridgedTokensWrapperOnSCProxy() {
    CHECK_VARIABLES BRIDGE_PROXY BRIDGED_TOKENS_WRAPPER

    operator sc invoke ${BRIDGE_PROXY} setBridgedTokensWrapperAddress --key-file=${ALICE} \
    --args A:${BRIDGED_TOKENS_WRAPPER} \
    --await --sign --node ${PROXY}
}

setMultiTransferOnSCProxy() {
    CHECK_VARIABLES BRIDGE_PROXY MULTI_TRANSFER

    operator sc invoke ${BRIDGE_PROXY} setMultiTransferAddress --key-file=${ALICE} \
    --args A:${MULTI_TRANSFER} \
    --await --sign --node ${PROXY}
}

setKdaSafeOnSCProxy() {
    CHECK_VARIABLES BRIDGE_PROXY SAFE

    operator sc invoke ${BRIDGE_PROXY} setKdaSafeAddress --key-file=${ALICE} \
    --args A:${SAFE} \
    --await --sign --node ${PROXY}
}

deployBridgeProxyForUpgrade() {
    CHECK_VARIABLES PROXY_WASM MULTI_TRANSFER

    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${PROXY_WASM} \
    --args A:${MULTI_TRANSFER} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "New proxy contract address: ${CONTRACT_ADDRESS}"
    
    # Store for later use in upgradeBridgeProxyContract
    NEW_BRIDGE_PROXY_ADDR=${CONTRACT_ADDRESS}
}

upgradeBridgeProxyContract() {
    CHECK_VARIABLES MULTISIG BRIDGE_PROXY NEW_BRIDGE_PROXY_ADDR

    operator sc invoke ${MULTISIG} upgradeChildContractFromSource --key-file=${ALICE} \
    --args A:${BRIDGE_PROXY} --args A:${NEW_BRIDGE_PROXY_ADDR} --args bool:false \
    --await --sign --node ${PROXY}

    update-config BRIDGE_PROXY ${NEW_BRIDGE_PROXY_ADDR}
}
