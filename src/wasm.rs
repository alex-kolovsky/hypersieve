use wasmi::{Caller, Engine, Instance, Linker, Module, Store};

#[derive(Debug)]
pub struct WasmModule<T>
where
    T: Clone,
{
    instance: Instance,
    store: Store<T>,
    panic_count: u8,
    standard_store_value: T,
    linker: Linker<T>,
    module: Module,
}

impl<T> WasmModule<T> where T: Clone {}

impl<T> WasmModule<T>
where
    T: Clone,
{
    fn new(wasm: &[u8], store_value: T, initialize_module_functions: fn(&mut Linker<T>)) -> Self {
        let engine = ENGINE.get().unwrap();
        let mut store = Store::new(engine, store_value.clone());

        // Create a Module struct from the raw .wasm data.
        let module = unsafe { Module::new_unchecked(engine, wasm) };

        let mut linker = <Linker<T>>::new(engine);
        initialize_module_functions(&mut linker);

        let instance = linker
            .instantiate_and_start(&mut store, module.as_ref().unwrap())
            .unwrap();
        Self {
            instance,
            store,
            panic_count: 0,
            standard_store_value: store_value,
            linker,
            module: module.unwrap(),
        }
    }

    fn recover(&mut self) {
        match self.panic_count {
            // At the first panic of the module, simply reset its Store.
            0 => {
                self.panic_count += 1;

                self.reset_store();
            }
            // At the second - reset its Instance and Store.
            1 => {
                self.panic_count += 1;

                self.reset_instance();
            }
            // TODO: Replace the current module with a spare one.
            2 => {
                panic!();
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
        crate::CLAIM_ASSIGNED_GUEST_MODULE_WASM,
        0u32,
        init_claim_assigned_guest_module,
    ));
    drop(claim_assigned_guest);

    let mut claim_guest = CLAIM_GUEST_MODULE.lock();
    *claim_guest = Some(WasmModule::new(
        crate::CLAIM_GUEST_MODULE_WASM,
        0u32,
        init_claim_guest_module,
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
        let module_name = "claim_guest";

        let mut module_opt = CLAIM_GUEST_MODULE.lock();
        let module = module_opt.as_mut().unwrap();
        let instance = module.instance;
        let mut store = &mut module.store;

        let function = instance
            .get_typed_func::<u32, u32>(&store, module_name)
            .unwrap();

        let result = function.call(&mut store, guest_id as u32);

        // Return the guest ID if the module did not panic; Otherwise, restore the module.
        loop {
            if let Ok(guest_id) = result {
                drop(module_opt);
                return guest_id;
            } else {
                module.recover();
                continue;
            }
        }
    }
    pub fn claim_assigned_guest(guest_id: usize, hart_id: usize) -> u32 {
        let module_name = "claim_assigned_guest";

        let mut module_opt = CLAIM_ASSIGNED_GUEST_MODULE.lock();
        let module = module_opt.as_mut().unwrap();
        let instance = &mut module.instance;
        let store = &mut module.store;

        let function = instance
            .get_typed_func::<(u32, u32), u32>(&*store, module_name)
            .unwrap();

        let result = function.call(&mut *store, (guest_id as u32, hart_id as u32));

        // Return the local guest ID if the module did not panic; Otherwise, restore the module.
        loop {
            if let Ok(local_guest_id) = result {
                drop(module_opt);
                return local_guest_id;
            } else {
                module.recover();
                continue;
            }
        }
    }
}
