use klv_price_aggregator_sc::{
    price_aggregator_data::{OracleStatus, TimestampedPrice, TokenPair},
    PriceAggregator, MAX_ROUND_DURATION_SECONDS,
};

use klever_sc_modules::pause::PauseModule;
use klever_sc_scenario::imports::*;
use staking_module::StakingModule;

pub const DECIMALS: u8 = 0;
pub const KLV_TICKER: &[u8] = b"KLV";
pub const NR_ORACLES: usize = 4;
pub const SLASH_AMOUNT: u64 = 10;
pub const SLASH_QUORUM: usize = 3;
pub const STAKE_AMOUNT: u64 = 20;
pub const SUBMISSION_COUNT: usize = 3;
pub const USD_TICKER: &[u8] = b"USDC";

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const PRICE_AGGREGATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price-aggregator");
const PRICE_AGGREGATOR_PATH_EXPR: KleverscPath =
    KleverscPath::new("kleversc:output/klv-price-aggregator-sc.kleversc.json");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    
    blockchain.register_contract(
        PRICE_AGGREGATOR_PATH_EXPR,
        klv_price_aggregator_sc::ContractBuilder,
    );

    blockchain
}

#[test]
fn test_price_aggregator_submit() {
    let (mut world, oracles) = setup();
    let price_aggregator_whitebox = WhiteboxContract::new("sc:price-aggregator", klv_price_aggregator_sc::contract_obj);
    
    // configure the number of decimals
    world
        .whitebox_call(&price_aggregator_whitebox, ScCallStep::new()
        .from(OWNER_ADDRESS)
        .to(PRICE_AGGREGATOR_ADDRESS),
        |sc| {
            sc.set_pair_decimals(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                DECIMALS,
            )
        }
    );


    // try submit while paused
    world
        .whitebox_call_check(&price_aggregator_whitebox, ScCallStep::new()
        .from(&oracles[0])
        .to(PRICE_AGGREGATOR_ADDRESS)
        .expect(TxExpect::user_error("str:Contract is paused")),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                99,
                managed_biguint!(100),
                DECIMALS,
            )
        },
        |r| r.assert_user_error("Contract is paused"),
    );

    // unpause
    world.whitebox_call(
        &price_aggregator_whitebox, 
        ScCallStep::new().from(OWNER_ADDRESS),
        |sc| {
            sc.unpause_endpoint();
        }
    );

    // submit first timestamp too old
    world
        .whitebox_call_check(&price_aggregator_whitebox, ScCallStep::new()
        .from(&oracles[0])
        .to(PRICE_AGGREGATOR_ADDRESS)
        .no_expect(),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                10,
                managed_biguint!(100),
                DECIMALS,
            )
        }, |r| {
            r.assert_user_error("First submission too old");
        }
    );
        
    // submit ok
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(&oracles[0]),
        |sc| {

            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                95,
                managed_biguint!(100),
                DECIMALS,
            )
        },
    );

    let current_timestamp = 100;
    world.whitebox_query(&price_aggregator_whitebox, 
        |sc| {
            let token_pair = TokenPair {
                from: managed_buffer!(KLV_TICKER),
                to: managed_buffer!(USD_TICKER),
            };
            assert_eq!(
                sc.first_submission_timestamp(&token_pair).get(),
                current_timestamp
            );
            assert_eq!(
                sc.last_submission_timestamp(&token_pair).get(),
                current_timestamp
            );

            let submissions = sc.submissions().get(&token_pair).unwrap();
            assert_eq!(submissions.len(), 1);
            assert_eq!(
                submissions.get(&ManagedAddress::from(&oracles[0])).unwrap(),
                managed_biguint!(100)
            );

            assert_eq!(
                sc.oracle_status()
                    .get(&ManagedAddress::from(&oracles[0]))
                    .unwrap(),
                OracleStatus {
                    total_submissions: 1,
                    accepted_submissions: 1
                }
            );
        }
    );

    // first oracle submit again - submission not accepted
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(&oracles[0]),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                95,
                managed_biguint!(100),
                DECIMALS,
            )
        },
    );

    world.whitebox_query(&price_aggregator_whitebox,
        |sc| {
            assert_eq!(
                sc.oracle_status()
                    .get(&ManagedAddress::from(&oracles[0]))
                    .unwrap(),
                OracleStatus {
                    total_submissions: 2,
                    accepted_submissions: 1
                }
            );
        },
    );
}

