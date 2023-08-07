import * as dotenv from 'dotenv'
import { MnemonicKey, MsgStoreCode, MsgInstantiateContract, LCDClient, Coins } from '@terra-money/feather.js';
import * as fs from 'fs';

dotenv.config()

const init = async () => {
    // Check if the oracle contract is deployed
    // and with the proper information stored in the file
    if (!fs.existsSync('./scripts/.oracle_address.log') 
        || fs.readFileSync('./scripts/.oracle_address.log').toString('utf-8') == "") {
        console.log(`Pleae deploy the oracle contract first or add it's address to the ./scripts/.oracle_address.log file to run this script`);
        return;
    }

    // Variable will populate after storing the code on chain
    let codeId: number;

    // Create the LCD Client to interact with the blockchain
    const lcd = LCDClient.fromDefaultConfig("testnet")

    const govAccountAddr = (await lcd.auth.moduleAccountInfo("pisco-1","gov"))?.baseAccount?.address;
    if (govAccountAddr == undefined) {
        console.log(`Something went wrong retreiving the governance account from on-chain`);
        return;
    }

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");
    console.log(`Wallet address: ${accAddress}`)

    // Create the message and broadcast the transaction on chain
    const msgStoreCode = new MsgStoreCode(
        accAddress,
        fs.readFileSync('./artifacts/alliance_hub.wasm').toString('base64')
    );
    let tx = await wallet.createAndSignTx({
        msgs: [msgStoreCode],
        memo: "Alliance Hub Contract",
        chainID: "pisco-1",
    });

    let result = await lcd.tx.broadcastBlock(tx, "pisco-1");
    codeId = Number(result.logs[0].events[1].attributes[1].value);
    console.log(`Smart contract deployed with 
    - Code ID: ${codeId}
    - Tx Hash: ${result.txhash}`);
    
    await new Promise(resolve => setTimeout(resolve, 3000));

    const oracleAddress = fs.readFileSync('./scripts/.oracle_address.log').toString('utf-8');

    // Instantiate the transaction and broadcast it on chain
    const msgInstantiateContract = new MsgInstantiateContract(
        accAddress,
        accAddress,
        codeId,
        {
            "controller": accAddress,
            "governance": govAccountAddr,
            "oracle" : oracleAddress,
            "reward_denom": "uluna",
        },
        Coins.fromString("10000000uluna"),
        "Create an Hub contract"
    );

    tx = await wallet.createAndSignTx({
        msgs: [msgInstantiateContract],
        memo: "Create an Alliance Hub Contract",
        chainID: "pisco-1",
    });
    result = await lcd.tx.broadcastBlock(tx, "pisco-1");
    const contractAddress = result.logs[0].events[0].attributes[0].value;
    console.log(`Alliance Hub smart contract instantiated with 
    - Code ID: ${codeId}
    - Tx Hash: ${result.txhash}
    - Contract Address: ${contractAddress}`);
    fs.writeFileSync('./scripts/.hub_address.log', contractAddress);
}

try {
    init();
}
catch (e) {
    console.log(e)
}