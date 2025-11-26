#![allow(unused)]

use std::ops::Add;

use kda_safe::{KDASafe, ProxyTrait as _};
use multi_transfer_kda::{bridged_tokens_wrapper_proxy, multi_transfer_proxy, ProxyTrait as _};

use multisig::{
    __endpoints_5__::multi_transfer_kda_address, kda_safe_proxy, multi_transfer_kda_proxy,
    multisig_proxy,
};
use klever_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        Empty,
    },
    contract_base::ManagedSerializer,
    hex_literal::hex,
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress, ManagedBuffer, ManagedByteArray,
        ManagedOption, ManagedType, ManagedVec, MultiValueEncoded, ReturnsNewManagedAddress,
        ReturnsResult, TestAddress, TestSCAddress, TestTokenIdentifier, TokenIdentifier,
    },
};
use klever_sc_modules::pause::ProxyTrait;
use klever_sc_scenario::{
    api::{StaticApi, VMHooksApi, VMHooksApiBackend}, imports::KleverscPath, managed_address, scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext}, scenario_model::*, ContractInfo, DebugApi, ExpectError, ExpectValue, ScenarioTxRun, ScenarioWorld
};

use eth_address::*;
use token_module::ProxyTrait as _;
use transaction::{CallData, EthTransaction, EthTxAsMultiValue, TxBatchSplitInFields};

const WKLV_TOKEN_ID: &[u8] = b"WKLV-123456";
const ETH_TOKEN_ID: &[u8] = b"ETH-123456";

const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 100_000_000;

const MULTISIG_CODE_PATH: KleverscPath = KleverscPath::new("output/multisig.kleversc.json");
const MULTI_TRANSFER_CODE_PATH: KleverscPath =
    KleverscPath::new("../multi-transfer-kda/output/multi-transfer-kda.kleversc.json");
const KDA_SAFE_CODE_PATH: KleverscPath = KleverscPath::new("../kda-safe/output/kda-safe.kleversc.json");
const BRIDGED_TOKENS_WRAPPER_CODE_PATH: KleverscPath =
    KleverscPath::new("../bridged-tokens-wrapper/output/bridged-tokens-wrapper.kleversc.json");
const PRICE_AGGREGATOR_CODE_PATH: KleverscPath =
    KleverscPath::new("../price-aggregator/price-aggregator.kleversc.json");

const MULTISIG_ADDRESS: TestSCAddress = TestSCAddress::new("multisig");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const KDA_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("kda-safe");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");
const PRICE_AGGREGATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price-aggregator");

const ORACLE_ADDRESS: TestAddress = TestAddress::new("oracle");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER1_ADDRESS: TestAddress = TestAddress::new("user1");
const USER2_ADDRESS: TestAddress = TestAddress::new("user2");
const RELAYER1_ADDRESS: TestAddress = TestAddress::new("relayer1");
const RELAYER2_ADDRESS: TestAddress = TestAddress::new("relayer2");

const RANDOM_SC_ADDRESS: TestSCAddress = TestSCAddress::new("random-sc");

const KDA_SAFE_ETH_TX_GAS_LIMIT: u64 = 150_000;

