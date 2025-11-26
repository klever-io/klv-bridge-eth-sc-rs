#![allow(unused)]

use bridged_tokens_wrapper::ProxyTrait as _;
use kda_safe::{KDASafe, ProxyTrait as _};
use multi_transfer_kda::{
    bridged_tokens_wrapper_proxy, kda_safe_proxy, multi_transfer_proxy, ProxyTrait as _,
};

use klever_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        Empty, TopEncode,
    },
    contract_base::ManagedSerializer,
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress,
        ManagedBuffer, ManagedByteArray, ManagedOption, ManagedVec, MultiValueEncoded,
        ReturnsNewManagedAddress, ReturnsResult, TestAddress, TestSCAddress, TestTokenIdentifier,
        TokenIdentifier,
    },
};
use klever_sc_modules::pause::ProxyTrait;
use klever_sc_scenario::{
    api::{StaticApi, VMHooksApi, VMHooksApiBackend}, imports::KleverscPath, klever_chain_vm::types::KDALocalRole, managed_address, scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext}, scenario_model::*, ContractInfo, DebugApi, ExpectError, ExpectValue, ScenarioTxRun, ScenarioWorld
};

use eth_address::*;
use token_module::ProxyTrait as _;
use transaction::{CallData, EthTransaction, TxBatchSplitInFields};

const UNIVERSAL_TOKEN_IDENTIFIER: &[u8] = b"UNIV-abc123";
const BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE-123456";
const WRAPPED_TOKEN_ID: &[u8] = b"WRAPPED-123456";
const TOKEN_ID: &[u8] = b"TOKEN";

const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 100_000_000;
const ERROR: u64 = 57;

const MULTI_TRANSFER_CODE_PATH: KleverscPath = KleverscPath::new("output/multi-transfer-kda.kleversc.json");
const KDA_SAFE_CODE_PATH: KleverscPath = KleverscPath::new("../kda-safe/output/kda-safe.kleversc.json");
const BRIDGED_TOKENS_WRAPPER_CODE_PATH: KleverscPath =
    KleverscPath::new("../bridged-tokens-wrapper/output/bridged-tokens-wrapper.kleversc.json");
const PRICE_AGGREGATOR_CODE_PATH: KleverscPath =
    KleverscPath::new("../price-aggregator/price-aggregator.kleversc.json");

const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const KDA_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("kda-safe");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");
const PRICE_AGGREGATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price-aggregator");

const ORACLE_ADDRESS: TestAddress = TestAddress::new("oracle");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER1_ADDRESS: TestAddress = TestAddress::new("user1");
const USER2_ADDRESS: TestAddress = TestAddress::new("user2");

const KDA_SAFE_ETH_TX_GAS_LIMIT: u64 = 150_000;
const MAX_AMOUNT: u64 = 30_000_000_000u64;

const BALANCE: &str = "2,000,000";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        MULTI_TRANSFER_CODE_PATH,
        multi_transfer_kda::ContractBuilder,
    );

    blockchain.register_contract(KDA_SAFE_CODE_PATH, kda_safe::ContractBuilder);

    blockchain.register_contract(
        BRIDGED_TOKENS_WRAPPER_CODE_PATH,
        bridged_tokens_wrapper::ContractBuilder,
    );

    blockchain
}

type MultiTransferContract = ContractInfo<multi_transfer_kda::Proxy<StaticApi>>;
type KdaSafeContract = ContractInfo<kda_safe::Proxy<StaticApi>>;
type BridgedTokensWrapperContract = ContractInfo<bridged_tokens_wrapper::Proxy<StaticApi>>;

struct MultiTransferTestState {
    world: ScenarioWorld,
}

