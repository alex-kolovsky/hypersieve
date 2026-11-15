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

pub fn init_engine() {
    ENGINE.call_once(Engine::default);
}

pub fn init_wasm_modules() {}

pub struct Modules;

impl Modules {}
