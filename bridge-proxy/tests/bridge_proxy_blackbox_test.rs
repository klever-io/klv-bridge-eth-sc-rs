#![allow(unused)]

use std::collections::LinkedList;
use std::ops::Add;

use bridge_proxy::{bridge_proxy_contract_proxy, config::ProxyTrait as _};
use bridge_proxy::{bridged_tokens_wrapper_proxy, ProxyTrait};

use crowdfunding_kda::crowdfunding_kda_proxy;
use klever_sc::codec::NestedEncode;
use klever_sc::contract_base::ManagedSerializer;
use klever_sc::sc_print;
use klever_sc::types::{
    KdaTokenPayment, ManagedOption, ReturnsNewAddress, TestAddress,
    TestSCAddress, TestTokenIdentifier,
};
use klever_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        TopEncodeMultiOutput,
    },
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress, ManagedArgBuffer, ManagedBuffer,
        ManagedByteArray, ManagedVec, TokenIdentifier,
    },
};
use klever_sc_scenario::imports::KleverscPath;
use klever_sc_scenario::{
    api::StaticApi,
    rust_biguint,
    scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext},
    scenario_model::*,
    ContractInfo, ScenarioWorld,
};
use klever_sc_scenario::{managed_address, ExpectValue, ScenarioTxRun};

use eth_address::*;
use transaction::{CallData, EthTransaction};

const BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE-1234";
const WBRIDGE_TOKEN_ID: &[u8] = b"WBRIDGE-1234";

const GAS_LIMIT: u64 = 10_000_000;
const CF_DEADLINE: u64 = 7 * 24 * 60 * 60; // 1 week in seconds

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const CROWDFUNDING_ADDRESS: TestSCAddress = TestSCAddress::new("crowfunding");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const KDA_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("kda-safe");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");

const BRIDGE_PROXY_PATH_EXPR: KleverscPath = KleverscPath::new("output/bridge-proxy.kleversc.json");
const CROWDFUNDING_PATH_EXPR: KleverscPath =
    KleverscPath::new("tests/test-contract/crowdfunding-kda.kleversc.json");
const MULTI_TRANSFER_PATH_EXPR: &str =
    "kleversc:../multi-transfer-kda/output/multi-transfer-kda.kleversc.json";
const KDA_SAFE_PATH_EXPR: &str = "kleversc:../kda-safe/output/kda-safe.kleversc.json";
const BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR: KleverscPath =
    KleverscPath::new("../bridged-tokens-wrapper/output/bridged-tokens-wrapper.kleversc.json");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(BRIDGE_PROXY_PATH_EXPR, bridge_proxy::ContractBuilder);
    blockchain.register_contract(CROWDFUNDING_PATH_EXPR, crowdfunding_kda::ContractBuilder);
    blockchain.register_contract(
        BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR,
        bridged_tokens_wrapper::ContractBuilder,
    );
    blockchain.register_contract(KDA_SAFE_PATH_EXPR, kda_safe::ContractBuilder);

    blockchain
}

type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;
type CrowdfundingContract = ContractInfo<crowdfunding_kda::Proxy<StaticApi>>;
type BridgedTokensWrapperContract = ContractInfo<bridged_tokens_wrapper::Proxy<StaticApi>>;

struct BridgeProxyTestState {
    world: ScenarioWorld,
}

impl BridgeProxyTestState {
    fn new() -> Self {
        let mut world = world();
        let multi_transfer_code = world.code_expression(MULTI_TRANSFER_PATH_EXPR);
        let kda_safe_code = world.code_expression(KDA_SAFE_PATH_EXPR);

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .kda_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 10_000u64)
            .account(MULTI_TRANSFER_ADDRESS)
            .kda_balance(TokenIdentifier::from(WBRIDGE_TOKEN_ID), 10_000u64)
            .kda_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 10_000u64)
            .code(multi_transfer_code)
            .account(KDA_SAFE_ADDRESS)
            .code(kda_safe_code);

        let roles = vec![
            "KDARoleMint".to_string(),
        ];

        world
            .account(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .kda_roles(TokenIdentifier::from(WBRIDGE_TOKEN_ID), roles.clone())
            .kda_roles(TokenIdentifier::from(BRIDGE_TOKEN_ID), roles)
            .kda_balance(TokenIdentifier::from(WBRIDGE_TOKEN_ID), 10_000u64)
            .kda_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 10_000u64)
            .code(BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR)
            .owner(OWNER_ADDRESS);

        let contract_address = &AddressValue::from(BRIDGED_TOKENS_WRAPPER_ADDRESS).to_address();

        world.set_kda_can_burn(
            managed_address!(contract_address),
            WBRIDGE_TOKEN_ID,
            0,
            true,
        );

        world.set_kda_can_burn(
            managed_address!(contract_address),
            BRIDGE_TOKEN_ID,
            0,
            true,
        );

        Self { world }
    }

    fn bridge_proxy_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .init(OptionalValue::Some(MULTI_TRANSFER_ADDRESS.eval_to_array()))
            .code(BRIDGE_PROXY_PATH_EXPR)
            .new_address(BRIDGE_PROXY_ADDRESS)
            .run();

        self
    }

    fn bridged_tokens_wrapper_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .init()
            .code(BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR)
            .new_address(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .run();

        self
    }

    fn deploy_crowdfunding(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(crowdfunding_kda_proxy::CrowdfundingProxy)
            .init(
                2_000u32,
                CF_DEADLINE,
                TokenIdentifier::klv(),
            )
            .code(CROWDFUNDING_PATH_EXPR)
            .new_address(CROWDFUNDING_ADDRESS)
            .run();
        self
    }

    fn config_bridge(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
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
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .set_bridged_tokens_wrapper_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.eval_to_array(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .whitelist_token(BRIDGE_TOKEN_ID, 18u32, WBRIDGE_TOKEN_ID)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .add_wrapped_token(WBRIDGE_TOKEN_ID, 18u32)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .deposit_liquidity()
            .single_kda(
                &TokenIdentifier::from(BRIDGE_TOKEN_ID),
                0u64,
                &BigUint::from(5_000u64),
            )
            .run();

        self
    }
}

