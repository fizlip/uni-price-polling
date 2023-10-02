/**
 *
 * This program will return the price of an asset on uniswap using a 2-hop 
 * strategy. We're basically doing the following calculation:
 *
 * TOKEN / ETH * ETH / USDT = TOKEN / USDT
 *
 * TOKEN is the ERC20 token you want to know the price of. 
 *
 * To use the program you need to input the address of the UNI V2 pool address
 * of the TOKEN / ETH pool.
 * 
**/

use ethers::{
    prelude::{abigen, ContractError},
    providers::{Http, Provider},
    types::Address,
};

use std::future::Future;
use std::sync::Arc;
use clap::Parser;

const RPC_URL: &str = "https://eth.llamarpc.com";

abigen!(
    IUniswapV2Pair,
    "[function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)]"
);

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub pool: String 
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    
    // Initialize provider and define addresses
    let provider = Arc::new(Provider::try_from(RPC_URL)?);
    let args = Args::parse();
    //let start_a: Address = "0xa2107fa5b38d9bbd2c461d6edf11b11a50f6b974".parse()?;
    let start_a: Address = args.pool.parse()?;
    let end_a: Address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852".parse()?;


    let p1 = provider.clone();

    // Gets UNI reserves of the assets in TOKEN/ETH pool and ETH/USDT pool
    let (token_1, eth_1, _) = get_reserves(provider, &start_a).await?;
    let (eth_2, usdt_1, _) = get_reserves(p1, &end_a).await?;


    // Reformat into decimals
    let (token_f64, eth_f64) = (reformat_wei(token_1), reformat_wei(eth_1));
    let (eth2_f64, usdt_f64) = (reformat_wei(eth_2), reformat_usd(usdt_1));

    // Get UNI V2 price of link
    let token_per_usdt = 1.0 / 
        ((token_f64 / eth_f64) * 
        (eth2_f64 / usdt_f64));

    
    println!("[UNI V2] LINK/USDT: ${token_per_usdt}");

    Ok(())

}

/**
 * @gist converts wei into eth values, 1 ETH = 10**16 wei
 * @param wei_int -- this is the wei value to be converted into eth
 * @output -- ETH value
**/
fn reformat_wei(wei_int: u128) -> f64 {
    wei_int as f64 / 10_f64.powf(16.0)
}

/**
 * @gist converts the USDT values in UNI pools into decimals
 * @param usd_int -- this is the usdt value returned by a UNI pool
 * @output USDT value
**/
fn reformat_usd(usd_int:u128) -> f64 {
    usd_int as f64 / 10_f64.powf(4.0)
}

/**
 * @gist get_reserves returns the reserves of tokens in a given uniswap pool.
 * @param provider -- this is used to send request to the UniswapV2Pair SC 
 * @param pair_address -- the pair address you want the reserves from
 * @output a future object containing the values of the reserves and a timestamp.
**/
fn get_reserves<'a>(provider: Arc<Provider<Http>>, pair_address: &'a Address) -> impl Future<Output = Result<(u128, u128, u32), ContractError<Provider<Http>>>> + 'a {

    async move {
        let uniswap_v2_pair = IUniswapV2Pair::new(*pair_address, provider);
        uniswap_v2_pair.get_reserves().call().await
    }
}
