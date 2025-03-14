use log::error;
use rand::Rng;

use super::module::ModuleMetadata;
use super::{module::Module, system_controller::SystemController};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

pub trait DynModule: Any + Send + Sync {
    fn metadata(&self) -> ModuleMetadata;
    fn as_any(&self) -> &dyn Any;
}

impl<M: Module + 'static + Send + Sync> DynModule for M {
    fn metadata(&self) -> ModuleMetadata {
        self.metadata()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct ModuleManager {
    modules: HashMap<u16, Arc<dyn DynModule>>,
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Generate a **random unique module ID**
    pub fn generate_unique_id(&self) -> u16 {
        let mut rng = rand::rng();

        loop {
            let id = rng.random_range(1..=9999); // Generate a random ID between 1-9999
            if !self.modules.contains_key(&id) {
                return id; // Ensure it's unique
            }
        }
    }

    pub fn register_module<M: Module + 'static + Send + Sync>(
        &mut self,
        mut module: M,
        system_controller: Arc<SystemController>,
    ) {
        let id = module.metadata().id;

        if let Err(err) = module.initialize(system_controller.clone()) {
            error!("Unable to initalize module {}", err);
        }

        self.modules.insert(id, Arc::new(module));
    }

    pub fn get_module<M: Module + Send + Sync + 'static>(&self, id: u16) -> Option<&M> {
        self.modules
            .get(&id)
            .and_then(|module| module.as_any().downcast_ref::<M>())
    }

    pub fn list_modules(&self) -> Vec<ModuleMetadata> {
        self.modules.values().map(|m| m.metadata()).collect()
    }

    pub fn remove_module(&mut self, id: u16) -> bool {
        self.modules.remove(&id).is_some()
    }
}
