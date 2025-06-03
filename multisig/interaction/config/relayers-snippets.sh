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
    RELAYER_WALLET0 RELAYER_WALLET1 RELAYER_WALLET2 RELAYER_WALLET3 RELAYER_WALLET4 \
    RELAYER_WALLET5 RELAYER_WALLET6 RELAYER_WALLET7 RELAYER_WALLET8 RELAYER_WALLET9

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^6" | bc)
    
    echo "Staking relayer 0..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET0} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 1..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET1} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 2..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET2} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 3..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET3} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 4..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET4} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 5..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET5} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 6..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET6} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 7..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET7} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 8..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET8} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
    echo "---------------------------------------------------------"
    
    echo "Staking relayer 9..."
    operator sc invoke ${MULTISIG} stake --key-file=${RELAYER_WALLET9} \
    --values "KLV=${MIN_STAKE}" \
    --await --sign --node ${PROXY}
}