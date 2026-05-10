use std::env;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use blake3::Hasher;
use hex;

struct KeyPair {
    public_key: VerifyingKey,
    secret_key: SigningKey,
    address: String,
}

impl KeyPair {
    fn generate() -> Self {
        let secret_key = SigningKey::generate(&mut OsRng);
        let public_key = secret_key.verifying_key();
        let address = Self::hash_bytes(&public_key.to_bytes());
        KeyPair { public_key, secret_key, address }
    }

    fn sign(&self, message: &[u8]) -> Vec<u8> {
        Signer::sign(&self.secret_key, message).to_vec()
    }

    fn verify(public_key: &VerifyingKey, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = Signature::from_slice(signature) {
            Verifier::verify(public_key, message, &sig).is_ok()
        } else {
            false
        }
    }

    fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hex::encode(hasher.finalize().as_bytes())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WalletFile {
    version: String,
    address: String,
    public_key_hex: String,
    secret_key_hex: String,
    created_at: String,
}

const WALLET_DIR: &str = ".luget";
const WALLET_FILE: &str = "wallet.json";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "generate" => cmd_generate(),
        "address" => cmd_address(),
        "balance" => cmd_balance(),
        "send" => {
            if args.len() < 5 {
                println!("Usage: lugwallet send <to_address> <amount> <fee>");
                return;
            }
            cmd_send(&args[2], args[3].parse().unwrap_or(0), args[4].parse().unwrap_or(0));
        }
        "info" => cmd_info(),
        "sign" => {
            if args.len() < 3 {
                println!("Usage: lugwallet sign <message>");
                return;
            }
            cmd_sign(&args[2]);
        }
        "--version" | "-v" => println!("lugwallet v0.8.0"),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET CLI Wallet                ║");
    println!("║     lugwallet v0.8.0                ║");
    println!("╚══════════════════════════════════════╝");
    println!();
    println!("Usage:");
    println!("  lugwallet generate              Generate a new keypair");
    println!("  lugwallet address               Show your address");
    println!("  lugwallet balance               Check balance (simulated)");
    println!("  lugwallet send <to> <amt> <fee> Create and sign a transaction");
    println!("  lugwallet sign <message>        Sign a message");
    println!("  lugwallet info                  Show wallet details");
}

fn get_wallet_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/{}/{}", home, WALLET_DIR, WALLET_FILE)
}

fn load_wallet() -> Result<WalletFile, String> {
    let path = get_wallet_path();
    if !Path::new(&path).exists() {
        return Err("No wallet found. Run 'lugwallet generate' first.".to_string());
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {}", e))
}

fn save_wallet(wallet: &WalletFile) -> Result<(), String> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = format!("{}/{}", home, WALLET_DIR);
    let _ = fs::create_dir_all(&dir);
    let path = get_wallet_path();
    let json = serde_json::to_string_pretty(wallet).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Write error: {}", e))?;
    println!("Wallet saved to {}", path);
    Ok(())
}

fn cmd_generate() {
    println!("Generating new LUGET keypair...");
    let keypair = KeyPair::generate();
    let wallet = WalletFile {
        version: "0.8.0".to_string(),
        address: keypair.address.clone(),
        public_key_hex: hex::encode(keypair.public_key.to_bytes()),
        secret_key_hex: hex::encode(keypair.secret_key.to_bytes()),
        created_at: "2026-05-10".to_string(),
    };
    save_wallet(&wallet).unwrap();
    println!();
    println!("╔══════════════════════════════════════╗");
    println!("║  LUGET Wallet Generated             ║");
    println!("║  Address: {}  ║", &wallet.address[..48]);
    println!("║           {}  ║", &wallet.address[48..]);
    println!("╚══════════════════════════════════════╝");
    println!();
    println!("Never share your secret key with anyone.");
}

fn cmd_address() {
    match load_wallet() {
        Ok(w) => println!("{}", w.address),
        Err(e) => println!("{}", e),
    }
}

fn cmd_balance() {
    match load_wallet() {
        Ok(w) => {
            println!("Address: {}...", &w.address[..32]);
            println!("Balance: 0 LGT (simulated — node query available at testnet)");
        }
        Err(e) => println!("{}", e),
    }
}

fn cmd_send(to: &str, amount: u64, fee: u64) {
    match load_wallet() {
        Ok(w) => {
            let secret_bytes: [u8; 32] = hex::decode(&w.secret_key_hex).unwrap()[..32].try_into().unwrap();
            let secret_key = SigningKey::from_bytes(&secret_bytes);
            let tx_data = format!("{}:{}:{}:{}", w.address, to, amount, fee);
            let signature = Signer::sign(&secret_key, tx_data.as_bytes());
            let sig_hex = hex::encode(signature.to_vec());
            let mut hasher = Hasher::new();
            hasher.update(tx_data.as_bytes());
            hasher.update(&signature.to_vec());
            let tx_hash = hex::encode(hasher.finalize().as_bytes());

            println!("╔══════════════════════════════════════╗");
            println!("║  Transaction Signed                 ║");
            println!("║  Tx Hash: {}  ║", &tx_hash[..48]);
            println!("║  From:    {}...  ║", &w.address[..32]);
            println!("║  To:      {}  ║", to);
            println!("║  Amount:  {} LGT                    ║", amount);
            println!("║  Fee:     {} LGT                    ║", fee);
            println!("║  Sig:     {}...  ║", &sig_hex[..32]);
            println!("║  Status:  SIGNED ✓                  ║");
            println!("╚══════════════════════════════════════╝");
        }
        Err(e) => println!("{}", e),
    }
}

fn cmd_sign(message: &str) {
    match load_wallet() {
        Ok(w) => {
            let secret_bytes: [u8; 32] = hex::decode(&w.secret_key_hex).unwrap()[..32].try_into().unwrap();
            let secret_key = SigningKey::from_bytes(&secret_bytes);
            let signature = Signer::sign(&secret_key, message.as_bytes());
            println!("Message:   {}", message);
            println!("Signer:    {}...", &w.address[..32]);
            println!("Signature: {}...", &hex::encode(signature.to_vec())[..32]);
            println!("Status:    SIGNED ✓");
        }
        Err(e) => println!("{}", e),
    }
}

fn cmd_info() {
    match load_wallet() {
        Ok(w) => {
            println!("╔══════════════════════════════════════╗");
            println!("║  Wallet Info                        ║");
            println!("║  Version: {}                       ║", w.version);
            println!("║  Created: {}          ║", w.created_at);
            println!("║  Address: {}  ║", &w.address[..32]);
            println!("║           {}  ║", &w.address[32..]);
            println!("╚══════════════════════════════════════╝");
        }
        Err(e) => println!("{}", e),
    }
}