impl MultiTransferTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(0)
            .kda_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 1001u64)
            .kda_balance(TokenIdentifier::from(TOKEN_ID), MAX_AMOUNT)
            .kda_balance(TokenIdentifier::from(WRAPPED_TOKEN_ID), 1001u64)
            .kda_balance(TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER), 1001u64)
            .account(USER1_ADDRESS)
            .nonce(0)
            .account(USER2_ADDRESS)
            .nonce(0);

        let roles = vec![
            "KDARoleMint".to_string(),
        ];

        world
            .account(KDA_SAFE_ADDRESS)
            .kda_roles(TokenIdentifier::from(BRIDGE_TOKEN_ID), roles.clone())
            .kda_roles(TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER), roles.clone())
            .kda_roles(TokenIdentifier::from(WRAPPED_TOKEN_ID), roles.clone())
            .code(KDA_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS);

        let kda_safe_address = AddressValue::from(KDA_SAFE_ADDRESS);

        world.set_kda_can_burn(
            managed_address!(&kda_safe_address.to_address()),
            UNIVERSAL_TOKEN_IDENTIFIER,
            0,
            true,
        );

        world.set_kda_can_burn(
            managed_address!(&kda_safe_address.to_address()),
            WRAPPED_TOKEN_ID,
            0,
            true,
        );

        world.set_kda_can_burn(
            managed_address!(&kda_safe_address.to_address()),
            BRIDGE_TOKEN_ID,
            0,
            true,
        );

        Self { world }
    }

    fn multi_transfer_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferKdaProxy)
            .init()
            .code(MULTI_TRANSFER_CODE_PATH)
            .new_address(MULTI_TRANSFER_ADDRESS)
            .run();
        self
    }

    fn safe_deploy(&mut self, price_aggregator_contract_address: Address) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .upgrade(
                ManagedAddress::zero(),
                MULTI_TRANSFER_ADDRESS.to_address(),
                KDA_SAFE_ETH_TX_GAS_LIMIT,
            )
            .code(KDA_SAFE_CODE_PATH)
            .run();

        self
    }

    fn bridged_tokens_wrapper_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .init()
            .code(BRIDGED_TOKENS_WRAPPER_CODE_PATH)
            .new_address(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .run();

        self
    }

    fn config_multi_transfer(&mut self) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferKdaProxy)
            .add_admin(OWNER_ADDRESS.eval_to_array())
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .add_admin(OWNER_ADDRESS.eval_to_array())
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferKdaProxy)
            .set_wrapping_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferKdaProxy)
            .set_kda_safe_contract_address(OptionalValue::Some(KDA_SAFE_ADDRESS.to_address()))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_multi_transfer_contract_address(OptionalValue::Some(
                MULTI_TRANSFER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_kda_bytes("BRIDGE-123456"),
                "BRIDGE",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(KDA_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();

        // Configure decimals for BRIDGE token (18 decimals on Ethereum, 6 on Klever)
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_token_decimals(TokenIdentifier::from_kda_bytes("BRIDGE-123456"), 18u32, 6u32)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferKdaProxy)
            .set_max_bridged_amount(BRIDGE_TOKEN_ID, MAX_AMOUNT - 1)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_kda_bytes("TOKEN"),
                "TOKEN",
                false,
                true,
                BigUint::from(MAX_AMOUNT),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(KDA_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .single_kda(
                &TokenIdentifier::from_kda_bytes("TOKEN"),
                0,
                &BigUint::from(MAX_AMOUNT),
            )
            .run();

        // Configure decimals for TOKEN (8 decimals on both ETH and KDA - no conversion needed)
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_token_decimals(TokenIdentifier::from_kda_bytes("TOKEN"), 8u32, 8u32)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferKdaProxy)
            .set_max_bridged_amount(TOKEN_ID, MAX_AMOUNT - 1)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_kda_bytes("WRAPPED-123456"),
                "BRIDGE2",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(KDA_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();

        // Configure decimals for WRAPPED token (8 decimals on both ETH and KDA - no conversion needed)
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_token_decimals(TokenIdentifier::from_kda_bytes("WRAPPED-123456"), 8u32, 8u32)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unpause_endpoint()
            .run();
    }

    fn config_bridged_tokens_wrapper(&mut self) {

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .add_admin(OWNER_ADDRESS.eval_to_array())
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_kda_bytes("UNIV-abc123"),
                "BRIDGE1",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(KDA_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();
        
        self.world.set_kda_balance(
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            WRAPPED_TOKEN_ID,
            BigUint::from(10_000_000u64),
        );

        let roles = vec![
            "KDARoleMint".to_string(),
        ];

        self.world.set_kda_local_roles(
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            UNIVERSAL_TOKEN_IDENTIFIER,
            roles,
        );

        let bridged_tokens_address = AddressValue::from(BRIDGED_TOKENS_WRAPPER_ADDRESS);

        self.world.set_kda_can_burn(
            managed_address!(&bridged_tokens_address.to_address()),
            UNIVERSAL_TOKEN_IDENTIFIER,
            0,
            true,
        );

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_eth_tx_gas_limit(0u64)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .init_supply_mint_burn(
                UNIVERSAL_TOKEN_IDENTIFIER,
                BigUint::from(600_000u64),
                BigUint::from(0u64),
            )
            .run();
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_kda_bytes("WRAPPED-123456"),
                "BRIDGE2",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(KDA_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .init_supply_mint_burn(
                WRAPPED_TOKEN_ID,
                BigUint::from(600_000u64),
                BigUint::from(0u64),
            )
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_bridged_tokens_wrapper_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .add_wrapped_token(TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER), 18u32)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .whitelist_token(
                TokenIdentifier::from(WRAPPED_TOKEN_ID),
                18u32,
                TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            )
            .run();
    }

    fn check_balances_on_safe(
        &mut self,
        token_id: &[u8],
        total_supply: BigUint<StaticApi>,
        total_minted: BigUint<StaticApi>,
        total_burned: BigUint<StaticApi>,
    ) {
        let actual_total_supply = self
            .world
            .query()
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .total_balances(token_id)
            .returns(ReturnsResult)
            .run();

        assert_eq!(
            actual_total_supply, total_supply,
            "Total supply balance is wrong"
        );
        let actual_total_burned = self
            .world
            .query()
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .burn_balances(token_id)
            .returns(ReturnsResult)
            .run();

        assert_eq!(
            actual_total_burned, total_burned,
            "Total burned balance is wrong"
        );

        let actual_total_minted = self
            .world
            .query()
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .mint_balances(token_id)
            .returns(ReturnsResult)
            .run();

        assert_eq!(
            actual_total_minted, total_minted,
            "Total minted balance is wrong"
        );
    }

    fn deploy_contracts(&mut self) {
        self.multi_transfer_deploy();
        self.safe_deploy(Address::zero());
        self.bridged_tokens_wrapper_deploy();
    }
}

#[test]
fn basic_transfer_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let call_data = ManagedBuffer::from(b"add");
    call_data
        .clone()
        .concat(ManagedBuffer::from(GAS_LIMIT.to_string()));
    call_data.clone().concat(ManagedBuffer::default());

    // Use TOKEN which has same decimals (8 on both ETH and KDA) - no conversion
    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(TOKEN_ID),
        amount: token_amount.clone(),
        converted_amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .batch_transfer_kda_token(1u32, transfers)
        .run();

    // With same decimals, amount should remain 500
    state
        .world
        .check_account(USER1_ADDRESS)
        .kda_balance(TokenIdentifier::from(TOKEN_ID), token_amount);
}

#[test]
fn basic_transfer_with_decimal_round_down_conversion_test() {
    let mut state = MultiTransferTestState::new();
    // Send 500 with 18 ETH decimals
    let eth_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let call_data = ManagedBuffer::from(b"add");
    call_data
        .clone()
        .concat(ManagedBuffer::from(GAS_LIMIT.to_string()));
    call_data.clone().concat(ManagedBuffer::default());

    // Use BRIDGE_TOKEN_ID which has 18 ETH decimals → 6 KDA decimals
    let kda_amount = state
        .world
        .query()
        .to(KDA_SAFE_ADDRESS)
        .typed(kda_safe_proxy::KDASafeProxy)
        .convert_eth_to_kda_amount_endpoint(BRIDGE_TOKEN_ID, &eth_amount)
        .returns(ReturnsResult)
        .run();
    
    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: eth_amount.clone(),
        converted_amount: kda_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .batch_transfer_kda_token(1u32, transfers)
        .run();

    state.check_balances_on_safe(BRIDGE_TOKEN_ID, BigUint::zero(), BigUint::zero(), BigUint::zero());

    // With 18→6 decimal conversion: 500 * 10^6 / 10^18 = 0 (rounded down)
    state
        .world
        .check_account(USER1_ADDRESS)
        .kda_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), BigUint::zero());
}

