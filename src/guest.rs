#[derive(Debug)]
pub struct Guest {
    pub entry_gpa: usize,
    pub vcpu: crate::vcpu::Vcpu,
    pub harts: spin::Mutex<usize>,
    pub harts_cap: usize,
    pub data: &'static [u8],
}
