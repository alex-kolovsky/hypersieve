use wasmi::{Caller, Engine, Instance, Linker, Module, Store};

#[derive(Debug)]
pub struct WasmModule<T>
where
    T: Clone,
{
    name: &'static str,
    instance: Instance,
    store: Store<T>,
    panic_count: u8,
    standard_store_value: T,
    linker: Linker<T>,
    module: Module,
    spare_wasm_module: Option<SpareWasmModule<T>>,
}

#[derive(Debug)]
pub struct WasmModuleInfo {
    pub name: &'static str,
    pub data: &'static [u8],
    pub spare_module_data: Option<&'static [u8]>,
}

#[derive(Debug)]
pub struct SpareWasmModule<T>
where
    T: Clone,
{
    instance: Instance,
    store: Store<T>,
    standard_store_value: T,
    linker: Linker<T>,
    module: Module,
    is_switched: bool,
}

impl<T> SpareWasmModule<T>
where
    T: Clone,
{
    fn into_wasm_module(
        self,
        wasm_module: Option<SpareWasmModule<T>>,
        name: &'static str,
    ) -> WasmModule<T> {
        WasmModule {
            name,
            instance: self.instance,
            store: self.store,
            panic_count: 0,
            standard_store_value: self.standard_store_value,
            linker: self.linker,
            module: self.module,
            spare_wasm_module: wasm_module,
        }
    }
}

impl<T> WasmModule<T>
where
    T: Clone,
{
    fn new(
        wasm: &[u8],
        store_value: T,
        spare_wasm_module_data: Option<&[u8]>,
        initialize_module_functions: fn(&mut Linker<T>),
        name: &'static str,
    ) -> Self {
        let engine = ENGINE.get().unwrap();
        let mut store = Store::new(engine, store_value.clone());

        // Create a Module struct from the raw .wasm data.
        let module = unsafe { Module::new_unchecked(engine, wasm) };

        let mut linker = <Linker<T>>::new(engine);
        initialize_module_functions(&mut linker);

        let instance = linker
            .instantiate_and_start(&mut store, module.as_ref().unwrap())
            .unwrap();

        let spare_wasm_module = if let Some(data) = spare_wasm_module_data {
            Some(
                WasmModule::new(
                    data,
                    store_value.clone(),
                    None,
                    initialize_module_functions,
                    name,
                )
                .into_spare_wasm_module(false),
            )
        } else {
            None
        };

        Self {
            name,
            instance,
            store,
            panic_count: 0,
            standard_store_value: store_value,
            linker,
            module: module.unwrap(),
            spare_wasm_module,
        }
    }

    fn into_spare_wasm_module(self, is_switched: bool) -> SpareWasmModule<T> {
        SpareWasmModule {
            instance: self.instance,
            store: self.store,
            standard_store_value: self.standard_store_value,
            linker: self.linker,
            module: self.module,
            is_switched,
        }
    }
    fn switch_with_spare_module(mut self) -> Self {
        let spare_module = self.spare_wasm_module.take().unwrap();

        let module_name = self.name;

        let switched_module = self.into_spare_wasm_module(true);
        spare_module.into_wasm_module(Some(switched_module), module_name)
    }

    fn recover(mut self, cause: wasmi::Error) -> Self {
        match self.panic_count {
            // At the first panic of the module, simply reset its Store.
            0 => {
                crate::println!(
                    "\n[WARN][MODULES]: {} module '{}' panicked (1/3): {cause}",
                    self.spare_prefix(),
                    self.name
                );
                self.panic_count += 1;

                self.reset_store();
                self
            }
            // At the second - reset its Instance and Store.
            1 => {
                crate::println!(
                    "\n[WARN][MODULES]: {} module '{}' panicked (2/3): {cause}",
                    self.spare_prefix(),
                    self.name
                );
                self.panic_count += 1;

                self.reset_instance();
                self
            }
            // Replace the current module with the spare one on the third fault.
            2 => {
                if let Some(spare_wasm_module) = &self.spare_wasm_module {
                    if spare_wasm_module.is_switched {
                        // If even the spare module faulted, panic.
                        panic!(
                            "[CRITICAL][MODULES]: spare '{}' reached panic limit (3/3), module is destroyed: {cause}",
                            self.name
                        );
                    }

                    crate::println!(
                        "\n[ERROR][MODULES]: module '{}' panicked (3/3), switching to the spare module: {cause}",
                        self.name
                    );

                    self.switch_with_spare_module()
                } else {
                    // Panic on the third fault if module has no spare module configured.
                    panic!(
                        "[CRITICAL][MODULES]: '{}' reached panic limit (3/3), no spare module configured, module is destroyed: {cause}",
                        self.name
                    );
                }
            }
            _ => unreachable!(),
        }
    }
    #[inline(always)]
    fn reset_store(self: &mut WasmModule<T>) {
        let store = &mut self.store;

        // Reset only the Store structure data, not the structure itself, to avoid resetting the entire instance.
        let old_store: &mut T = store.data_mut();
        *old_store = self.standard_store_value.clone();
    }

    #[inline(always)]
    fn reset_instance(self: &mut WasmModule<T>) {
        self.reset_store();
        let mut store = &mut self.store;

        let old_instance = &mut self.instance;
        let instance = self.linker.instantiate_and_start(&mut store, &self.module);

        *old_instance = instance.unwrap();
    }
    fn spare_prefix(&self) -> &'static str {
        if let Some(ref spare) = self.spare_wasm_module
            && spare.is_switched
        {
            "spare"
        } else {
            ""
        }
    }
}

