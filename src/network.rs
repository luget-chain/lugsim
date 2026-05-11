// LUGET Networking Module
// Multi-node P2P communication over TCP
// Kipngetich Clinton, Kenya

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// A message that can be sent between nodes
#[derive(Debug, Clone)]
pub enum Message {
    Transaction { tx_data: String, from: String },
    BlockProposal { block_height: u64, proposer: String, tx_count: usize },
    Vote { block_height: u64, voter: String, approve: bool },
    EpochStateRoot { epoch: u64, root: String },
    PeerHello { node_id: String, port: u16 },
}

impl Message {
    /// Serialize to a string for transmission
    pub fn serialize(&self) -> String {
        match self {
            Message::Transaction { tx_data, from } => {
                format!("TX|{}|{}", from, tx_data)
            }
            Message::BlockProposal { block_height, proposer, tx_count } => {
                format!("BLOCK|{}|{}|{}", block_height, proposer, tx_count)
            }
            Message::Vote { block_height, voter, approve } => {
                format!("VOTE|{}|{}|{}", block_height, voter, approve)
            }
            Message::EpochStateRoot { epoch, root } => {
                format!("EPOCH|{}|{}", epoch, root)
            }
            Message::PeerHello { node_id, port } => {
                format!("HELLO|{}|{}", node_id, port)
            }
        }
    }

    /// Deserialize from a string
    pub fn deserialize(data: &str) -> Option<Message> {
        let parts: Vec<&str> = data.split('|').collect();
        if parts.is_empty() {
            return None;
        }
        match parts[0] {
            "TX" if parts.len() >= 3 => Some(Message::Transaction {
                tx_data: parts[2].to_string(),
                from: parts[1].to_string(),
            }),
            "BLOCK" if parts.len() >= 4 => Some(Message::BlockProposal {
                block_height: parts[1].parse().unwrap_or(0),
                proposer: parts[2].to_string(),
                tx_count: parts[3].parse().unwrap_or(0),
            }),
            "VOTE" if parts.len() >= 4 => Some(Message::Vote {
                block_height: parts[1].parse().unwrap_or(0),
                voter: parts[2].to_string(),
                approve: parts[3] == "true",
            }),
            "EPOCH" if parts.len() >= 3 => Some(Message::EpochStateRoot {
                epoch: parts[1].parse().unwrap_or(0),
                root: parts[2].to_string(),
            }),
            "HELLO" if parts.len() >= 3 => Some(Message::PeerHello {
                node_id: parts[1].to_string(),
                port: parts[2].parse().unwrap_or(0),
            }),
            _ => None,
        }
    }
}

/// A network node that can send and receive messages
pub struct Node {
    pub node_id: String,
    pub port: u16,
    pub peers: Arc<Mutex<Vec<(String, u16)>>>,  // (node_id, port)
    pub inbox: Arc<Mutex<Vec<Message>>>,
}

impl Node {
    /// Create a new node
    pub fn new(node_id: &str, port: u16) -> Self {
        Node {
            node_id: node_id.to_string(),
            port,
            peers: Arc::new(Mutex::new(Vec::new())),
            inbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a peer to connect to
    pub fn add_peer(&mut self, node_id: &str, port: u16) {
        self.peers.lock().unwrap().push((node_id.to_string(), port));
    }

    /// Start listening for incoming connections
    pub fn start_listening(&self) -> thread::JoinHandle<()> {
        let inbox = Arc::clone(&self.inbox);
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .expect("Failed to bind to port");

        println!("[NET] Node {} listening on port {}", self.node_id, self.port);

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = [0u8; 1024];
                        match stream.read(&mut buffer) {
                            Ok(size) => {
                                let data = String::from_utf8_lossy(&buffer[..size]);
                                for line in data.lines() {
                                    if let Some(msg) = Message::deserialize(line) {
                                        inbox.lock().unwrap().push(msg);
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    Err(_) => {}
                }
            }
        })
    }

    /// Send a message to a specific peer
    pub fn send_to(&self, port: u16, msg: &Message) {
        let serialized = msg.serialize() + "\n";
        if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{}", port)) {
            let _ = stream.write_all(serialized.as_bytes());
        }
    }

    /// Broadcast a message to all known peers
    pub fn broadcast(&self, msg: &Message) {
        let peers = self.peers.lock().unwrap();
        for (_id, port) in peers.iter() {
            self.send_to(*port, msg);
        }
    }

    /// Get all messages received since last check
    pub fn fetch_messages(&self) -> Vec<Message> {
        let mut inbox = self.inbox.lock().unwrap();
        let msgs = inbox.clone();
        inbox.clear();
        msgs
    }
}

/// Simulate a validator node with networking
pub struct ValidatorNode {
    pub node: Node,
    pub block_height: u64,
    pub pending_tx_count: usize,
}

impl ValidatorNode {
    pub fn new(node_id: &str, port: u16) -> Self {
        ValidatorNode {
            node: Node::new(node_id, port),
            block_height: 0,
            pending_tx_count: 0,
        }
    }