#[test]
fn basic_transfer_with_decimal_conversion_test() {
    let mut state = MultiTransferTestState::new();
    // Send 1234 tokens with 18 ETH decimals (1234 * 10^18)
    let eth_amount = BigUint::from(1234u64) * BigUint::from(10u64).pow(18u32);
    // Expected KDA amount: 1234 tokens with 6 decimals (1234 * 10^6)
    let expected_kda_amount = BigUint::from(1234u64) * BigUint::from(10u64).pow(6u32);

    state.deploy_contracts();
    state.config_multi_transfer();

    let call_data = ManagedBuffer::from(b"add");
    call_data
        .clone()
        .concat(ManagedBuffer::from(GAS_LIMIT.to_string()));
    call_data.clone().concat(ManagedBuffer::default());

    // Use BRIDGE_TOKEN_ID which has 18 ETH decimals → 6 KDA decimals
    let kda_amount = state
        .world
        .query()
        .to(KDA_SAFE_ADDRESS)
        .typed(kda_safe_proxy::KDASafeProxy)
        .convert_eth_to_kda_amount_endpoint(BRIDGE_TOKEN_ID, &eth_amount)
        .returns(ReturnsResult)
        .run();
    
    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: eth_amount.clone(),
        converted_amount: kda_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .batch_transfer_kda_token(1u32, transfers)
        .run();

    // Verify the decimal conversion: 1234 * 10^18 → 1234 * 10^6
    state
        .world
        .check_account(USER1_ADDRESS)
        .kda_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), expected_kda_amount);
}

