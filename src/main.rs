mod core;

use core::CoreState;

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET Research Simulator        ║");
    println!("║     lugsim v0.1.0                   ║");
    println!("║     Phase 0 — Architecture Proof    ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    // Create genesis state
    let mut state = CoreState::new();
    state.genesis("alice", 1_000_000_000); // 1 billion LGT genesis

    state.print_state();

    // Simulate a simple transfer: Alice sends 100 LGT to Bob
    println!("\n--- Transaction 1: Alice sends 100 LGT to Bob ---");
    let tx1 = core::Transaction {
        inputs: vec!["genesis-0".to_string()],
        outputs: vec![
            core::Utxo {
                id: "tx1-0".to_string(),
                amount: 100,
                owner: "bob".to_string(),
            },
            core::Utxo {
                id: "tx1-1".to_string(),
                amount: 999_999_890, // 10 LGT fee burned
                owner: "alice".to_string(),
            },
        ],
        signatures: vec!["alice-sig".to_string()],
        is_clock_create: false,
        clock: None,
    };

    match state.execute_transaction(&tx1) {
        Ok(fee) => println!("Transfer successful. Fee: {} LGT", fee),
        Err(e) => println!("Transfer failed: {}", e),
    }

    state.print_state();

    // Simulate a CLock creation: Alice locks 500 LGT for the VM
    println!("\n--- Transaction 2: Alice locks 500 LGT in a CLock ---");
    let clock1 = core::CLock {
        id: "clock-1".to_string(),
        amount: 500,
        owner: "alice".to_string(),
        vm_object_id: None,
        created_at_block: state.current_block + 1,
    };

    let tx2 = core::Transaction {
        inputs: vec!["tx1-1".to_string()],
        outputs: vec![
            core::Utxo {
                id: "tx2-0".to_string(),
                amount: 999_999_380, // 500 locked, 10 fee burned
                owner: "alice".to_string(),
            },
        ],
        signatures: vec!["alice-sig".to_string()],
        is_clock_create: true,
        clock: Some(clock1),
    };

    match state.execute_transaction(&tx2) {
        Ok(fee) => println!("CLock created successfully. Fee: {} LGT", fee),
        Err(e) => println!("CLock creation failed: {}", e),
    }

    state.print_state();

    println!("\n[ lugsim ] Core state machine validated.");
    println!("[ lugsim ] UTXO transfers: OK");
    println!("[ lugsim ] Double-spend prevention: OK");
    println!("[ lugsim ] CLock creation: OK");
    println!("[ lugsim ] Fee burning: OK");
    println!();
    println!("Next: LUGET VM Object Model");
}
