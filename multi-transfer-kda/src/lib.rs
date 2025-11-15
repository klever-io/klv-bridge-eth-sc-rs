#![no_std]

use klever_sc::{imports::*, storage::StorageKey};

use eth_address::EthAddress;
use klever_sc_modules::only_admin;
use transaction::{EthTransaction, PaymentsVec, Transaction, TxNonce};

pub mod bridged_tokens_wrapper_proxy;
pub mod kda_safe_proxy;
pub mod multi_transfer_proxy;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;
const CHAIN_SPECIFIC_TO_UNIVERSAL_TOKEN_MAPPING: &[u8] = b"chainSpecificToUniversalMapping";

#[klever_sc::contract]
pub trait MultiTransferKda:
    tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + only_admin::OnlyAdminModule
{
    #[init]
    fn init(&self) {
        self.max_tx_batch_size()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);
        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(1);
        self.last_batch_id().set_if_empty(1);
    }

    #[upgrade]
    fn upgrade(&self) {
        self.max_tx_batch_size()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);
        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(1);
        self.last_batch_id().set_if_empty(1);
    }

    #[only_admin]
    #[endpoint(batchTransferKdaToken)]
    fn batch_transfer_kda_token(
        &self,
        batch_id: u64,
        transfers: MultiValueEncoded<EthTransaction<Self::Api>>,
    ) {
        let mut valid_payments_list = ManagedVec::new();
        let mut valid_tx_list = ManagedVec::new();
        let mut refund_tx_list = ManagedVec::new();

        let safe_address = self.kda_safe_contract_address().get();

        for eth_tx in transfers {
            // First, convert ETH amount to KDA amount using kda-safe's conversion (single source of truth)
            let kda_amount: BigUint = self
                .tx()
                .to(safe_address.clone())
                .typed(kda_safe_proxy::KDASafeProxy)
                .convert_eth_to_kda_amount_endpoint(&eth_tx.token_id, &eth_tx.amount)
                .returns(ReturnsResult)
                .sync_call();

            // Then, get tokens with the already-converted KDA amount
            let is_success: bool = self
                .tx()
                .to(safe_address.clone())
                .typed(kda_safe_proxy::KDASafeProxy)
                .get_tokens(&eth_tx.token_id, &kda_amount)
                .returns(ReturnsResult)
                .sync_call();

            require!(is_success, "Invalid token or amount");

            let mut must_refund = false;
            if eth_tx.to.is_zero() || self.blockchain().is_smart_contract(&eth_tx.to) {
                self.transfer_failed_invalid_destination(batch_id, eth_tx.tx_nonce);
                must_refund = true;
            } else if self.is_above_max_amount(&eth_tx.token_id, &kda_amount) {
                self.transfer_over_max_amount(batch_id, eth_tx.tx_nonce);
                must_refund = true;
            }

            if must_refund {
                let refund_tx = self.convert_to_refund_tx(eth_tx);
                refund_tx_list.push(refund_tx);

                continue;
            }

            // emit event before the actual transfer so we don't have to save the tx_nonces as well
            // Use KDA amount since that's what was actually minted/transferred
            self.transfer_performed_event(
                batch_id,
                eth_tx.from.clone(),
                eth_tx.to.clone(),
                eth_tx.token_id.clone(),
                kda_amount.clone(),
                eth_tx.tx_nonce,
            );

            valid_tx_list.push(eth_tx.clone());
            // Use KDA amount for payment since that's the converted amount
            valid_payments_list.push(KdaTokenPayment::new(eth_tx.token_id, 0, kda_amount));
        }

        let payments_after_wrapping = self.wrap_tokens(valid_payments_list);
        self.distribute_payments(valid_tx_list, payments_after_wrapping);

        self.add_multiple_tx_to_batch(&refund_tx_list);
    }

    #[only_admin]
    #[endpoint(moveRefundBatchToSafe)]
    fn move_refund_batch_to_safe(&self) {
        let opt_current_batch = self.get_first_batch_any_status();
        match opt_current_batch {
            OptionalValue::Some(current_batch) => {
                let first_batch_id = self.first_batch_id().get();
                let mut first_batch = self.pending_batches(first_batch_id);

                self.clear_first_batch(&mut first_batch);
                let (_batch_id, all_tx_fields) = current_batch.into_tuple();
                let mut refund_batch = ManagedVec::new();
                let mut refund_payments = ManagedVec::new();

                for tx_fields in all_tx_fields {
                    let (_, _, _, _, token_identifier, amount) =
                        tx_fields.clone().into_tuple();

                        refund_batch.push(Transaction::from(tx_fields));
                        refund_payments.push(KdaTokenPayment::new(token_identifier, 0, amount));
                }

                let kda_safe_addr = self.kda_safe_contract_address().get();
                self.tx()
                    .to(kda_safe_addr)
                    .typed(kda_safe_proxy::KDASafeProxy)
                    .add_refund_batch(refund_batch)
                    .payment(refund_payments)
                    .sync_call();
            }
            OptionalValue::None => {}
        }
    }

    #[only_admin]
    #[endpoint(setWrappingContractAddress)]
    fn set_wrapping_contract_address(&self, opt_new_address: OptionalValue<ManagedAddress>) {
        match opt_new_address {
            OptionalValue::Some(sc_addr) => {
                require!(
                    self.blockchain().is_smart_contract(&sc_addr),
                    "Invalid unwrapping contract address"
                );

                self.wrapping_contract_address().set(&sc_addr);
            }
            OptionalValue::None => self.wrapping_contract_address().clear(),
        }
    }

    #[only_admin]
    #[endpoint(addUnprocessedRefundTxToBatch)]
    fn add_unprocessed_refund_tx_to_batch(&self, tx_id: u64) {
        let refund_tx = self.unprocessed_refund_txs(tx_id).get();
        let mut refund_tx_list = ManagedVec::new();
        refund_tx_list.push(refund_tx);
        self.add_multiple_tx_to_batch(&refund_tx_list);

        self.unprocessed_refund_txs(tx_id).clear();
    }

    #[only_admin]
    #[endpoint(setKdaSafeContractAddress)]
    fn set_kda_safe_contract_address(&self, opt_new_address: OptionalValue<ManagedAddress>) {
        match opt_new_address {
            OptionalValue::Some(sc_addr) => {
                self.kda_safe_contract_address().set(&sc_addr);
            }
            OptionalValue::None => self.kda_safe_contract_address().clear(),
        }
    }

    #[only_admin]
    #[endpoint(changeContractName)]
    fn change_contract_name(&self, new_name: ManagedBuffer) {
        self.send().set_account_name(new_name);
    }

    // private

    fn get_universal_token(&self, eth_tx: EthTransaction<Self::Api>) -> TokenIdentifier {
        let mut storage_key = StorageKey::new(CHAIN_SPECIFIC_TO_UNIVERSAL_TOKEN_MAPPING);
        storage_key.append_item(&eth_tx.token_id);

        let chain_specific_to_universal_token_mapper: SingleValueMapper<
            TokenIdentifier,
            ManagedAddress,
        > = SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            self.wrapping_contract_address().get(),
            storage_key,
        );
        if chain_specific_to_universal_token_mapper.is_empty() {
            eth_tx.token_id
        } else {
            chain_specific_to_universal_token_mapper.get()
        }
    }

    fn convert_to_refund_tx(&self, eth_tx: EthTransaction<Self::Api>) -> Transaction<Self::Api> {
        Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: eth_tx.tx_nonce,
            from: eth_tx.from.as_managed_buffer().clone(),
            to: eth_tx.to.as_managed_buffer().clone(),
            token_identifier: eth_tx.token_id,
            amount: eth_tx.amount,
            is_refund_tx: true,
        }
    }

    fn wrap_tokens(&self, payments: PaymentsVec<Self::Api>) -> PaymentsVec<Self::Api> {
        if self.wrapping_contract_address().is_empty() {
            return payments;
        }

        let bridged_tokens_wrapper_addr = self.wrapping_contract_address().get();
        self.tx()
            .to(bridged_tokens_wrapper_addr)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .wrap_tokens()
            .payment(payments)
            .returns(ReturnsResult)
            .sync_call()
    }

    fn distribute_payments(
        &self,
        transfers: ManagedVec<EthTransaction<Self::Api>>,
        payments: PaymentsVec<Self::Api>,
    ) {
        for (eth_tx, p) in transfers.iter().zip(payments.iter()) {
            self.tx()
                .to(&eth_tx.to)
                .single_kda(&p.token_identifier, 0, &p.amount)
                .transfer();
        }
    }

    // storage
    #[view(getWrappingContractAddress)]
    #[storage_mapper("wrappingContractAddress")]
    fn wrapping_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getKdaSafeContractAddress)]
    #[storage_mapper("kdaSafeContractAddress")]
    fn kda_safe_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("unprocessedRefundTxs")]
    fn unprocessed_refund_txs(&self, tx_id: u64) -> SingleValueMapper<Transaction<Self::Api>>;

    // events

    #[event("transferPerformedEvent")]
    fn transfer_performed_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] from: EthAddress<Self::Api>,
        #[indexed] to: ManagedAddress,
        #[indexed] token_id: TokenIdentifier,
        #[indexed] amount: BigUint,
        #[indexed] tx_id: TxNonce,
    );

    #[event("transferFailedInvalidDestination")]
    fn transfer_failed_invalid_destination(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedInvalidToken")]
    fn transfer_failed_invalid_token(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedFrozenDestinationAccount")]
    fn transfer_failed_frozen_destination_account(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
    );

    #[event("transferOverMaxAmount")]
    fn transfer_over_max_amount(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("unprocessedRefundTxs")]
    fn unprocessed_refund_txs_event(&self, #[indexed] tx_id: u64);
}
