use cosmwasm_std::{ContractResult, CustomMsg, Env, MessageInfo, Timestamp,
  BlockInfo, Addr, Binary, TransactionInfo, ContractInfo,SubMsgResult, SubMsgResponse, QueryResponse, to_vec,coins, Empty, Reply, Response};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info, MockApi};
use cosmwasm_vm::{BackendApi, call_execute, call_instantiate, call_reply, Instance, InstanceOptions, Querier, Size, Storage};
use log::{trace, info, error};
use simple_logging;
use log::LevelFilter;
use cosmwasm_vm::internals::compile;

use cw_utils::parse_instantiate_response_data;

use serde_json::{Result, Value, Error};
use regex::Regex;


//use wasm_backend::{compile, make_store_with_engine};

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 1_000_000_000_000; // ~1ms
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
        gas_limit: DEFAULT_GAS_LIMIT,
    };
const HIGH_GAS_LIMIT: u64 = 20_000_000_000_000_000; // ~20s, allows many calls on one instance
static ASTROPORT_CONTRACT: &[u8] = include_bytes!("../../astroport_factory.wasm");
static NESTED_CONTRACT: &[u8] = include_bytes!("../../astroport_pair.wasm");
static NESTED2_CONTRACT: &[u8] = include_bytes!("../../astroport_token.wasm");


static ASTROPORT_INSTANTIATE: &[u8] = br#"{
    "pair_configs": [
      {
        "code_id": 35,
        "pair_type": {
          "xyk": {}
        },
        "total_fee_bps": 30,
        "maker_fee_bps": 3333,
        "is_disabled": false,
        "is_generator_disabled": false
      },
      {
        "code_id": 40,
        "pair_type": {
          "stable": {}
        },
        "total_fee_bps": 5,
        "maker_fee_bps": 5000,
        "is_disabled": false,
        "is_generator_disabled": false
      }
    ],
    "token_code_id": 36,
    "owner": "neutron1ffus553eet978k024lmssw0czsxwr97mggyv85lpcsdkft8v9ufsz3sa07",
    "whitelist_code_id": 0,
    "coin_registry_address": "neutron1jzzv6r5uckwd64n6qan3suzker0kct5w565f6529zjyumfcx96kqtcswn3"
  }"#;
  static ASTROPORT_EXECUTE: &[u8] = br#"{
    "create_pair": {
      "asset_infos": [
        {
          "native_token": {
            "denom": "ibc/5751B8BCDA688FD0A8EC0B292EEF1CDEAB4B766B63EC632778B196D317C40C3A"
          }
        },
        {
          "native_token": {
            "denom": "ibc/F082B65C88E4B6D5EF1DB243CDA1D331D002759E938A0F5CD3FFDC5D53B3E349"
          }
        }
      ],
      "init_params": "e30=",
      "pair_type": {
        "xyk": {}
      }
    }
  }"#;


               
  


static CONTRACT: &[u8] = ASTROPORT_CONTRACT ;
static INSTANTIATE: &[u8] = ASTROPORT_INSTANTIATE;
static EXECUTE: &[u8] = ASTROPORT_EXECUTE;