    /// Propose a new block
    pub fn propose_block(&self) -> Message {
        Message::BlockProposal {
            block_height: self.block_height + 1,
            proposer: self.node.node_id.clone(),
            tx_count: self.pending_tx_count,
        }
    }

    /// Vote on a block proposal
    pub fn vote(&self, block_height: u64, approve: bool) -> Message {
        Message::Vote {
            block_height,
            voter: self.node.node_id.clone(),
            approve,
        }
    }
}

/// Run a multi-node simulation
pub fn run_multi_node_simulation() {
    println!("═══════════ MULTI-NODE SIMULATION ═══════════");

    // Create 4 validator nodes
    let mut validators: Vec<ValidatorNode> = (0..4)
        .map(|i| ValidatorNode::new(&format!("validator-{}", i + 1), 9000 + i as u16))
        .collect();

    // Each node knows about all others
    let ports: Vec<u16> = (9000..9004).collect();
    for v in &mut validators {
        for (j, &port) in ports.iter().enumerate() {
            if port != v.node.port {
                v.node.add_peer(&format!("validator-{}", j + 1), port);
            }
        }
    }

    // Start all listeners
    let _handles: Vec<_> = validators.iter().map(|v| v.node.start_listening()).collect();

    // Give listeners time to start
    thread::sleep(Duration::from_millis(500));

    // Send hello messages between peers
    println!("\n--- Peer Discovery ---");
    for v in &mut validators {
        for &port in &ports {
            if port != v.node.port {
                v.node.send_to(port, &Message::PeerHello {
                    node_id: v.node.node_id.clone(),
                    port: v.node.port,
                });
            }
        }
    }
    thread::sleep(Duration::from_millis(300));

    // Process hello messages
    for v in &mut validators {
        let msgs = v.node.fetch_messages();
        for msg in &msgs {
            if let Message::PeerHello { node_id, port } = msg {
                println!("[{}] Discovered peer {} on port {}", v.node.node_id, node_id, port);
            }
        }
    }

    // Simulate transaction gossip
    println!("\n--- Transaction Gossip ---");
    let tx = Message::Transaction {
        tx_data: "alice→bob:50 LGT".to_string(),
        from: "validator-1".to_string(),
    };
    validators[0].node.broadcast(&tx);
    thread::sleep(Duration::from_millis(300));

    for v in &mut validators {
        let msgs = v.node.fetch_messages();
        for msg in &msgs {
            if let Message::Transaction { tx_data, from } = msg {
                println!("[{}] Received tx from {}: {}", v.node.node_id, from, tx_data);
                v.pending_tx_count += 1;
            }
        }
    }

    // Simulate block proposal and voting
    println!("\n--- Block Proposal & Voting ---");
    let proposer = &validators[0];
    let block = proposer.propose_block();
    proposer.node.broadcast(&block);
    thread::sleep(Duration::from_millis(300));

    // Each validator votes
    for v in &mut validators {
        let msgs = v.node.fetch_messages();
        let mut saw_block = false;
        for msg in &msgs {
            if let Message::BlockProposal { block_height, proposer, tx_count } = msg {
                println!("[{}] Saw block proposal: height={}, proposer={}, txs={}",
                    v.node.node_id, block_height, proposer, tx_count);
                saw_block = true;
            }
        }
        if saw_block {
            let vote = v.vote(1, true);
            v.node.broadcast(&vote);
        }
    }
    thread::sleep(Duration::from_millis(300));

    // Count votes
    println!("\n--- Vote Tally ---");
    for v in &mut validators {
        let msgs = v.node.fetch_messages();
        let yes_votes = msgs.iter()
            .filter(|m| matches!(m, Message::Vote { approve: true, .. }))
            .count();
        println!("[{}] Counted {} YES votes", v.node.node_id, yes_votes);
    }

    // Epoch state root broadcast
    println!("\n--- Epoch Finalization ---");
    let epoch_msg = Message::EpochStateRoot {
        epoch: 1,
        root: "blake3-root-00112233445566778899aabbccddeeff".to_string(),
    };
    validators[0].node.broadcast(&epoch_msg);
    thread::sleep(Duration::from_millis(300));

    for v in &mut validators {
        let msgs = v.node.fetch_messages();
        for msg in &msgs {
            if let Message::EpochStateRoot { epoch, root } = msg {
                println!("[{}] Received epoch {} state root: {}", v.node.node_id, epoch, &root[..32]);
            }
        }
    }

    println!("\n[NET] Multi-node simulation complete.");
    println!("[NET] 4 validators gossiped, proposed blocks, voted, and finalized an epoch.");
}
