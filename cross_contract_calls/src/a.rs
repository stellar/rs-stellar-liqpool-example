use soroban_sdk::contractimpl;

pub struct ContractA;

#[contractimpl(export_if = "export")]
impl ContractA {
    pub fn add(x: u32, y: u32) -> u32 {
        x.checked_add(y).expect("no overflow")
    }
}