#[test]
fn batch_transfer_both_executed_test() {
    let mut state = MultiTransferTestState::new();
    // Use larger amounts that will demonstrate decimal conversion properly
    // For BRIDGE (18→6): 1000000000000000000 (1e18) → 1000000 (1e6)
    let bridge_eth_amount = BigUint::from(1000000000000000000u64); // 1 token with 18 decimals
    let bridge_expected_kda = BigUint::from(1000000u64); // Converts to 1 token with 6 decimals
    
    // For WRAPPED (8→8): no conversion, 500 stays 500
    let wrapped_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    // First transaction: BRIDGE_TOKEN_ID with 18→6 decimal conversion
    let bridge_kda_amount = state
        .world
        .query()
        .to(KDA_SAFE_ADDRESS)
        .typed(kda_safe_proxy::KDASafeProxy)
        .convert_eth_to_kda_amount_endpoint(BRIDGE_TOKEN_ID, &bridge_eth_amount)
        .returns(ReturnsResult)
        .run();
    
    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER2_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: bridge_eth_amount.clone(),
        converted_amount: bridge_kda_amount,
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data.clone()),
    };

    // Second transaction: WRAPPED_TOKEN_ID with 8→8 (no conversion)
    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(WRAPPED_TOKEN_ID),
        amount: wrapped_amount.clone(),
        converted_amount: wrapped_amount.clone(),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .batch_transfer_kda_token(1u32, transfers)
        .run();

    // Check USER1 received WRAPPED tokens (no conversion: 500 → 500)
    state
        .world
        .check_account(USER1_ADDRESS)
        .kda_balance(TokenIdentifier::from(WRAPPED_TOKEN_ID), wrapped_amount);

    // Check USER2 received BRIDGE tokens (with conversion: 1e18 → 1e6)
    state
        .world
        .check_account(USER2_ADDRESS)
        .kda_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), bridge_expected_kda);
}

