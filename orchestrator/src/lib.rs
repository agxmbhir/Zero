// use hex::decode;
// use hex_literal::hex;
// use std::collections::HashMap;

// use web3::{
//     contract::{Contract, Options},
//     transports::Http,
//     types,
// };

// pub struct Target {
//     chain_id: usize,
//     addr: String,
//     func: String,
//     args: Vec<String>,
//     json: String, //File_path of the ABI
// }
// pub struct Request {
//     chain_Id: String,
//     account: String,
//     transport_url: String,
//     target: Target,
// }

// pub async fn orchestrate<T>(req: Request) {
//     let transport: Http = web3::transports::Http::new(&req.transport_url).unwrap();
//     let web3 = web3::Web3::new(transport);

//     let contract_addr = req.target.addr.trim_start_matches("0x");

//     let contract = Contract::from_json(
//         web3.eth(),
//         types::Address::from_slice(&decode(contract_addr).unwrap()),
//         &req.target.json.as_bytes(),
//     )
//     .unwrap();

//     contract.call(
//         &req.target.func, req.target.args, 
//         types::Address::from_slice(&decode(req.account).unwrap()), Options::default()).await.unwrap();
        
// }
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//     }
// }
