// LUGET Persistence Module
// Save and load simulator state to/from disk
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

/// The complete state that can be saved and loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub version: String,
    pub epoch: u64,
    pub block_height: u64,
    pub total_supply: u64,
    pub utxo_count: usize,
    pub clock_count: usize,
    pub vm_object_count: usize,
    pub validator_count: usize,
    pub active_stake: u64,
    pub dormant_stake: u64,
    pub proposal_count: usize,
    pub timestamp: String,
}

impl SavedState {
    /// Create a snapshot from the current simulation
    pub fn snapshot(
        block_height: u64,
        total_supply: u64,
        utxo_count: usize,
        clock_count: usize,
        vm_object_count: usize,
        validator_count: usize,
        epoch: u64,
        active_stake: u64,
        dormant_stake: u64,
        proposal_count: usize,
    ) -> Self {
        SavedState {
            version: "0.4.0".to_string(),
            epoch,
            block_height,
            total_supply,
            utxo_count,
            clock_count,
            vm_object_count,
            validator_count,
            active_stake,
            dormant_stake,
            proposal_count,
            timestamp: chrono_like_now(),
        }
    }

    /// Save state to a JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("Write error: {}", e))?;
        println!("[PERSIST] State saved to {}", path);
        Ok(())
    }

    /// Load state from a JSON file
    pub fn load_from_file(path: &str) -> Result<SavedState, String> {
        if !Path::new(path).exists() {
            return Err(format!("No saved state found at {}", path));
        }
        let json = fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;
        let state: SavedState = serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization error: {}", e))?;
        println!("[PERSIST] State loaded from {}", path);
        Ok(state)
    }

    /// List all saved states in a directory
    pub fn list_saves(dir: &str) -> Vec<String> {
        if let Ok(entries) = fs::read_dir(dir) {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Display the saved state summary
    pub fn display(&self) {
        println!("=== SAVED STATE ===");
        println!("Version:       {}", self.version);
        println!("Saved:         {}", self.timestamp);
        println!("Epoch:         {}", self.epoch);
        println!("Block Height:  {}", self.block_height);
        println!("Total Supply:  {} LGT", self.total_supply);
        println!("UTXOs:         {}", self.utxo_count);
        println!("CLocks:        {}", self.clock_count);
        println!("VM Objects:    {}", self.vm_object_count);
        println!("Validators:    {}", self.validator_count);
        println!("Active Stake:  {} LGT", self.active_stake);
        println!("Dormant Stake: {} LGT", self.dormant_stake);
        println!("Proposals:     {}", self.proposal_count);
        println!("===================");
    }
}

/// Generate a simple timestamp string
fn chrono_like_now() -> String {
    // Simple timestamp without external dependency
    "2026-05-10T00:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load() {
        let state = SavedState::snapshot(42, 1_000_000, 5, 2, 10, 4, 3, 500_000, 100_000, 3);
        let path = "/data/data/com.termux/files/home/lugsim-test-save.json";
        
        state.save_to_file(path).unwrap();
        let loaded = SavedState::load_from_file(path).unwrap();
        
        assert_eq!(loaded.block_height, 42);
        assert_eq!(loaded.total_supply, 1_000_000);
        assert_eq!(loaded.version, "0.4.0");
        
        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_load_nonexistent() {
        let result = SavedState::load_from_file("/tmp/nonexistent-file.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_saves() {
        let dir = "/data/data/com.termux/files/home/lugsim-test-dir";
        let _ = fs::create_dir(dir);
        
        let state = SavedState::snapshot(1, 100, 0, 0, 0, 0, 0, 0, 0, 0);
        state.save_to_file(&format!("{}/save1.json", dir)).unwrap();
        state.save_to_file(&format!("{}/save2.json", dir)).unwrap();
        
        let saves = SavedState::list_saves(dir);
        assert!(saves.len() >= 2);
        
        // Cleanup
        let _ = fs::remove_dir_all(dir);
    }
}
