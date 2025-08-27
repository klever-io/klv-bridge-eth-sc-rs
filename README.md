# KleverBlockchain-Ethereum Bridge

## Abstract
In this repository, you will find smart contracts on the Klever Blockchain side for the KleverBlockchain-Ethereum bridge. For setup details, take a look [here](docs/setup.md). Documentation for relayers can be found [here](docs/relayer.md).  

Although cryptocurrency is the future, there are still many challenges to overcome. One of them is providing an easy way to manage multiple different types of coins. Although we already have some exchange services, they can be a bit difficult to use and require many steps to perform the actual transfer, and they require you to trust a centralized third party and/or a centralized ERC20 token smart contract. With this suite of smart contracts, we aim to provide a decentralized way of transferring tokens between Klever Blockchain and Ethereum.  

For this to be truly decentralized, we're using a multi-sig smart contract. We're going to have a set of relayers that validate the transaction. You could think of it as a mini-blockchain, whose sole purpose is to handle cross-chain transactions.  

To be able to transfer the tokens, we make use of a concept known as "Wrapped Tokens". You don't transfer native KLV or ETH tokens, but instead, you lock the native tokens in a contract and generate "Wrapped" versions of them. On Klever Blockchain, we are going to use KDA for this purpose. On Ethereum, we're most likely going to use something like ERC20.  

Now you might ask, how is this any different? There already is an ERC20-style contract on Ethereum for KLV! The main difference is you won't have to go through exchange services at all. And there's one more important difference: This is decentralized. There isn't a single contract owner doing all the work.  We have a set of trusted accounts that will handle the transactions.  

But why should _you_ trust them? We use the "Proof of Stake" concept: each of them will have to stake a certain amount of KLV to be able to become a relayer, just like validators on the Klever Blockchain blockchain. If any of them misbehaves, their stake will be "slashed" and they'll lose quite a bit of money as a result.

## Wrapped Tokens

As said above, we're going to use KDA to implement wrapped tokens, but how is the wrapping done? For that, we have the `KlvKdaSwap`, a very simple SC, whose only purpose is to exchange 1:1 native KLV to WrappedKLV KDA tokens. You can also do the reverse operation at any time, which is known as unwrapping.  

One important thing to note is you can never unwrap your WrappedETH while on the Klever Blockchain blockchain, as that is not a native Klever Blockchain token. You will only be able to unwrap them by transferring them to one of your Ethereum accounts and then unwrapping them there.  


## Klever Blockchain -> Ethereum transaction

To be able to send KLV to an Ethereum account, we first have to wrap the tokens through the `KlvKdaSwap` contract.   

Then you can create a transaction by making a smart contract call to the `KdaSafe` SC with the tokens you want to transfer and the receiver's address. The tokens will be locked in the contract until the transaction is processed. If the transaction is successful, the tokens on the Klever Blockchain side will be burned. If the transaction fails for whatever reason, you will get your tokens back.  

Note that not all tokens will be transferred, part of them will be deducted for transaction fees.  

## Ethereum -> Klever Blockchain transaction

To be able to transfer your tokens back, you will likely have to use an ERC20 contract on the Ethereum blockchain. Once your transaction has been processed on that side, our relayers will simply transfer the tokens back to your Klever Blockchain account, through the `MultiTransferKda` SC. No additional fees have to be paid for this kind of transaction.  

## Conclusion

And that sums up the Klever Blockchain-Ethereum bridge. It's open source, so if you're interested in the details, you can always check out the implementation.