#[test]
fn batch_two_transfers_same_token_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER2_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(TOKEN_ID),
        amount: token_amount.clone(),
        converted_amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data.clone()),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(TOKEN_ID),
        amount: token_amount.clone(),
        converted_amount: token_amount.clone(),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .batch_transfer_kda_token(1u32, transfers)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .kda_balance(TokenIdentifier::from(TOKEN_ID), token_amount.clone());

    state
        .world
        .check_account(USER2_ADDRESS)
        .kda_balance(TokenIdentifier::from(TOKEN_ID), token_amount);
}

#[test]
fn batch_transfer_both_failed_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(TOKEN_ID),
        amount: token_amount.clone(),
        converted_amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data.clone()),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER2_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(TOKEN_ID),
        amount: token_amount.clone(),
        converted_amount: token_amount.clone(),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .batch_transfer_kda_token(1u32, transfers)
        .run();

    let first_batch = state
        .world
        .query()
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .get_first_batch_any_status()
        .returns(ReturnsResult)
        .run();

    assert!(first_batch.is_none());

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .move_refund_batch_to_safe()
        .run();

    let first_batch = state
        .world
        .query()
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .get_first_batch_any_status()
        .returns(ReturnsResult)
        .run();

    assert!(first_batch.is_none());
}

#[test]
fn test_unwrap_token_create_transaction_paused() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();

    state.config_bridged_tokens_wrapper();

    state.world.set_kda_balance(
        USER1_ADDRESS,
        UNIVERSAL_TOKEN_IDENTIFIER,
        BigUint::from(10u64),
    );

    state
        .world
        .tx()
        .from(USER1_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(
            TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            KDA_SAFE_ADDRESS.to_address(),
            EthAddress::zero(),
        )
        .klv_or_single_kda(
            &TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(10u64),
        )
        .returns(ExpectError(ERROR, "Contract is paused"))
        .run();
}

