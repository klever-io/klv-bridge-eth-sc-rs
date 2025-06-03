# 1. deployBridgedTokensWrapper
# 3. setLocalRolesBridgedTokensWrapper # - keep in mind we need to do this with the token owner
# 4. addWrappedToken
# 5. whitelistToken
# If the SC already exists, skip the first step
# If we want to add another chain, do only the last step

deployBridgedTokensWrapper() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER_WASM
    
    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${BRIDGED_TOKENS_WRAPPER_WASM} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Bridged tokens wrapper SC: ${CONTRACT_ADDRESS}"
    update-config BRIDGED_TOKENS_WRAPPER ${CONTRACT_ADDRESS}
}

setLocalRolesBridgedTokensWrapper() {
    CHECK_VARIABLES UNIVERSAL_TOKEN BRIDGED_TOKENS_WRAPPER
    
    # Trigger 6: AddRole - Add a permission role to the asset
    local TRIGGER_ADD_ROLE=6

    operator kda trigger ${TRIGGER_ADD_ROLE} --kdaID=${UNIVERSAL_TOKEN} --addRolesMint=${BRIDGED_TOKENS_WRAPPER} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

unsetLocalRolesBridgedTokensWrapper() {
    CHECK_VARIABLES UNIVERSAL_TOKEN BRIDGED_TOKENS_WRAPPER
    
    # Trigger 7: RemoveRole - Remove a permission role of the asset
    local TRIGGER_REMOVE_ROLE=7

    operator kda trigger ${TRIGGER_REMOVE_ROLE} --kdaID=${UNIVERSAL_TOKEN} --receiver=${BRIDGED_TOKENS_WRAPPER} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

addWrappedToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER UNIVERSAL_TOKEN NR_DECIMALS_UNIVERSAL

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} addWrappedToken --key-file=${ALICE} \
    --args String:${UNIVERSAL_TOKEN} --args u32:${NR_DECIMALS_UNIVERSAL} \
    --await --sign --node ${PROXY}
}

removeWrappedToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER UNIVERSAL_TOKEN

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} removeWrappedToken --key-file=${ALICE} \
    --args String:${UNIVERSAL_TOKEN} \
    --await --sign --node ${PROXY}
}

wrapper-whitelistToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER CHAIN_SPECIFIC_TOKEN NR_DECIMALS_CHAIN_SPECIFIC UNIVERSAL_TOKEN

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} whitelistToken --key-file=${ALICE} \
    --args String:${CHAIN_SPECIFIC_TOKEN} --args u32:${NR_DECIMALS_CHAIN_SPECIFIC} --args String:${UNIVERSAL_TOKEN} \
    --await --sign --node ${PROXY}
}

wrapper-blacklistToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER CHAIN_SPECIFIC_TOKEN UNIVERSAL_TOKEN

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} blacklistToken --key-file=${ALICE} \
    --args String:${CHAIN_SPECIFIC_TOKEN} \
    --await --sign --node ${PROXY}
}

wrapper-updateWrappedToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER UNIVERSAL_TOKEN NR_DECIMALS_UNIVERSAL

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} updateWrappedToken --key-file=${ALICE} \
    --args String:${UNIVERSAL_TOKEN} --args u32:${NR_DECIMALS_UNIVERSAL} \
    --await --sign --node ${PROXY}
}

wrapper-updateWhitelistedToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER CHAIN_SPECIFIC_TOKEN NR_DECIMALS_CHAIN_SPECIFIC

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} updateWhitelistedToken --key-file=${ALICE} \
    --args String:${CHAIN_SPECIFIC_TOKEN} --args u32:${NR_DECIMALS_CHAIN_SPECIFIC} \
    --await --sign --node ${PROXY}
}

wrapper-unpause() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} unpause --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

wrapper-pause() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} pause --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

wrapper-pauseV2() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER_v2

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER_v2} pause --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

wrapper-upgrade() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER BRIDGED_TOKENS_WRAPPER_WASM

    operator sc upgrade ${BRIDGED_TOKENS_WRAPPER} --key-file=${ALICE} \
    --wasm ${BRIDGED_TOKENS_WRAPPER_WASM} \
    --await --sign --node ${PROXY}
    
    echo ""
    echo "Bridged tokens wrapper upgraded successfully at address: ${BRIDGED_TOKENS_WRAPPER}"
}
