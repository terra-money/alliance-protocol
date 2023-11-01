import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgRegisterFeeShare, MsgUpdateFeeShare } from '@terra-money/feather.js';
import * as fs from 'fs';

dotenv.config()

const init = async () => {
    const lcd = LCDClient.fromDefaultConfig("testnet")
    const oracleAddress = fs.readFileSync('./scripts/.oracle_address.log').toString('utf-8');
    const hubAddress = fs.readFileSync('./scripts/.hub_address.log').toString('utf-8');

    try {
        const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
        const randomaddr = new MnemonicKey().accAddress("terra");
        const wallet = lcd.wallet(mk);
        const accAddress = wallet.key.accAddress("terra");
        let tx = await wallet.createAndSignTx({
            msgs: [
                new MsgUpdateFeeShare(
                    oracleAddress,
                    accAddress,
                    randomaddr,
                ),
                new MsgUpdateFeeShare(
                    hubAddress,
                    accAddress,
                    randomaddr,
                ),
            ],
            memo: "Register feeshare",
            chainID: "pisco-1",
        });
        let result = await lcd.tx.broadcastSync(tx, "pisco-1");
        console.log("result", result)
    }
    catch (e) {
        console.log(e)
    }
}

try {
    init();
}
catch (e) {
    console.log(e)
}