use web3::{
    types::{Address, H256, FilterBuilder, BlockNumber},
     Web3,
};
use hex_literal::hex;
#[tokio::main]
async fn main() {
    let transport = web3::transports::Http::new("https://polygon-mumbai.g.alchemy.com/v2/VQcG-3SB9Xd0q7tE6Koq4cxYTUHq3wDN").unwrap();
    let web3 = Web3::new(transport);

    let contract_address = "0xC36442b4a4522E871399CD717aBDD847Ab11FE88".parse::<Address>().unwrap();
    let event_topic = hex!("3067048beee31b25b2f1681f88dac838c8bba36af25bfb2b7cf7473a5847e35f").into();
    let from_block = BlockNumber::Earliest;
    let filter = FilterBuilder::default()
    .address(vec![contract_address])
    .topics(Some(vec![event_topic]), None, None, None)
    .build();

    loop {
        let logs = web3.eth().logs(filter.clone()).await.unwrap();
        for log in logs.iter() {
            println!("{:?}", log)
        }
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
    
}