#[test]
fn test_price_aggregator_submit_round_ok() {
    let (mut world, oracles) = setup();
    let price_aggregator_whitebox = WhiteboxContract::new("sc:price-aggregator", klv_price_aggregator_sc::contract_obj);

    // configure the number of decimals
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS),
        |sc| {
            sc.set_pair_decimals(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                DECIMALS,
            )
        },
    );

    // unpause
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS), |sc| 
        sc.unpause_endpoint(),
    );

    // submit first
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(&oracles[0]),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                95,
                managed_biguint!(10_000),
                DECIMALS,
            )
        },
    );

    let current_timestamp = 110;
    world.current_block().block_timestamp(current_timestamp);

    // submit second
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(&oracles[1]),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                101,
                managed_biguint!(11_000),
                DECIMALS,
            )
        },
    );

    // submit third
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(&oracles[2]),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                105,
                managed_biguint!(12_000),
                DECIMALS,
            )
        },
    );

    world.whitebox_query(&price_aggregator_whitebox,
        |sc| {
            let result =
                sc.latest_price_feed(managed_buffer!(KLV_TICKER), managed_buffer!(USD_TICKER));

            let (round_id, from, to, timestamp, price, decimals) = result.into_tuple();
            assert_eq!(round_id, 1);
            assert_eq!(from, managed_buffer!(KLV_TICKER));
            assert_eq!(to, managed_buffer!(USD_TICKER));
            assert_eq!(timestamp, current_timestamp);
            assert_eq!(price, managed_biguint!(11_000));
            assert_eq!(decimals, DECIMALS);

            // submissions are deleted after round is created
            let token_pair = TokenPair { from, to };
            let submissions = sc.submissions().get(&token_pair).unwrap();
            assert_eq!(submissions.len(), 0);

            let rounds = sc.rounds().get(&token_pair).unwrap();
            assert_eq!(rounds.len(), 1);
            assert_eq!(
                rounds.get(1),
                TimestampedPrice {
                    timestamp,
                    price,
                    decimals
                }
            );
        },
    );
}

#[test]
fn test_price_aggregator_discarded_round() {
    let (mut world, oracles) = setup();
    let price_aggregator_whitebox = WhiteboxContract::new("sc:price-aggregator", klv_price_aggregator_sc::contract_obj);

    // configure the number of decimals
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS),
        |sc| {
            sc.set_pair_decimals(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                DECIMALS,
            )
        },
    );

    // unpause
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS),|sc| 
            sc.unpause_endpoint(),
    );

    // submit first
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(&oracles[0]),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                95,
                managed_biguint!(10_000),
                DECIMALS,
            )
        },
    );

    let current_timestamp = 100 + MAX_ROUND_DURATION_SECONDS + 1;
    world.current_block().block_timestamp(current_timestamp);

    // submit second - this will discard the previous submission
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(&oracles[1]),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                current_timestamp - 1,
                managed_biguint!(11_000),
                DECIMALS,
            )
        },
    );

    world.whitebox_query(&price_aggregator_whitebox,
        |sc| {
            let token_pair = TokenPair {
                from: managed_buffer!(KLV_TICKER),
                to: managed_buffer!(USD_TICKER),
            };
            let submissions = sc.submissions().get(&token_pair).unwrap();
            assert_eq!(submissions.len(), 1);
            assert_eq!(
                submissions.get(&managed_address!(&oracles[1])).unwrap(),
                managed_biguint!(11_000)
            );
        },
    );
}

