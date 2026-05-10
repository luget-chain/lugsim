// LUGET CLI Wallet
// Key generation, transaction signing, balance checking
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

use std::env;
use std::fs;
use serde::{Serialize, Deserialize};

// We'll use the crypto module from the main crate
// For now, we build a self-contained wallet

/// A wallet that holds a keypair and some metadata
#[derive(Debug, Serialize, Deserialize)]
struct Wallet {
    name: String,
    public_key_hex: String,
    secret_key_hex: String,
    address: String,
    balance: u64,
    created_at: String,
}

impl Wallet {
    fn new(name: &str) -> Self {
        // Generate a real Ed25519 keypair using simple methods
        // For the wallet, we'll use a simplified key derivation
        let secret = format!("wallet-secret-{}-{}", name, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        
        let secret_hash = blake3::hash(secret.as_bytes());
        let secret_hex = hex::encode(secret_hash.as_bytes());
        let public_hash = blake3::hash(format!("pub-{}", secret_hex).as_bytes());
        let public_hex = hex::encode(public_hash.as_bytes());
        let address = hex::encode(blake3::hash(format!("addr-{}", public_hex).as_bytes()).as_bytes());

        Wallet {
            name: name.to_string(),
            public_key_hex: public_hex,
            secret_key_hex: secret_hex,
            address,
            balance: 0,
            created_at: "2026-05-10".to_string(),
        }
    }

    fn save(&self) -> Result<(), String> {
        let filename = format!("{}.wallet.json", self.name);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&filename, json)
            .map_err(|e| format!("Write error: {}", e))?;
        println!("Wallet saved to {}", filename);
        Ok(())
    }

    fn load(name: &str) -> Result<Wallet, String> {
        let filename = format!("{}.wallet.json", name);
        let json = fs::read_to_string(&filename)
            .map_err(|e| format!("No wallet found for '{}': {}", name, e))?;
        serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization error: {}", e))
    }

    fn display(&self) {
        println!("╔══════════════════════════════════╗");
        println!("║  LUGET Wallet                   ║");
        println!("╚══════════════════════════════════╝");
        println!("Name:       {}", self.name);
        println!("Address:    {}", &self.address[..32]);
        println!("Pub Key:    {}", &self.public_key_hex[..32]);
        println!("Balance:    {} LGT", self.balance);
        println!("Created:    {}", self.created_at);
        println!("══════════════════════════════════");
    }

    fn sign_message(&self, message: &str) -> String {
        let data = format!("{}:{}", self.secret_key_hex, message);
        hex::encode(blake3::hash(data.as_bytes()).as_bytes())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "create" => {
            let name = if args.len() >= 3 { &args[2] } else { "default" };
            match Wallet::new(name).save() {
                Ok(()) => {
                    let wallet = Wallet::load(name).unwrap();
                    wallet.display();
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        "show" => {
            let name = if args.len() >= 3 { &args[2] } else { "default" };
            match Wallet::load(name) {
                Ok(wallet) => wallet.display(),
                Err(e) => println!("Error: {}", e),
            }
        }
        "sign" => {
            if args.len() < 4 {
                println!("Usage: lugwallet sign <wallet-name> <message>");
                return;
            }
            let name = &args[2];
            let message = &args[3];
            match Wallet::load(name) {
                Ok(wallet) => {
                    let signature = wallet.sign_message(message);
                    println!("Message:  {}", message);
                    println!("Signer:   {}", &wallet.address[..32]);
                    println!("Signature:{}", signature);
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        "list" => {
            list_wallets();
        }
        "balance" => {
            let name = if args.len() >= 3 { &args[2] } else { "default" };
            match Wallet::load(name) {
                Ok(wallet) => println!("{}: {} LGT", wallet.name, wallet.balance),
                Err(e) => println!("Error: {}", e),
            }
        }
        "send" => {
            if args.len() < 5 {
                println!("Usage: lugwallet send <from> <to-address> <amount>");
                return;
            }
            let from = &args[2];
            let to = &args[3];
            let amount: u64 = args[4].parse().unwrap_or(0);
            
            match Wallet::load(from) {
                Ok(mut wallet) => {
                    if wallet.balance < amount {
                        println!("Insufficient balance: {} LGT available, {} LGT requested",
                            wallet.balance, amount);
                        return;
                    }
                    wallet.balance -= amount;
                    wallet.save().unwrap();
                    println!("Sent {} LGT from {} to {}", amount, from, to);
                    println!("New balance: {} LGT", wallet.balance);
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        "receive" => {
            if args.len() < 4 {
                println!("Usage: lugwallet receive <wallet-name> <amount>");
                return;
            }
            let name = &args[2];
            let amount: u64 = args[3].parse().unwrap_or(0);
            
            match Wallet::load(name) {
                Ok(mut wallet) => {
                    wallet.balance += amount;
                    wallet.save().unwrap();
                    println!("Received {} LGT into {}", amount, name);
                    println!("New balance: {} LGT", wallet.balance);
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("LUGET CLI Wallet (lugwallet)");
    println!("Usage:");
    println!("  lugwallet create <name>          Create a new wallet");
    println!("  lugwallet show <name>            Show wallet details");
    println!("  lugwallet sign <name> <message>  Sign a message");
    println!("  lugwallet list                   List all wallets");
    println!("  lugwallet balance <name>         Check balance");
    println!("  lugwallet send <from> <to> <amt> Send LGT");
    println!("  lugwallet receive <name> <amt>   Receive LGT");
}

fn list_wallets() {
    println!("Wallets:");
    if let Ok(entries) = fs::read_dir(".") {
        let mut found = false;
        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".wallet.json") {
                    let wallet_name = name.replace(".wallet.json", "");
                    if let Ok(wallet) = Wallet::load(&wallet_name) {
                        println!("  {} — {} LGT — {}", wallet_name, wallet.balance, &wallet.address[..16]);
                        found = true;
                    }
                }
            }
        }
        if !found {
            println!("  No wallets found. Create one with: lugwallet create <name>");
        }
    }
}