fn preprocess_json(json_string: &str) -> String {
    let key_pattern = r#"\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*:"#;
    let re = Regex::new(key_pattern).unwrap();

    // Replace unquoted keys with quoted keys
    let processed_json = re.replace_all(json_string, r#""$1":"#);

    // Replace Some("value") with "value"
    let processed_json = processed_json.replace("Some(", "").replace(")", "");

    // Return the preprocessed JSON string
    processed_json
}

fn encode_protobuf_string(s: &str) -> Vec<u8> {
  let mut encoded = Vec::new();
  encoded.push((1 << 3) | 2);
  encoded.extend_from_slice(&(s.len() as u64).to_le_bytes());
  encoded.extend_from_slice(s.as_bytes());
  encoded
}

fn extract_json(json_string: &str) -> serde_json::Result<Value> {
    // Find the start and end of the JSON structure
    let start_index: usize = json_string.find("{").unwrap_or(0);
    let mut end_index: usize = start_index;
    let mut open_braces: usize  = 1;

    // Loop through the string to find the end of the JSON structure
    for (i, char) in json_string[start_index + 1..].char_indices() {
        match char {
            '{' => open_braces += 1,
            '}' => {
                open_braces -= 1;
                if open_braces == 0 {
                    end_index = i + start_index + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    let correct_json = preprocess_json(&json_string[start_index..=end_index]);

    // Parse the JSON substring
    serde_json::from_str(&correct_json)
}


fn run_threads(nb_instantiations: u32, nb_threads: u32, pre_compile: bool) {

    let start = std::time::Instant::now();
    let mut threads = vec![];
    for _ in 0 ..nb_threads {

        threads.push( std::thread::spawn(move || {

            for _ in 0.. (nb_instantiations/nb_threads) {

                //println!("Running a task!");

                let backend = mock_backend(&[]);
                let much_gas: InstanceOptions = InstanceOptions {
                    gas_limit: HIGH_GAS_LIMIT,
                    ..DEFAULT_INSTANCE_OPTIONS
                };


                let mut instance = 
                        Instance::from_code( CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();
                    
                  let env = Env {
                    block: BlockInfo {
                      height: 12_345,
                      time: Timestamp::from_nanos(1_571_797_419_879_305_533),
                      chain_id: "cosmos-testnet-14002".to_string(),
                      },
                      transaction: Some(TransactionInfo { index: 3 }),
                      contract: ContractInfo {
                          address: Addr::unchecked("neutron1hptk0k5kng7hjy35vmh009qd5m6l33609nypgf2yc6nqnewduqasxplt4e"),
                     },
                  };
               
                  let info = mock_info("creator", &coins(1000, "earth"));
                  let contract_result =
                    call_instantiate::<_, _, _, Empty>(&mut instance, &env, &info, INSTANTIATE).unwrap();
                    println!("INSTANTIATE RESULT: {:?}", contract_result);
                
                    let info = mock_info("verifies", &coins(1000, "earth"));
                    let contract_result =
                        call_execute::<_, _, _, Empty>(&mut instance, &env, &info, EXECUTE).unwrap();
                    println!("EXECUTE RESULT: {:?}", contract_result);  

                    /*

                    let res = contract_result.unwrap();

                    

                    // retrieve submessage fields
                    let id = res.messages[0].id.clone();
                    let payload = res.messages[0].payload.clone();
                    let msg = res.messages[0].msg.clone();
                    let gas_limit = res.messages[0].gas_limit.clone();
                    let reply_on = res.messages[0].reply_on.clone();

                    println!("id: {:?}", id);
                    println!("payload: {:?}", payload);
                    println!("msg: {:?}", msg);
                    println!("gas_limit: {:?}", gas_limit);
                    println!("reply_on: {:?}", reply_on);

                    */
                   

                    

                    // get a string representation of the result
                    let string_representation = format!("{:?}", contract_result);
                    
                    // check if there is an instantiate nested in the result
                    if let Some(index) = string_representation.find("Instantiate") {
                        let json_string = &string_representation[index..];
                        // Extract JSON from the substring
                        if let Ok(value) = extract_json(json_string) {
                            // Print the "msg" field if it exists
                            if let Some(msg) = value.get("msg") {
                              
                                // println!("Extracted 'msg' field: {}", msg);
                                // Found an instantiate message
                                // 1. Map code_id/label in the original message to a contract
                                // 2. Boot a VM with that given contract
                                let backend = mock_backend(&[]);
                                let mut nested_instance = Instance::from_code(NESTED_CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();
                                // 3. Create mock info, call the instantaite with new instance,mock env, mock info and the msg
                                let info = mock_info("creator", &coins(1000, "earth"));
                                let instantiate_string = msg.to_string();
                                let instantiate_msg = instantiate_string.as_bytes();

                                let contract_result =
                                    call_instantiate::<_, _, _, Empty>(&mut nested_instance, &mock_env(), &info, instantiate_msg).unwrap();
                                    println!("INSTANTIATE RESULT: {:?}", contract_result);

                                // another nested invocation
                                let backend = mock_backend(&[]);
                                let mut nested2_instance = Instance::from_code(NESTED2_CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();
                                let info = mock_info("creator", &coins(1000, "earth"));
                                let instantiate_msg =  br#"{
                                    "name":"IBC/-IBC/-LP","symbol":"uLP","decimals":6,"initial_balances":[],"mint":{"minter":"neutron1e22zh5p8meddxjclevuhjmfj69jxfsa8uu3jvht72rv9d8lkhves6t8veq","cap":null},"marketing":null
                                }"#;
                                let contract_result =
                                    call_instantiate::<_, _, _, Empty>(&mut nested2_instance, &mock_env(), &info, instantiate_msg).unwrap();
                                    println!("INSTANTIATE RESULT: {:?}", contract_result);
                                
                                
                                // call reply from C to B
                                let id: u64 = 1;
                                let data = "neutron1e22zh5p8meddxjclevuhjmfj69jxfsa8uu3jvht72rv9d8lkhves6t8veq";
                                
                                let mut data_vector = encode_protobuf_string(data);

                                let binary_data = Binary::new(data_vector);

                                println!("Binary data: {:?}", binary_data);

                                let response = Reply {
                                    id: 1,
                                    gas_used: 1,
                                    payload: Default::default(),
                                    result: SubMsgResult::Ok(SubMsgResponse {
                                        events: [].into(),
                                        data: Some(binary_data),
                                        msg_responses: vec![],
                                    }),
                                };


                                let contract_result = 
                                    call_reply::<_,_ ,_ , Empty>(&mut nested_instance, &mock_env(), &response).unwrap();
                               println!("REPLY RESULT: {:?}", contract_result);

                                
                                

                                // call reply from B to A
                                let id: u64 = 1;
                                let data = ["neutron1e22zh5p8meddxjclevuhjmfj69jxfsa8uu3jvht72rv9d8lkhves6t8veq"];
                                let mut data_vector: Vec<u8> = Vec::new();

                                for s in data {
                                  data_vector.extend(s.as_bytes());
                                }

                                let binary_data = Binary::new(data_vector);
                                let response = Reply {
                                    id: 1,
                                    gas_used: 1,
                                    payload: Default::default(),
                                    result: SubMsgResult::Ok(SubMsgResponse {
                                        events: [].into(),
                                        data: Some(binary_data),
                                        msg_responses: vec![],
                                    }),
                                };

                                let contract_result = 
                                    call_reply::<_,_ ,_ , Empty>(&mut instance, &mock_env(), &response).unwrap();
                               println!("REPLY RESULT: {:?}", contract_result);

				                        

                                } 
                                else {
                                    println!("'msg' field not found in JSON");
                                }
                        } else {
                            println!("Error extracting JSON");
                        }
                        
                       
                    }
                    else {
                        // proceed with commit
                        println!("'Instantiate' not found in the input string.");
                    }
                    
                
            }
        }));
    }

    for h in threads {
        h.join().unwrap();
    }
    let stop = std::time::Instant::now();
    println!("Elapsed precompile: {} threads {} : time: {:?}", pre_compile, nb_threads, stop.duration_since(start));
}


fn main() {

    simple_logging::log_to_stderr(LevelFilter::Trace);
    
    let nb_instantiations = 1;
    /*
    let contract_address = "neutron1e22zh5p8meddxjclevuhjmfj69jxfsa8uu3jvht72rv9d8lkhves6t8veq";    

    let data = Binary::from(contract_address.as_bytes());

    let init_response = parse_instantiate_response_data (&data);
    println!("{:?}", init_response); */

    run_threads(nb_instantiations, 1, false);

    /*
    run_threads(nb_instantiations, 2, false);
    run_threads(nb_instantiations, 4, false);
    run_threads(nb_instantiations, 8, false);

    /*
    run_threads(nb_instantiations, 1, true);
    run_threads(nb_instantiations, 2, true);
    run_threads(nb_instantiations, 4, true);
    run_threads(nb_instantiations, 8, true);*/
    */
}