#[test]
fn test_price_aggregator_slashing() {
    let (mut world, oracles) = setup();
    let price_aggregator_whitebox = WhiteboxContract::new("sc:price-aggregator", klv_price_aggregator_sc::contract_obj);
    
    // configure the number of decimals
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS),
        |sc| {
            sc.set_pair_decimals(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                DECIMALS,
            )
        },
    );

    // unpause
    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS),|sc| 
            sc.unpause_endpoint(),
    );

    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new()
        .from(&oracles[0]), 
        |sc| {
            sc.vote_slash_member(ManagedAddress::from(&oracles[1]));
        }
    );

    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new()
        .from(&oracles[2]), 
        |sc| {
            sc.vote_slash_member(ManagedAddress::from(&oracles[1]));
        }
    );

    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new()
        .from(&oracles[3]), 
        |sc| {
            sc.vote_slash_member(ManagedAddress::from(&oracles[1]));
        }
    );

    world.whitebox_query(&price_aggregator_whitebox,
        |sc| {
            let list = sc.slashing_proposal_voters(&ManagedAddress::from(&oracles[1]));
            assert!(list.contains(&ManagedAddress::from(&oracles[0])));
            assert!(list.contains(&ManagedAddress::from(&oracles[2])));
            assert!(list.contains(&ManagedAddress::from(&oracles[3])));
        },
    );

    world.whitebox_call(
        &price_aggregator_whitebox,
        ScCallStep::new()
        .from(&oracles[0]), 
        |sc| {
            sc.slash_member(ManagedAddress::from(&oracles[1]));
        }
    );

    // oracle 1 try submit after slashing
    world.whitebox_call_check(
        &price_aggregator_whitebox,
        ScCallStep::new().from(&oracles[1]).no_expect(),
        |sc| {
            sc.submit(
                managed_buffer!(KLV_TICKER),
                managed_buffer!(USD_TICKER),
                95,
                managed_biguint!(10_000),
                DECIMALS,
            )
        },
        |r| {
            r.assert_user_error("only oracles allowed");
        },
    );
}

fn setup() -> (ScenarioWorld, Vec<Address>) {
    // setup
    let mut world = world();
    let price_aggregator_whitebox = WhiteboxContract::new("sc:price-aggregator", klv_price_aggregator_sc::contract_obj);
    
    let price_aggregator_code = world.code_expression(PRICE_AGGREGATOR_PATH_EXPR.eval_to_expr().as_str());
    
    let mut set_state_step = SetStateStep::new()
        .put_account(OWNER_ADDRESS, Account::new().nonce(1))
        .new_address(OWNER_ADDRESS, 2, PRICE_AGGREGATOR_ADDRESS)
        .block_timestamp(100);

    let mut oracles = Vec::new();
    for i in 1..=NR_ORACLES {
        let oracle_address_expr = format!("oracle{i}");
        let oracle_address = TestAddress::new(&oracle_address_expr);

        world.account(oracle_address).nonce(1).balance(STAKE_AMOUNT);
        set_state_step = set_state_step.put_account(
            oracle_address,
            Account::new().nonce(1).balance(STAKE_AMOUNT),
        );

        oracles.push(oracle_address.eval_to_array().into());
    }

    // init price aggregator
    world.set_state_step(set_state_step).whitebox_deploy(
        &price_aggregator_whitebox,
        ScDeployStep::new()
            .from(OWNER_ADDRESS)
            .code(price_aggregator_code),|sc| {
            let mut oracle_args = MultiValueEncoded::new();
            for oracle_address in &oracles {
                oracle_args.push(ManagedAddress::from(oracle_address));
            }

            sc.init(
                TokenIdentifier::klv(),
                managed_biguint!(STAKE_AMOUNT),
                managed_biguint!(SLASH_AMOUNT),
                SLASH_QUORUM,
                SUBMISSION_COUNT,
                oracle_args,
            )
        });

    for oracle_address in &oracles {
        world.whitebox_call(
            &price_aggregator_whitebox,
            ScCallStep::new()
            .from(oracle_address)
            .klv_value(STAKE_AMOUNT), 
            |sc| {
                sc.stake();
            });
    }

    (world, oracles)
}
