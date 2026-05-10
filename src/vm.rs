// LUGET VM Module
// Object-centric programmable layer
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAbilities {
    pub copy: bool,
    pub drop: bool,
    pub store: bool,
    pub key: bool,
}

#[derive(Debug, Clone)]
pub struct ObjectType {
    pub type_id: String,
    pub name: String,
    pub abilities: TypeAbilities,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Owner {
    Address(String),
    Object(String),
    Shared,
}

#[derive(Debug, Clone)]
pub struct VmObject {
    pub id: String,
    pub type_id: String,
    pub owner: Owner,
    pub data: Vec<u8>,
    pub stored_objects: HashMap<String, VmObject>,
}

#[derive(Debug, Clone)]
pub struct VmState {
    pub objects: HashMap<String, VmObject>,
    pub types: HashMap<String, ObjectType>,
    pub total_objects_created: u64,
}

impl VmState {
    pub fn new() -> Self {
        VmState {
            objects: HashMap::new(),
            types: HashMap::new(),
            total_objects_created: 0,
        }
    }

    pub fn register_type(&mut self, object_type: ObjectType) {
        println!("[VM] Type registered: {} (copy:{}, drop:{}, store:{}, key:{})",
            object_type.name,
            object_type.abilities.copy,
            object_type.abilities.drop,
            object_type.abilities.store,
            object_type.abilities.key,
        );
        self.types.insert(object_type.type_id.clone(), object_type);
    }

    pub fn create_object(
        &mut self,
        type_id: &str,
        owner: Owner,
        data: Vec<u8>,
    ) -> Result<VmObject, String> {
        let object_type = self.types.get(type_id)
            .ok_or(format!("Type {} not registered", type_id))?;

        if owner != Owner::Shared && !object_type.abilities.key {
            return Err(format!("Type {} does not have Key ability", type_id));
        }

        self.total_objects_created += 1;
        let object_id = format!("obj-{}", self.total_objects_created);

        let object = VmObject {
            id: object_id.clone(),
            type_id: type_id.to_string(),
            owner,
            data,
            stored_objects: HashMap::new(),
        };

        println!("[VM] Object created: {} (type: {})", object_id, object_type.name);
        Ok(object)
    }

    pub fn transfer_object(&mut self, object_id: &str, new_owner: Owner) -> Result<(), String> {
        let object = self.find_object_mut(object_id)?;
        if object.owner == Owner::Shared {
            return Err("Cannot transfer a shared (immutable) object".to_string());
        }
        object.owner = new_owner;
        println!("[VM] Object {} transferred", object_id);
        Ok(())
    }

    pub fn store_object_inside(&mut self, child_id: &str, parent_id: &str) -> Result<(), String> {
        let child_type_id = {
            let child = self.find_object(child_id)?;
            child.type_id.clone()
        };
        let child_type = self.types.get(&child_type_id).ok_or("Child type not found")?;
        if !child_type.abilities.store {
            return Err(format!("Type {} does not have Store ability", child_type_id));
        }
        let parent_type_id = {
            let parent = self.find_object(parent_id)?;
            parent.type_id.clone()
        };
        let parent_type = self.types.get(&parent_type_id).ok_or("Parent type not found")?;
        if !parent_type.abilities.key {
            return Err(format!("Type {} does not have Key ability", parent_type_id));
        }
        let child_obj = self.objects.remove(child_id).ok_or("Child object not found")?;
        let parent = self.find_object_mut(parent_id)?;
        parent.stored_objects.insert(child_id.to_string(), child_obj);
        println!("[VM] Object {} stored inside {}", child_id, parent_id);
        Ok(())
    }

    pub fn copy_object(&mut self, object_id: &str, new_owner: Owner) -> Result<VmObject, String> {
        let type_id = {
            let object = self.find_object(object_id)?;
            object.type_id.clone()
        };
        let object_type = self.types.get(&type_id).ok_or("Type not found")?;
        if !object_type.abilities.copy {
            return Err(format!("Type {} does not have Copy ability - duplication rejected", object_type.name));
        }
        self.total_objects_created += 1;
        let new_id = format!("obj-{}", self.total_objects_created);
        let new_object = VmObject {
            id: new_id.clone(),
            type_id,
            owner: new_owner,
            data: vec![],
            stored_objects: HashMap::new(),
        };
        println!("[VM] Object {} copied to {}", object_id, new_id);
        Ok(new_object)
    }

    pub fn drop_object(&mut self, object_id: &str) -> Result<(), String> {
        let type_id = {
            let object = self.find_object(object_id)?;
            object.type_id.clone()
        };
        let object_type = self.types.get(&type_id).ok_or("Type not found")?;
        if !object_type.abilities.drop {
            return Err(format!("Type {} does not have Drop ability - silent deletion rejected", object_type.name));
        }
        self.objects.remove(object_id);
        println!("[VM] Object {} dropped", object_id);
        Ok(())
    }

    pub fn explicit_destroy(&mut self, object_id: &str, _auth: &str) -> Result<(), String> {
        self.objects.remove(object_id);
        println!("[VM] Object {} explicitly destroyed", object_id);
        Ok(())
    }

    fn find_object(&self, object_id: &str) -> Result<&VmObject, String> {
        if let Some(obj) = self.objects.get(object_id) { return Ok(obj); }
        for obj in self.objects.values() {
            if let Some(found) = Self::search_in_object(obj, object_id) { return Ok(found); }
        }
        Err(format!("Object {} not found", object_id))
    }

    fn find_object_mut(&mut self, object_id: &str) -> Result<&mut VmObject, String> {
        if self.objects.contains_key(object_id) {
            return self.objects.get_mut(object_id).ok_or_else(|| "Object not found".to_string());
        }
        for obj in self.objects.values_mut() {
            if Self::search_in_object(obj, object_id).is_some() {
                return Self::search_in_object_mut(obj, object_id);
            }
        }
        Err(format!("Object {} not found", object_id))
    }

