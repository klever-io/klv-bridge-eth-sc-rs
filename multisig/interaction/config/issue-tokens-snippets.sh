issueUniversalToken() {
    CHECK_VARIABLES UNIVERSAL_TOKEN_DISPLAY_NAME UNIVERSAL_TOKEN_TICKER NR_DECIMALS_UNIVERSAL

    # KDA Asset Type 0: Fungible token
    local KDA_TYPE_FUNGIBLE=0

    operator kda create ${KDA_TYPE_FUNGIBLE} --canMint=true --canBurn=true --canAddRoles=true --initialSupply=0 \
    --name=${UNIVERSAL_TOKEN_DISPLAY_NAME} --ticker=${UNIVERSAL_TOKEN_TICKER} --precision=${NR_DECIMALS_UNIVERSAL} \
    --key-file=${ALICE} --await --sign --node ${PROXY}
}

issueChainSpecificToken() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN_DISPLAY_NAME CHAIN_SPECIFIC_TOKEN_TICKER \
    NR_DECIMALS_CHAIN_SPECIFIC UNIVERSAL_TOKENS_ALREADY_MINTED
    
    VALUE_TO_MINT=$(echo "scale=0; $UNIVERSAL_TOKENS_ALREADY_MINTED*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)

    # KDA Asset Type 0: Fungible token
    local KDA_TYPE_FUNGIBLE=0

    operator kda create ${KDA_TYPE_FUNGIBLE} --canMint=true --canBurn=true --canAddRoles=true --initialSupply=${VALUE_TO_MINT} \
    --name=${CHAIN_SPECIFIC_TOKEN_DISPLAY_NAME} --ticker=${CHAIN_SPECIFIC_TOKEN_TICKER} --precision=${NR_DECIMALS_CHAIN_SPECIFIC} \
    --key-file=${ALICE} --await --sign --node ${PROXY}
}

transferToSC() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER CHAIN_SPECIFIC_TOKEN

    VALUE_TO_MINT=$(echo "scale=0; $UNIVERSAL_TOKENS_ALREADY_MINTED*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)

    operator sc invoke ${BRIDGED_TOKENS_WRAPPER} depositLiquidity --values ${CHAIN_SPECIFIC_TOKEN}=${VALUE_TO_MINT} \
     --key-file=${ALICE} --await --sign --node ${PROXY}
}

setMintRole() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN ALICE_ADDRESS
    
    # Trigger 6: AddRole - Add a permission role to the asset
    local TRIGGER_ADD_ROLE=6

    operator kda trigger ${TRIGGER_ADD_ROLE} --kdaID=${CHAIN_SPECIFIC_TOKEN} --addRolesMint=${ALICE_ADDRESS} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

unSetMintRole() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN
    
    # Trigger 7: RemoveRole - Remove a permission role of the asset
    local TRIGGER_REMOVE_ROLE=7

    operator kda trigger ${TRIGGER_REMOVE_ROLE} --kdaID=${CHAIN_SPECIFIC_TOKEN} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}

mint() {
    CHECK_VARIABLES NR_DECIMALS_CHAIN_SPECIFIC CHAIN_SPECIFIC_TOKEN ALICE_ADDRESS
    read -p "Amount to mint(without decimals): " AMOUNT_TO_MINT
    VALUE_TO_MINT=$(echo "scale=0; $AMOUNT_TO_MINT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)
    
    # Trigger 0: Mint - Directly mint assets in the receiver
    local TRIGGER_MINT=0
    
    operator kda trigger ${TRIGGER_MINT} --kdaID=${CHAIN_SPECIFIC_TOKEN} --receiver=${ALICE_ADDRESS} --amount=${VALUE_TO_MINT} --key-file=${ALICE} \
    --await --sign --node ${PROXY}
}