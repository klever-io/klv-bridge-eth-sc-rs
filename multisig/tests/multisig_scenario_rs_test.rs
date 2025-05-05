use klever_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("file:output/multisig.wasm", multisig::ContractBuilder);
    blockchain.register_contract(
        "file:../multi-transfer-kda/output/multi-transfer-kda.wasm",
        multi_transfer_kda::ContractBuilder,
    );
    blockchain.register_contract(
        "file:../kda-safe/output/kda-safe.wasm",
        kda_safe::ContractBuilder,
    );
    blockchain.register_contract(
        "file:../price-aggregator/output/klv-price-aggregator-sc.wasm",
        klv_price_aggregator_sc::ContractBuilder,
    );
    blockchain
}

#[test]
fn change_token_config_rs() {
    world().run("scenarios/change_token_config.scen.json");
}

#[test]
fn create_klever_to_ethereum_tx_batch_rs() {
    world().run("scenarios/create_klever_to_ethereum_tx_batch.scen.json");
}

#[test]
#[ignore] //There is an equivalent blackbox test
fn ethereum_to_klever_tx_batch_ok_rs() {
    world().run("scenarios/ethereum_to_klever_tx_batch_ok.scen.json");
}

#[test]
#[ignore] //There is an equivalent blackbox test
fn ethereum_to_klever_tx_batch_rejected_rs() {
    world().run("scenarios/ethereum_to_klever_tx_batch_rejected.scen.json");
}

#[test]
#[ignore] //There is an equivalent blackbox test
fn ethereum_to_klever_tx_batch_without_data_rs() {
    world().run("scenarios/ethereum_to_klever_tx_batch_without_data.scen.json");
}

#[test]
fn execute_klever_to_ethereum_tx_batch_rs() {
    world().run("scenarios/execute_klever_to_ethereum_tx_batch.scen.json");
}

#[test]
fn get_empty_batch_rs() {
    world().run("scenarios/get_empty_batch.scen.json");
}

#[test]
fn reject_klever_to_ethereum_tx_batch_rs() {
    world().run("scenarios/reject_klever_to_ethereum_tx_batch.scen.json");
}

#[test]
fn setup_rs() {
    world().run("scenarios/setup.scen.json");
}

#[test]
fn unstake_rs() {
    world().run("scenarios/unstake.scen.json");
}
