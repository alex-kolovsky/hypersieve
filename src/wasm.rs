use wasmi::Engine;

pub struct WasmModule<T> {
    instance: wasmi::Instance,
    store: spin::Mutex<wasmi::Store<T>>,
}

impl<T> WasmModule<T> {
    fn new(
        wasm: &[u8],
        store_value: T,
        initialize_module_functions: fn(&mut wasmi::Linker<T>),
    ) -> Self {
        let engine = ENGINE.get().unwrap();
        let mut store = wasmi::Store::new(engine, store_value);

        // Create a Module struct from the raw .wasm data.
        let module = unsafe { wasmi::Module::new_unchecked(engine, wasm) };

        let mut linker = <wasmi::Linker<T>>::new(engine);
        initialize_module_functions(&mut linker);

        let instance = linker
            .instantiate_and_start(&mut store, &module.unwrap())
            .unwrap();
        Self {
            instance,
            store: spin::Mutex::new(store),
        }
    }
}

pub static ENGINE: spin::Once<Engine> = spin::Once::new();
pub static VCPU_SWITCH_MODULE: spin::Once<WasmModule<u32>> = spin::Once::new();

pub fn init_engine() {
    ENGINE.call_once(Engine::default);
}

pub fn init_wasm_modules() {
    VCPU_SWITCH_MODULE
        .call_once(|| WasmModule::new(crate::CLAIM_GUEST_MODULE, 0u32, init_vcpu_functions));
}

fn init_vcpu_functions(linker: &mut wasmi::Linker<u32>) {
    linker
        .func_wrap(
            "env",
            "claim_vcpu_for_hart_if_available",
            |_caller: wasmi::Caller<'_, u32>, param: u32| -> u32 {
                crate::guest::claim_vcpu_for_hart_if_available(param as usize) as u32
            },
        )
        .unwrap();
}

pub struct Modules;

impl Modules {
    pub fn claim_guest(guest_id: usize) -> u32 {
        let function_name = "claim_guest";

        let instance = VCPU_SWITCH_MODULE.get().unwrap().instance;
        let mut store = VCPU_SWITCH_MODULE.get().unwrap().store.lock();

        let function = instance
            .get_typed_func::<u32, u32>(&*store, function_name)
            .unwrap();

        let result = function.call(&mut *store, guest_id as u32).unwrap();

        drop(store);

        result
    }
}
