use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SharedError {
    NotInitialized = 1,
    Unauthorized = 2,
    InvalidInput = 3,
    ResourceNotFound = 4,
    InvalidLength = 5,
}