#[test]
fn test_unwrap_token_create_transaction_insufficient_liquidity() {
    let mut state = MultiTransferTestState::new();
    state.deploy_contracts();
    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unpause_endpoint()
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .deposit_liquidity()
        .klv_or_single_kda(
            &TokenIdentifier::from(WRAPPED_TOKEN_ID),
            0u64,
            &BigUint::from(1_000u64),
        )
        .run();

    state
        .world
        .set_kda_balance(USER1_ADDRESS, UNIVERSAL_TOKEN_IDENTIFIER, BigUint::from(5_000u64));

    state
        .world
        .tx()
        .from(USER1_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(WRAPPED_TOKEN_ID, KDA_SAFE_ADDRESS.to_address(), EthAddress::zero())
        .klv_or_single_kda(
            &TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(2_000u64),
        )
        .returns(ExpectError(ERROR, "Contract does not have enough funds"))
        .run();
}

#[test]
fn test_unwrap_token_create_transaction_should_work() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();

    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .deposit_liquidity()
        .klv_or_single_kda(
            &TokenIdentifier::from(WRAPPED_TOKEN_ID),
            0u64,
            &BigUint::from(1_000u64),
        )
        .run();

    state
        .world
        .set_kda_balance(USER1_ADDRESS, UNIVERSAL_TOKEN_IDENTIFIER, BigUint::from(5_000u64));

    state.check_balances_on_safe(
        WRAPPED_TOKEN_ID,
        BigUint::zero(),
        BigUint::from(600000u64),
        BigUint::zero(),
    );
    
    state
        .world
        .query()
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .token_liquidity(WRAPPED_TOKEN_ID)
        .returns(ExpectValue(BigUint::from(1000u64)))
        .run();

    state
        .world
        .tx()
        .from(USER1_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(WRAPPED_TOKEN_ID, KDA_SAFE_ADDRESS.to_address(), EthAddress::zero())
        .klv_or_single_kda(
            &TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(900u64),
        )
        .run();

    state
        .world
        .query()
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .token_liquidity(WRAPPED_TOKEN_ID)
        .returns(ExpectValue(BigUint::from(100u64)))
        .run();

    state.check_balances_on_safe(
        WRAPPED_TOKEN_ID,
        BigUint::zero(),
        BigUint::from(600000u64),
        BigUint::from(900u64),
    );
}

#[test]
fn test_unwrap_token_create_transaction_should_fail() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();

    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();

    state
        .world
        .set_kda_balance(USER1_ADDRESS, TOKEN_ID, BigUint::from(5_000u64));

    state
        .world
        .tx()
        .from(USER1_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(WRAPPED_TOKEN_ID, KDA_SAFE_ADDRESS.to_address(), EthAddress::zero())
        .klv_or_single_kda(
            &TokenIdentifier::from(TOKEN_ID),
            0u64,
            &BigUint::from(1_000u64),
        )
        .returns(ExpectError(ERROR, "KDA token unavailable"))
        .run();
}

#[test]
fn test_unwrap_token_create_transaction_amount_zero() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();

    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();

    state
        .world
        .set_kda_balance(USER1_ADDRESS, UNIVERSAL_TOKEN_IDENTIFIER, BigUint::from(5_000u64));

    state
        .world
        .tx()
        .from(USER1_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(
            TokenIdentifier::from(WRAPPED_TOKEN_ID),
            KDA_SAFE_ADDRESS.to_address(),
            EthAddress::zero(),
        )
        .klv_or_single_kda(
            &TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(0u64),
        )
        .returns(ExpectError(ERROR, "Must pay more than 0 tokens!"))
        .run();
}

#[test]
fn add_refund_batch_test() {
    let mut state = MultiTransferTestState::new();

    state.multi_transfer_deploy();
    state.safe_deploy(Address::zero());
    state.bridged_tokens_wrapper_deploy();
    state.config_multi_transfer();

    let eth_tx = EthTransaction {
        from: EthAddress::zero(),
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(TOKEN_ID),
        amount: BigUint::from(MAX_AMOUNT),
        converted_amount: BigUint::from(MAX_AMOUNT),
        tx_nonce: 1u64,
        call_data: ManagedOption::none(),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx.clone());

    let fee = state
        .world
        .query()
        .to(KDA_SAFE_ADDRESS)
        .typed(kda_safe_proxy::KDASafeProxy)
        .calculate_required_fee(TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    state.check_balances_on_safe(
        TOKEN_ID,
        BigUint::from(MAX_AMOUNT),
        BigUint::zero(),
        BigUint::zero(),
    );

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .batch_transfer_kda_token(1u32, transfers)
        .run();
    state.check_balances_on_safe(TOKEN_ID, BigUint::zero(), BigUint::zero(), BigUint::zero());

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .move_refund_batch_to_safe()
        .run();

    state.check_balances_on_safe(
        TOKEN_ID,
        BigUint::from(MAX_AMOUNT) - fee,
        BigUint::zero(),
        BigUint::zero(),
    );
}

#[test]
fn add_refund_batch_with_decimal_conversion_test() {
    let mut state = MultiTransferTestState::new();

    state.multi_transfer_deploy();
    state.safe_deploy(Address::zero());
    state.bridged_tokens_wrapper_deploy();
    state.config_multi_transfer();

    // Use BRIDGE-123456 which has 18 ETH decimals → 6 KDA decimals
    // For this test, use MAX_AMOUNT in ETH decimals (18 decimals)
    let eth_amount = BigUint::from(30_000u64) *  BigUint::from(10u64).pow(18u32); // This is in 18 ETH decimals

    let kda_amount = state
        .world
        .query()
        .to(KDA_SAFE_ADDRESS)
        .typed(kda_safe_proxy::KDASafeProxy)
        .convert_eth_to_kda_amount_endpoint(BRIDGE_TOKEN_ID, &eth_amount)
        .returns(ReturnsResult)
        .run();
    
    let eth_tx = EthTransaction {
        from: EthAddress::zero(),
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: eth_amount.clone(),
        converted_amount: kda_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::none(),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx.clone());

    // Calculate fee for BRIDGE token
    let fee = state
        .world
        .query()
        .to(KDA_SAFE_ADDRESS)
        .typed(kda_safe_proxy::KDASafeProxy)
        .calculate_required_fee(BRIDGE_TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    // Initial balance should be 0 since BRIDGE is a mint token
    state.check_balances_on_safe(
        BRIDGE_TOKEN_ID,
        BigUint::zero(),
        BigUint::zero(),
        BigUint::zero(),
    );

    // Transfer the tokens - this will attempt to mint and transfer them on KDA side
    // Since the user doesn't exist or can't receive, this will trigger a refund
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .batch_transfer_kda_token(1u32, transfers)
        .run();
    
    let expected_converted_amount = eth_amount / BigUint::from(10u64).pow(12u32);

    state.check_balances_on_safe(BRIDGE_TOKEN_ID, BigUint::zero(), expected_converted_amount.clone(), BigUint::zero());

    // Move refund batch to safe - this should restore balance minus fees
    // The refund transaction should contain the non-converted (ETH) amount
    // because this is what needs to be returned on the Ethereum side
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferKdaProxy)
        .move_refund_batch_to_safe()
        .run();

    // Get the batch from kda-safe to verify the refund transaction
    let batch: OptionalValue<TxBatchSplitInFields<StaticApi>> = state
        .world
        .query()
        .to(KDA_SAFE_ADDRESS)
        .typed(kda_safe_proxy::KDASafeProxy)
        .get_batch(1u64)
        .returns(ReturnsResult)
        .run();

    match batch {
        OptionalValue::Some(batch_data) => {
            let (batch_id, transactions) = batch_data.into_tuple();
            assert_eq!(batch_id, 1u64, "Batch ID should be 1");
            
            // Get the first (and only) transaction from the batch
            let mut tx_count = 0;
            let mut found_amount = BigUint::zero();
            let mut found_token_id = TokenIdentifier::from("");
            
            for tx in transactions.into_iter() {
                tx_count += 1;
                let (block_nonce, nonce, from, to, token_id, amount, converted_amount) = tx.into_tuple();
                found_amount = amount;
                found_token_id = token_id;
            }
            
            assert_eq!(tx_count, 1, "Should have exactly one refund transaction");
            
            // Verify the refund transaction contains the NON-CONVERTED (ETH) amount
            // This is crucial because the refund needs to return the original ETH amount minus fees
            let expected_fee_converted = fee.clone() *  BigUint::from(10u64).pow(12u32);
            let expected_eth_amount = BigUint::from(30_000u64) *  BigUint::from(10u64).pow(18u32) - expected_fee_converted;
            assert_eq!(
                found_amount, expected_eth_amount,
                "Refund transaction should contain the original ETH amount (non-converted)"
            );
            assert_eq!(
                found_token_id,
                TokenIdentifier::from(BRIDGE_TOKEN_ID),
                "Token ID should match"
            );
        }
        OptionalValue::None => {
            panic!("Batch should exist after moving refund");
        }
    }

    // Expected KDA amount: (converted_amount - fee) 
    let expected_kda_amount = (expected_converted_amount.clone() - fee);
    state.check_balances_on_safe(
        BRIDGE_TOKEN_ID,
        BigUint::zero(),
        expected_converted_amount,
        expected_kda_amount,
    );
}
