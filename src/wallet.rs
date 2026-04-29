use bitcoin::{key::Secp256k1, secp256k1::SecretKey};
use silentpayments::{
    receiving::{Label, Receiver},
    Network,
};

#[derive(Debug)]
pub struct SpWallet {
    pub receiver: Receiver,
    pub b_scan: SecretKey,
}

impl SpWallet {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let scan_secret = SecretKey::from_slice(&[0x01; 32]).unwrap();
        let spend_secret = SecretKey::from_slice(&[0x02; 32]).unwrap();

        let scan_pk = scan_secret.public_key(&secp);
        let spend_pk = spend_secret.public_key(&secp);

        let change_label = Label::new(scan_secret, 0);
        Self {
            receiver: Receiver::new(0, scan_pk, spend_pk, change_label, Network::Testnet).unwrap(),
            b_scan: scan_secret,
        }
    }

    pub fn get_balance(&self) -> u64 {
        todo!();
    }

    pub fn list_owned_outputs(&self) {
        todo!();
    }
}

impl Default for SpWallet {
    fn default() -> Self {
        Self::new()
    }
}
