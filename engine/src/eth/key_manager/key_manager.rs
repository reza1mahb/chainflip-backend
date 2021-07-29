//! Contains the information required to use the KeyManager contract as a source for
//! the EthEventStreamer

use core::str::FromStr;

use crate::eth::{utils, EventProducerError, EventSource};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use web3::{
    contract::tokens::Tokenizable,
    ethabi::{self, Function, Token},
    types::{BlockNumber, FilterBuilder, H160},
};

use anyhow::Result;

#[derive(Clone)]
/// A wrapper for the KeyManager Ethereum contract.
pub struct KeyManager {
    pub deployed_address: H160,
    contract: ethabi::Contract,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainflipKey {
    pub_key_x: ethabi::Uint,
    pub_key_y_parity: ethabi::Uint,
}

impl Tokenizable for ChainflipKey {
    fn from_token(token: ethabi::Token) -> Result<Self, web3::contract::Error>
    where
        Self: Sized,
    {
        if let Token::Tuple(members) = token {
            if members.len() != 2 {
                Err(web3::contract::Error::InvalidOutputType(
                    stringify!(ChainflipKey).to_owned(),
                ))
            } else {
                Ok(ChainflipKey {
                    pub_key_x: ethabi::Uint::from_token(members[0].clone())?,
                    pub_key_y_parity: ethabi::Uint::from_token(members[1].clone())?,
                })
            }
        } else {
            Err(web3::contract::Error::InvalidOutputType(
                stringify!(ChainflipKey).to_owned(),
            ))
        }
    }

    fn into_token(self) -> ethabi::Token {
        Token::Tuple(vec![
            // Key
            Token::Uint(self.pub_key_x),
            Token::Uint(self.pub_key_y_parity),
        ])
    }
}

/// Represents the events that are expected from the KeyManager contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyManagerEvent {
    /// The `Staked(nodeId, amount)` event.
    KeyChange {
        /// Whether the change was signed by the AggKey.
        signed: bool,
        /// The old key.
        old_key: ChainflipKey,
        /// The new key.
        new_key: ChainflipKey,
    },
}

impl KeyManager {
    /// Loads the contract abi to get event definitions
    pub fn load(deployed_address: &str) -> Result<Self> {
        Ok(Self {
            deployed_address: H160::from_str(deployed_address)?,
            contract: ethabi::Contract::load(
                std::include_bytes!("../abis/KeyManager.json").as_ref(),
            )?,
        })
    }

    /// Extracts a reference to the "setAggKeyWithAggKey" function definition. Panics if it can't be found.
    pub fn set_agg_key_with_agg_key(&self) -> &Function {
        self.contract
            .function("setAggKeyWithAggKey")
            .expect("Function 'setAggKeyWithAggKey' should be defined in the KeyManager abi.")
    }

    /// Event definition for the 'Staked' event
    pub fn key_change_event_definition(&self) -> &ethabi::Event {
        self.get_event("KeyChange")
            .expect("KeyManager contract should provide 'KeyChange' event.")
    }

    // TODO: move this to a common place with stake manager?
    // Get the event type definition from the contract abi
    fn get_event(&self, name: &str) -> Result<&ethabi::Event> {
        Ok(self.contract.event(name)?)
    }
}

impl Display for KeyManagerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            KeyManagerEvent::KeyChange {
                signed,
                old_key,
                new_key,
            } => write!(f, "KeyChange({}, {:?}, {:?}", signed, old_key, new_key),
        }
    }
}

impl EventSource for KeyManager {
    type Event = KeyManagerEvent;

    fn filter_builder(&self, block: BlockNumber) -> FilterBuilder {
        FilterBuilder::default()
            .from_block(block)
            .address(vec![self.deployed_address])
    }

