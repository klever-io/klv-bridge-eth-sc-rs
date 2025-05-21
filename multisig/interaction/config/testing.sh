deployFaucet() {
    CHECK_VARIABLES FAUCET_WASM ALICE

    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${FAUCET_WASM} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Faucet contract address: ${CONTRACT_ADDRESS}"
    update-config FAUCET ${CONTRACT_ADDRESS}
}

setMintRoleForUniversalToken() {
    CHECK_VARIABLES UNIVERSAL_TOKEN ALICE_ADDRESS
    
    # Trigger 6: AddRole - Add a permission role to the asset
    local TRIGGER_ADD_ROLE=6

    operator kda trigger ${TRIGGER_ADD_ROLE} --kdaID=${UNIVERSAL_TOKEN} --addRolesMint=${ALICE_ADDRESS} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

mintAndDeposit() {
    CHECK_VARIABLES FAUCET UNIVERSAL_TOKEN

    VALUE_TO_MINT=1000000000000000000
    
    # Trigger 0: Mint - Directly mint assets in the receiver
    local TRIGGER_MINT=0
    
    operator kda trigger ${TRIGGER_MINT} --kdaID=${UNIVERSAL_TOKEN} --receiver=${ALICE_ADDRESS} --amount=${VALUE_TO_MINT} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
    
    operator sc invoke ${FAUCET} deposit --values ${UNIVERSAL_TOKEN}=${VALUE_TO_MINT} \
     --key-file=${ALICE} --await --sign --node ${PROXY}
}

unSetMintRoleForUniversalToken() {
    CHECK_VARIABLES UNIVERSAL_TOKEN
    
    # Trigger 7: RemoveRole - Remove a permission role of the asset
    local TRIGGER_REMOVE_ROLE=7

    operator kda trigger ${TRIGGER_REMOVE_ROLE} --kdaID=${UNIVERSAL_TOKEN} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

deployTestCaller() {
    CHECK_VARIABLES TEST_CALLER_WASM ALICE

    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${TEST_CALLER_WASM} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Test caller contract address: ${CONTRACT_ADDRESS}"
    update-config TEST_CALLER ${CONTRACT_ADDRESS}
}