#[test]
fn deploy_test() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_crowdfunding();
    test.config_bridge();
}

#[test]
fn bridge_proxy_execute_crowdfunding_test() {
    let mut test = BridgeProxyTestState::new();

    test.world.start_trace();

    test.bridge_proxy_deploy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(CROWDFUNDING_ADDRESS.eval_to_array()),
        token_id: BRIDGE_TOKEN_ID.into(),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data),
    };

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx, 1u64)
        .klv_or_single_kda(
            &TokenIdentifier::from(BRIDGE_TOKEN_ID),
            0,
            &BigUint::from(500u64),
        )
        .run();

    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .get_pending_transaction_by_id(1u32)
        .returns(ExpectValue(eth_tx))
        .run();

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .gas(200_000_000)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(1u32)
        .run();

    test.world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_kda_proxy::CrowdfundingProxy)
        .get_current_funds()
        .returns(ExpectValue(500u64))
        .run();

    test.world
        .write_scenario_trace("scenarios/bridge_proxy_execute_crowdfunding.scen.json");
}

#[test]
fn multiple_deposit_test() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(CROWDFUNDING_ADDRESS.eval_to_array()),
        token_id: BRIDGE_TOKEN_ID.into(),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data.clone()),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(CROWDFUNDING_ADDRESS.eval_to_array()),
        token_id: BRIDGE_TOKEN_ID.into(),
        amount: BigUint::from(500u64),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx1, 1u64)
        .single_kda(
            &TokenIdentifier::from(BRIDGE_TOKEN_ID),
            0u64,
            &BigUint::from(500u64),
        )
        .run();

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx2, 1u64)
        .single_kda(
            &TokenIdentifier::from(BRIDGE_TOKEN_ID),
            0u64,
            &BigUint::from(500u64),
        )
        .run();

    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .get_pending_transaction_by_id(1u32)
        .returns(ExpectValue(eth_tx1))
        .run();

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .gas(200_000_000)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(1u32)
        .run();

    test.world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_kda_proxy::CrowdfundingProxy)
        .get_current_funds()
        .returns(ExpectValue(500u64))
        .run();

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .gas(200_000_000)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(2u32)
        .run();

    test.world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_kda_proxy::CrowdfundingProxy)
        .get_current_funds()
        .returns(ExpectValue(BigUint::from(1_000u32)))
        .run();

    test.world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_kda_proxy::CrowdfundingProxy)
        .get_current_funds()
        .returns(ExpectValue(1_000u64))
        .run();
}

#[test]
fn test_highest_tx_id() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    // Generate 1600 transactions
    let mut transactions = Vec::new();
    for i in 1..=1600 {
        let eth_tx = EthTransaction {
            from: EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            to: ManagedAddress::from(CROWDFUNDING_ADDRESS.eval_to_array()),
            token_id: BRIDGE_TOKEN_ID.into(),
            amount: BigUint::from(5u64),
            tx_nonce: i as u64,
            call_data: ManagedOption::some(call_data.clone()),
        };
        transactions.push(eth_tx);
    }
    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .highest_tx_id()
        .returns(ExpectValue(0usize))
        .run();

    // Deposit all transactions
    let mut expected_tx_id = 1usize;
    for tx in &transactions {
        test.world
            .tx()
            .from(MULTI_TRANSFER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .deposit(tx, 1u64)
            .single_kda(
                &TokenIdentifier::from(BRIDGE_TOKEN_ID),
                0u64,
                &BigUint::from(5u64),
            )
            .run();

        test.world
            .query()
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .highest_tx_id()
            .returns(ExpectValue(expected_tx_id))
            .run();
        expected_tx_id += 1;
    }

    // Execute all transactions
    for i in (1..=1600usize).rev() {
        test.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .gas(200_000_000)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .execute(i)
            .run();
    }
}
