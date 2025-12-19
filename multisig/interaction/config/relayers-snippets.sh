addBoardMember() {
    CHECK_VARIABLES MULTISIG

    read -p "Relayer address: " RELAYER_ADDR
    operator sc invoke ${MULTISIG} addBoardMember --key-file=${ALICE} \
    --args A:${RELAYER_ADDR} \
    --await --sign --node ${PROXY}
}

removeBoardMember() {
    CHECK_VARIABLES MULTISIG

    read -p "Relayer address: " RELAYER_ADDR
    operator sc invoke ${MULTISIG} removeUser --key-file=${ALICE} \
    --args A:${RELAYER_ADDR} \
    --await --sign --node ${PROXY}
}

unstake() {
    CHECK_VARIABLES MULTISIG RELAYER_REQUIRED_STAKE

    read -p "Relayer address: " RELAYER_ADDR
    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^6" | bc)
    
    operator sc invoke ${MULTISIG} unstake --key-file="./walletsRelay/${RELAYER_ADDR}.pem" \
    --args n:${MIN_STAKE} \
    --await --sign --node ${PROXY}
}

stakeRelayers() {
    CHECK_VARIABLES MULTISIG RELAYER_REQUIRED_STAKE \
    RELAYER_WALLET0 RELAYER_WALLET1 RELAYER_WALLET2

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^6" | bc)
    
    echo "Staking relayer 0..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET0} \
    --values "KFI=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 1..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET1} \
    --values "KFI=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 2..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET2} \
    --values "KFI=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
}