    fn parse_event(&self, log: web3::types::Log) -> Result<Self::Event> {
        let sig = log
            .topics
            .first()
            .ok_or_else(|| EventProducerError::EmptyTopics)?
            .clone();

        let raw_log = ethabi::RawLog {
            topics: log.topics,
            data: log.data.0,
        };

        log::debug!(
            "Parsing event from block {:?} with signature: {:?}",
            log.block_number.unwrap_or_default(),
            sig
        );

        match sig {
            _ if sig == self.key_change_event_definition().signature() => {
                let log = self.key_change_event_definition().parse_log(raw_log)?;

                let event = KeyManagerEvent::KeyChange {
                    signed: utils::decode_log_param::<bool>(&log, "signedByAggKey")?,
                    old_key: utils::decode_log_param::<ChainflipKey>(&log, "oldKey")?,
                    new_key: utils::decode_log_param::<ChainflipKey>(&log, "newKey")?,
                };
                Ok(event)
            }
            s => Err(EventProducerError::UnexpectedEvent(s))?,
        }
    }
}

#[cfg(test)]
mod tests {

    use web3::types::{H256, U256};

    use super::*;

    // 🔑 Aggregate Key sets the new Aggregate Key 🔑
    const AGG_SET_AGG_LOG: &'static str = r#"{
        "logIndex": "0x0",
        "transactionIndex": "0x0",
        "transactionHash": "0x04629152b064c0d1343161c43f3b78cf67e9be35fc97f66bbb0e1ca1a0206bae", 
        "blockHash": "0x68c5dfba660af922463f3d47c76b551760161711e9341cf8563bae7e146f6b8d", 
        "blockNumber": "0xC5064B", 
        "address": "0xD537bF4b795b7D07Bd5F4bAf7017e3ce8360B1DE", 
        "data": "0x000000000000000000000000000000000000000000000000000000000000000131b2ba4b46201610901c5164f42edd1f64ce88076fde2e2c544f9dc3d7b350ae00000000000000000000000000000000000000000000000000000000000000011742daacd4dbfbe66d4c8965550295873c683cb3b65019d3a53975ba553cc31d0000000000000000000000000000000000000000000000000000000000000001", 
        "topics": ["0x19389c59b816d8b0ec43f2d5ed9b41bddc63d66dac1ecd808efe35b86b9ee0bf"],
        "type": "mined",
        "removed": false
    }"#;

    // 🔑 Governance Key sets the new Aggregate Key 🔑
    const GOV_SET_AGG_LOG: &'static str = r#"{
        "logIndex": "0x0", 
        "transactionIndex": "0x0", 
        "transactionHash": "0x6320cfd702415644192bf57702ceccc0d6de0ddc54fe9aa53f9b1a5d9035fe52", 
        "blockHash": "0x042a88e77cb7455f72f15b806dc88304ce113a0a39a03274712e31274bb8fbfa", 
        "blockNumber": "0xC5064C", 
        "address": "0xD537bF4b795b7D07Bd5F4bAf7017e3ce8360B1DE", 
        "data": "0x00000000000000000000000000000000000000000000000000000000000000001742daacd4dbfbe66d4c8965550295873c683cb3b65019d3a53975ba553cc31d000000000000000000000000000000000000000000000000000000000000000131b2ba4b46201610901c5164f42edd1f64ce88076fde2e2c544f9dc3d7b350ae0000000000000000000000000000000000000000000000000000000000000001", 
        "topics": ["0x19389c59b816d8b0ec43f2d5ed9b41bddc63d66dac1ecd808efe35b86b9ee0bf"], 
        "type": "mined", 
        "removed": false
    }"#;

    // 🔑 Governance Key sets the new Governance Key 🔑
    const GOV_SET_GOV_LOG: &'static str = r#"{
        "logIndex": "0x0", 
        "transactionIndex": "0x0", 
        "transactionHash": "0x9215ce54309fddf0ce9b1e8fd10319c62cf9603635ffa0c06ac9db8338348f95", 
        "blockHash": "0x55d818c9efc4b9d6ac54609f779c06df7bc92919c7ac3fa123d178205ffea351", 
        "blockNumber": "0xC5064D", 
        "address": "0xD537bF4b795b7D07Bd5F4bAf7017e3ce8360B1DE", 
        "data": "0x0000000000000000000000000000000000000000000000000000000000000000423ebe9d54bf7cb10dfebe2b323bb9a01bfede660619a7f49531c96a23263dd800000000000000000000000000000000000000000000000000000000000000014e3d72babbee4133675d42db3bba62a7dfbc47a91ddc5db56d95313d908c08f80000000000000000000000000000000000000000000000000000000000000000", 
        "topics": ["0x19389c59b816d8b0ec43f2d5ed9b41bddc63d66dac1ecd808efe35b86b9ee0bf"], 
        "type": "mined", 
        "removed": false
    }"#;

    const KEY_CHANGE_EVENT_SIG: &'static str =
        "0x19389c59b816d8b0ec43f2d5ed9b41bddc63d66dac1ecd808efe35b86b9ee0bf";

    const CONTRACT_ADDRESS: &'static str = "0xD537bF4b795b7D07Bd5F4bAf7017e3ce8360B1DE";

    #[test]
    fn test_key_change_parsing() -> anyhow::Result<()> {
        println!("Loading KeyManager");
        let km = KeyManager::load(CONTRACT_ADDRESS)?;

        println!("Parsing event 1");
        match km.parse_event(serde_json::from_str(AGG_SET_AGG_LOG)?)? {
            KeyManagerEvent::KeyChange {
                signed,
                old_key,
                new_key,
            } => {
                assert_eq!(signed, true);

                let expected_key  = ChainflipKey{
                    pub_key_x: U256::from_dec_str("22479114112312168431982914496826057754130808976066989807481484372215659188398").unwrap(),
                    pub_key_y_parity: U256::from_dec_str("1").unwrap(),
                };
                assert_eq!(old_key, expected_key);

                let expected_key  = ChainflipKey{
                    pub_key_x: U256::from_dec_str("10521316663921629387264629518161886172223783929820773409615991397525613232925").unwrap(),
                    pub_key_y_parity: U256::from_dec_str("1").unwrap(),
                };
                assert_eq!(new_key, expected_key);
            }
        }

        println!("Parsing event 2");
        match km.parse_event(serde_json::from_str(GOV_SET_AGG_LOG)?)? {
            KeyManagerEvent::KeyChange {
                signed,
                old_key,
                new_key,
            } => {
                assert_eq!(signed, false);

                let expected_key  = ChainflipKey{
                    pub_key_x: U256::from_dec_str("10521316663921629387264629518161886172223783929820773409615991397525613232925").unwrap(),
                    pub_key_y_parity: U256::from_dec_str("1").unwrap(),
                };
                assert_eq!(old_key, expected_key);

                let expected_key  = ChainflipKey{
                    pub_key_x: U256::from_dec_str("22479114112312168431982914496826057754130808976066989807481484372215659188398").unwrap(),
                    pub_key_y_parity: U256::from_dec_str("1").unwrap(),
                };
                assert_eq!(new_key, expected_key);
            }
        }

        println!("Parsing event 3");
        match km.parse_event(serde_json::from_str(GOV_SET_GOV_LOG)?)? {
            KeyManagerEvent::KeyChange {
                signed,
                old_key,
                new_key,
            } => {
                assert_eq!(signed, false);

                let expected_key  = ChainflipKey{
                    pub_key_x: U256::from_dec_str("29963508097954364125322164523090632495724997135004046323041274775773196467672").unwrap(),
                    pub_key_y_parity: U256::from_dec_str("1").unwrap(),
                };
                assert_eq!(old_key, expected_key);

                let expected_key  = ChainflipKey{
                    pub_key_x: U256::from_dec_str("35388971693871284788334991319340319470612669764652701045908837459480931993848").unwrap(),
                    pub_key_y_parity: U256::from_dec_str("0").unwrap(),
                };
                assert_eq!(new_key, expected_key);
            }
        }

        Ok(())
    }

    #[test]
    fn abi_topic_sigs() -> anyhow::Result<()> {
        let km = KeyManager::load(CONTRACT_ADDRESS)?;

        // key change event
        assert_eq!(
            km.key_change_event_definition().signature(),
            H256::from_str(KEY_CHANGE_EVENT_SIG)
                .expect("Couldn't cast key change event sig to H256"),
            "key change event doesn't match signature"
        );

        Ok(())
    }
}