const BALANCE: &str = "2,000,000";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(MULTISIG_CODE_PATH, multisig::ContractBuilder);
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
            .nonce(1)
            .kda_balance(TokenIdentifier::from(WKLV_TOKEN_ID), 1001u64)
            .kda_balance(TokenIdentifier::from(ETH_TOKEN_ID), 1001u64)
            .account(USER1_ADDRESS)
            .nonce(1)
            .account(RELAYER1_ADDRESS)
            .nonce(1)
            .balance(1_000u64)
            .account(RELAYER2_ADDRESS)
            .nonce(1)
            .balance(1_000u64);

        let roles = vec![
            "KDARoleMint".to_string(),
        ];
        world
            .account(KDA_SAFE_ADDRESS)
            .kda_roles(TokenIdentifier::from(WKLV_TOKEN_ID), roles.clone())
            .kda_roles(TokenIdentifier::from(ETH_TOKEN_ID), roles)
            .code(KDA_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS);

        let bridged_tokens_address = AddressValue::from(BRIDGED_TOKENS_WRAPPER_ADDRESS);

        world.set_kda_can_burn(
            managed_address!(&bridged_tokens_address.to_address()),
            WKLV_TOKEN_ID,
            0,
            true,
        );

        world.set_kda_can_burn(
            managed_address!(&bridged_tokens_address.to_address()),
            ETH_TOKEN_ID,
            0,
            true,
        );

        Self { world }
    }

    fn multisig_deploy(&mut self) -> &mut Self {
        let mut board: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> =
            MultiValueEncoded::new();
        board.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
        board.push(ManagedAddress::from(RELAYER2_ADDRESS.eval_to_array()));
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
            .init(
                KDA_SAFE_ADDRESS.to_address(),
                MULTI_TRANSFER_ADDRESS.to_address(),
                1_000u64,
                500u64,
                2usize,
                board,
            )
            .code(MULTISIG_CODE_PATH)
            .new_address(MULTISIG_ADDRESS)
            .run();
        self
    }

    fn multi_transfer_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(multi_transfer_kda_proxy::MultiTransferKdaProxy)
            .init()
            .code(MULTI_TRANSFER_CODE_PATH)
            .new_address(MULTI_TRANSFER_ADDRESS)
            .run();
        
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_kda_proxy::MultiTransferKdaProxy)
            .add_admin(MULTISIG_ADDRESS.eval_to_array())
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

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .add_admin(OWNER_ADDRESS.eval_to_array())
            .run();


        self
    }

    fn config_multisig(&mut self) {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferKdaProxy)
            .set_wrapping_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
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
                TokenIdentifier::from_kda_bytes("WKLV-123456"),
                "WKLV",
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
            .add_token_to_whitelist(
                TokenIdentifier::from_kda_bytes("ETH-123456"),
                "ETH",
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
            .add_token_to_whitelist(
                TokenIdentifier::from_kda_bytes("ETHUSDC-afa689"),
                "ETHUSDC",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(KDA_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();

        // Configure decimals for WKLV token (8 decimals on both sides - no conversion)
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_token_decimals(TokenIdentifier::from_kda_bytes("WKLV-123456"), 8u32, 8u32)
            .run();

        // Configure decimals for ETH token (18 ETH decimals → 8 KDA decimals)
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_token_decimals(TokenIdentifier::from_kda_bytes("ETH-123456"), 18u32, 8u32)
            .run();

        // Configure decimals for ETHUSDC token (6 ETH decimals → 6 KDA decimals - no conversion)
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(KDA_SAFE_ADDRESS)
            .typed(kda_safe_proxy::KDASafeProxy)
            .set_token_decimals(TokenIdentifier::from_kda_bytes("ETHUSDC-afa689"), 6u32, 6u32)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unpause_endpoint()
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
            .from(RELAYER1_ADDRESS)
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
            .stake()
            .klv(1_000)
            .run();

        self.world
            .tx()
            .from(RELAYER2_ADDRESS)
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
            .stake()
            .klv(1_000)
            .run();

        let staked_relayers = self
            .world
            .query()
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
            .get_all_staked_relayers()
            .returns(ReturnsResult)
            .run();

        assert!(staked_relayers
            .to_vec()
            .contains(&ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array())));
        assert!(staked_relayers
            .to_vec()
            .contains(&ManagedAddress::from(RELAYER2_ADDRESS.eval_to_array())));
    }
}

#[test]
fn config_test() {
    let mut state = MultiTransferTestState::new();

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();
}

#[test]
fn ethereum_to_klever_call_data_empty_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WKLV_TOKEN_ID),
        token_amount.clone(),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_kda_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .kda_balance(TokenIdentifier::from(WKLV_TOKEN_ID), token_amount.clone());
}

#[test]
fn ethereum_to_klever_relayer_call_data_several_tx_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(5_000u64);

    state.world.start_trace();

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

    let addr =
        Address::from_slice(b"klv12e0kqcvqsrayj8j0c4dqjyvnv4ep253m5anx4rfj4jeq34lxsg8s84ec9j");
    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(addr.clone()),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let eth_tx2 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(addr.clone()),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        token_amount.clone(),
        2u64,
        ManagedOption::none(),
    ));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::none(),
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx3 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(addr.clone()),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        token_amount.clone(),
        3u64,
        ManagedOption::some(call_data),
    ));

    let args = ManagedVec::from_single_item(ManagedBuffer::from(b"5"));
    let call_data2: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };
    let call_data2 = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data2);

    let eth_tx4 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(addr.clone()),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        token_amount.clone(),
        4u64,
        ManagedOption::some(call_data2),
    ));
    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);
    transfers.push(eth_tx2);
    transfers.push(eth_tx3);
    transfers.push(eth_tx4);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_kda_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .returns(ExpectError(57, "Invalid token or amount"))
        .run();

    state.world.write_scenario_trace(
        "scenarios/ethereum_to_klever_relayer_call_data_several_tx_test.scen.json",
    );
}