    fn search_in_object<'a>(obj: &'a VmObject, target_id: &str) -> Option<&'a VmObject> {
        if let Some(found) = obj.stored_objects.get(target_id) { return Some(found); }
        for child in obj.stored_objects.values() {
            if let Some(found) = Self::search_in_object(child, target_id) { return Some(found); }
        }
        None
    }

    fn search_in_object_mut<'a>(obj: &'a mut VmObject, target_id: &str) -> Result<&'a mut VmObject, String> {
        obj.stored_objects.get_mut(target_id).ok_or_else(|| format!("Object {} not found", target_id))
    }

    pub fn print_state(&self) {
        println!("=== VM STATE ===");
        println!("Registered Types: {}", self.types.len());
        for t in self.types.values() {
            println!("  Type: {} (copy:{}, drop:{}, store:{}, key:{})",
                t.name, t.abilities.copy, t.abilities.drop, t.abilities.store, t.abilities.key);
        }
        println!("Top-Level Objects: {}", self.objects.len());
        for (id, obj) in &self.objects {
            let owner_str = match &obj.owner {
                Owner::Address(a) => format!("address:{}", &a[..8.min(a.len())]),
                Owner::Object(o) => format!("object:{}", &o[..8.min(o.len())]),
                Owner::Shared => "shared".to_string(),
            };
            println!("  {}: type={}, owner={}, stored={}", id, obj.type_id, owner_str, obj.stored_objects.len());
        }
        println!("==============================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_coin_type(vm: &mut VmState) {
        vm.register_type(ObjectType {
            type_id: "Coin<LGT>".to_string(), name: "Coin<LGT>".to_string(),
            abilities: TypeAbilities { copy: false, drop: false, store: true, key: true },
        });
    }

    fn setup_nft_type(vm: &mut VmState) {
        vm.register_type(ObjectType {
            type_id: "NFT<Art>".to_string(), name: "NFT<Art>".to_string(),
            abilities: TypeAbilities { copy: false, drop: false, store: true, key: true },
        });
    }

    fn setup_receipt_type(vm: &mut VmState) {
        vm.register_type(ObjectType {
            type_id: "Receipt".to_string(), name: "Receipt".to_string(),
            abilities: TypeAbilities { copy: true, drop: true, store: false, key: false },
        });
    }

    #[test] fn test_create_object() {
        let mut vm = VmState::new(); setup_coin_type(&mut vm);
        assert!(vm.create_object("Coin<LGT>", Owner::Address("alice".to_string()), vec![]).is_ok());
    }

    #[test] fn test_transfer_object() {
        let mut vm = VmState::new(); setup_coin_type(&mut vm);
        let obj = vm.create_object("Coin<LGT>", Owner::Address("alice".to_string()), vec![]).unwrap();
        let id = obj.id.clone(); vm.objects.insert(id.clone(), obj);
        vm.transfer_object(&id, Owner::Address("bob".to_string())).unwrap();
        assert_eq!(vm.objects.get(&id).unwrap().owner, Owner::Address("bob".to_string()));
    }

    #[test] fn test_copy_rejected_for_coin() {
        let mut vm = VmState::new(); setup_coin_type(&mut vm);
        let obj = vm.create_object("Coin<LGT>", Owner::Address("alice".to_string()), vec![]).unwrap();
        let id = obj.id.clone(); vm.objects.insert(id.clone(), obj);
        assert!(vm.copy_object(&id, Owner::Address("bob".to_string())).is_err());
    }

    #[test] fn test_copy_allowed_for_receipt() {
        let mut vm = VmState::new(); setup_receipt_type(&mut vm);
        // Receipt lacks Key, so must be created as Shared (ephemeral)
        let obj = vm.create_object("Receipt", Owner::Shared, vec![]).unwrap();
        let id = obj.id.clone(); vm.objects.insert(id.clone(), obj);
        // Copying to an address is allowed since Copy is true
        assert!(vm.copy_object(&id, Owner::Address("bob".to_string())).is_ok());
    }

    #[test] fn test_drop_rejected_for_coin() {
        let mut vm = VmState::new(); setup_coin_type(&mut vm);
        let obj = vm.create_object("Coin<LGT>", Owner::Address("alice".to_string()), vec![]).unwrap();
        let id = obj.id.clone(); vm.objects.insert(id.clone(), obj);
        assert!(vm.drop_object(&id).is_err());
    }

    #[test] fn test_explicit_destroy_works() {
        let mut vm = VmState::new(); setup_coin_type(&mut vm);
        let obj = vm.create_object("Coin<LGT>", Owner::Address("alice".to_string()), vec![]).unwrap();
        let id = obj.id.clone(); vm.objects.insert(id.clone(), obj);
        assert!(vm.explicit_destroy(&id, "bridge").is_ok());
    }

    #[test] fn test_store_object_inside() {
        let mut vm = VmState::new(); setup_coin_type(&mut vm); setup_nft_type(&mut vm);
        let vault = vm.create_object("NFT<Art>", Owner::Address("alice".to_string()), vec![]).unwrap();
        let vault_id = vault.id.clone(); vm.objects.insert(vault_id.clone(), vault);
        let coin = vm.create_object("Coin<LGT>", Owner::Address("alice".to_string()), vec![]).unwrap();
        let coin_id = coin.id.clone(); vm.objects.insert(coin_id.clone(), coin);
        vm.store_object_inside(&coin_id, &vault_id).unwrap();
        assert!(vm.objects.get(&vault_id).unwrap().stored_objects.contains_key(&coin_id));
    }
}
