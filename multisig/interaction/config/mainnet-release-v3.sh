deployMultisigMainnetV3() {
    CHECK_VARIABLES RELAYER_ADDR_0 RELAYER_ADDR_1 RELAYER_ADDR_2 RELAYER_ADDR_3 \
    RELAYER_ADDR_4 RELAYER_ADDR_5 RELAYER_ADDR_6 RELAYER_ADDR_7 RELAYER_ADDR_8 \
    RELAYER_ADDR_9 SAFE MULTI_TRANSFER RELAYER_REQUIRED_STAKE SLASH_AMOUNT QUORUM MULTISIG_WASM

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^6" | bc)
    
    SC_RESULT=$(eval operator sc create --key-file=${ALICE} --wasm ${MULTISIG_WASM} \
    --args A:${SAFE} --args A:${MULTI_TRANSFER} \
    --args n:${MIN_STAKE} --args n:${SLASH_AMOUNT} --args n:${QUORUM} \
    --args A:${RELAYER_ADDR_0} --args A:${RELAYER_ADDR_1} --args A:${RELAYER_ADDR_2} --args A:${RELAYER_ADDR_3} \
    --args A:${RELAYER_ADDR_4} --args A:${RELAYER_ADDR_5} --args A:${RELAYER_ADDR_6} --args A:${RELAYER_ADDR_7} \
    --args A:${RELAYER_ADDR_8} --args A:${RELAYER_ADDR_9} \
    --await --result-only --sign --node ${PROXY})

    check_result ${SC_RESULT}

    CONTRACT_ADDRESS=$(jq '.logs.events[] | select(.identifier=="SCDeploy") | .address' <<< "${SC_RESULT}")
    CONTRACT_ADDRESS=$(echo ${CONTRACT_ADDRESS} | tr -d '"')

    echo ""
    echo "Multisig contract address: ${CONTRACT_ADDRESS}"
    update-config MULTISIG ${CONTRACT_ADDRESS}
}