pub static ENGINE: spin::Once<Engine> = spin::Once::new();
pub static CLAIM_GUEST_MODULE: spin::Mutex<Option<WasmModule<u32>>> = spin::Mutex::new(None);
pub static CLAIM_ASSIGNED_GUEST_MODULE: spin::Mutex<Option<WasmModule<u32>>> =
    spin::Mutex::new(None);

pub fn init_engine() {
    ENGINE.call_once(Engine::default);
}

pub fn init_wasm_modules() {
    let mut claim_assigned_guest = CLAIM_ASSIGNED_GUEST_MODULE.lock();
    *claim_assigned_guest = Some(WasmModule::new(
        crate::CLAIM_ASSIGNED_GUEST_MODULE_WASM.data,
        0u32,
        crate::CLAIM_ASSIGNED_GUEST_MODULE_WASM.spare_module_data,
        init_claim_assigned_guest_module,
        "claim_assigned_guest",
    ));
    drop(claim_assigned_guest);

    let mut claim_guest = CLAIM_GUEST_MODULE.lock();
    *claim_guest = Some(WasmModule::new(
        crate::CLAIM_GUEST_MODULE_WASM.data,
        0u32,
        crate::CLAIM_GUEST_MODULE_WASM.spare_module_data,
        init_claim_guest_module,
        "claim_guest",
    ));
    drop(claim_guest);
}

fn init_claim_guest_module(linker: &mut Linker<u32>) {
    linker
        .func_wrap(
            "env",
            "claim_vcpu_for_hart_if_available",
            |_caller: Caller<'_, u32>, guest_id: u32| -> u32 {
                crate::guest::claim_vcpu_for_hart_if_available(guest_id as usize) as u32
            },
        )
        .unwrap();
}

fn init_claim_assigned_guest_module(linker: &mut Linker<u32>) {
    linker
        .func_wrap(
            "env",
            "claim_assigned_hart_slot_if_available",
            |_caller: Caller<'_, u32>, guest_id: u32| -> u32 {
                crate::guest::claim_assigned_hart_slot_if_available(guest_id as usize) as u32
            },
        )
        .unwrap();
}

pub struct Modules;

impl Modules {
    pub fn claim_guest(guest_id: usize) -> u32 {
        let mut module_lock = CLAIM_GUEST_MODULE.lock();

        // Subsequent loop iterations restore the module if it will panic.
        loop {
            let mut module: WasmModule<u32> = module_lock.take().unwrap();
            let instance = module.instance;
            let mut store = &mut module.store;

            let function = instance
                .get_typed_func::<u32, u32>(&store, module.name)
                .unwrap();

            let result = function.call(&mut store, guest_id as u32);

            // Return the guest ID if the module did not panic; Otherwise, restore the module.
            if let Ok(guest_id) = result {
                *module_lock = Some(module);
                drop(module_lock);
                return guest_id;
            } else if let Err(cause) = result {
                *module_lock = Some(module.recover(cause));
                continue;
            }
        }
    }
    pub fn claim_assigned_guest(guest_id: usize, hart_id: usize) -> u32 {
        let mut module_lock = CLAIM_ASSIGNED_GUEST_MODULE.lock();

        // Subsequent loop iterations restore the module if it will panic.
        loop {
            let mut module: WasmModule<u32> = module_lock.take().unwrap();
            let instance = &mut module.instance;
            let store = &mut module.store;

            let function = instance
                .get_typed_func::<(u32, u32), u32>(&*store, module.name)
                .unwrap();

            let result = function.call(&mut *store, (guest_id as u32, hart_id as u32));

            // Return the local guest ID if the module did not panic; Otherwise, restore the module.
            if let Ok(local_guest_id) = result {
                *module_lock = Some(module);
                drop(module_lock);
                return local_guest_id;
            } else if let Err(cause) = result {
                *module_lock = Some(module.recover(cause));
                continue;
            }
        }
    }
}