#[test]
fn ethereum_to_klever_relayer_query_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);
    state.world.start_trace();

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WKLV_TOKEN_ID),
        token_amount.clone(),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_kda_batch(1u32, transfers.clone())
        .run();

    let was_transfer = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .was_transfer_action_proposed(1u64, transfers.clone())
        .returns(ReturnsResult)
        .run();

    assert!(was_transfer);

    let get_action_id = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .get_action_id_for_transfer_batch(1u64, transfers)
        .returns(ReturnsResult)
        .run();

    assert!(get_action_id == 1usize);

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .kda_balance(TokenIdentifier::from(WKLV_TOKEN_ID), token_amount.clone());

    state
        .world
        .write_scenario_trace("scenarios/ethereum_to_klever_relayer_query_test.scen.json");
}

#[test]
fn ethereum_to_klever_relayer_query2_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(5_000u64);
    state.world.start_trace();

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

    let addr =
        Address::from_slice(b"klv12e0kqcvqsrayj8j0c4dqjyvnv4ep253m5anx4rfj4jeq34lxsg8s84ec9j");

    const ADDR: [u8; 32] = hex!("691dee92137cddbe76ec34eeacbc3b7d91264148da5a69205133c395aa7662cf");

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(ADDR),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_kda_batch(1u32, transfers.clone())
        .run();

    let was_transfer = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .was_transfer_action_proposed(1u64, transfers.clone())
        .returns(ReturnsResult)
        .run();

    assert!(was_transfer);

    let get_action_id = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .get_action_id_for_transfer_batch(1u64, transfers)
        .returns(ReturnsResult)
        .run();

    assert!(get_action_id == 1usize);

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .returns(ExpectError(57, "Invalid token or amount"))
        .run();

    state
        .world
        .write_scenario_trace("scenarios/ethereum_to_klever_relayer_query2_test.scen.json");
}

#[test]
fn ethereum_to_klever_tx_batch_ok_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);
    state.world.start_trace();

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8, 6u8]));
    args.push(ManagedBuffer::from(&[7u8, 8u8, 9u8]));
    args.push(ManagedBuffer::from(&[7u8, 8u8, 9u8, 10u8, 11u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WKLV_TOKEN_ID),
        token_amount.clone(),
        token_amount.clone(),
        1u64,
        ManagedOption::some(call_data.clone()),
    ));

    let eth_tx2 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(ETH_TOKEN_ID),
        token_amount.clone(),
        token_amount.clone(),
        2u64,
        ManagedOption::some(call_data.clone()),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_kda_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .kda_balance(TokenIdentifier::from(WKLV_TOKEN_ID), token_amount.clone())
        .kda_balance(TokenIdentifier::from(ETH_TOKEN_ID), token_amount.clone());

    state.world.write_scenario_trace(
        "scenarios/ethereum_to_klever_tx_batch_ok_call_data_encoded.scen.json",
    );
}

#[test]
fn ethereum_to_klever_tx_batch_rejected_test() {
    let mut state = MultiTransferTestState::new();
    let over_the_limit_token_amount = BigUint::from(101_000_000_000u64);

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WKLV_TOKEN_ID),
        over_the_limit_token_amount.clone(),
        over_the_limit_token_amount.clone(),
        1u64,
        ManagedOption::some(call_data.clone()),
    ));

    let eth_tx2 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(ETH_TOKEN_ID),
        over_the_limit_token_amount.clone(),
        over_the_limit_token_amount.clone(),
        2u64,
        ManagedOption::some(call_data.clone()),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_kda_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    let refund_tx = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .get_current_refund_batch()
        .returns(ReturnsResult)
        .run();

    assert!(refund_tx.is_none());

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .move_refund_batch_to_safe_from_child_contract()
        .run();
}

#[test]
fn propose_multi_transfer_without_decimals_configured_should_fail() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

    // Add a token to whitelist but don't configure decimals
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(KDA_SAFE_ADDRESS)
        .typed(kda_safe_proxy::KDASafeProxy)
        .add_token_to_whitelist(
            TokenIdentifier::from_kda_bytes("UNCONFIGURED-123456"),
            "UNCONF",
            true,
            false,
            BigUint::zero(),
            BigUint::zero(),
            BigUint::zero(),
            OptionalValue::Some(BigUint::from(KDA_SAFE_ETH_TX_GAS_LIMIT)),
        )
        .run();

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from_kda_bytes("UNCONFIGURED-123456"),
        token_amount.clone(),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    // Try to propose a batch with unconfigured token decimals - should fail
    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_kda_batch(1u64, transfers)
        .returns(ExpectError(
            57,
            "ETH decimals not configured for this token. Call setTokenDecimals first.",
        ))
        .run();
}
