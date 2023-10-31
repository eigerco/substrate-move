//! SS58 address format converter for Substrate and Move accounts.
//! Based on the Pontem solution.

use anyhow::{anyhow, ensure, Result};
use blake2::{Blake2b512, Digest};
use move_core_types::account_address::AccountAddress;

// Substrate address prefix
// Read more: https://docs.substrate.io/reference/address-formats/
const SS58_PREFIX: &[u8] = b"SS58PRE";

// Public key length in bytes
const PUB_KEY_LENGTH: usize = 32;

// Checksum length in bytes
const CHECK_SUM_LEN: usize = 2;

// Blake2b512 hash length in bytes
const HASH_LEN: usize = 64;

/// Convert ss58 address string to Move address structure
/// In case if such conversion is not possible, return error.
/// ```
/// use move_vm_support::ss58_address::ss58_to_move_address;
/// let substrate_address = "gkNW9pAcCHxZrnoVkhLkEQtsLsW5NWTC75cdAdxAMs9LNYCYg";
/// let move_address = ss58_to_move_address(substrate_address).unwrap();
/// assert_eq!(
///    "0x8EAF04151687736326C9FEA17E25FC5287613693C912909CB226AA4794F26A48",
///   format!("{:#X}", move_address)
/// );
/// ```
pub fn ss58_to_move_address(ss58: &str) -> Result<AccountAddress> {
    let bs58 = bs58::decode(ss58).into_vec()?;

    ensure!(
        bs58.len() > PUB_KEY_LENGTH + CHECK_SUM_LEN,
        format!(
            "Address length must be equal or greater than {} bytes",
            PUB_KEY_LENGTH + CHECK_SUM_LEN
        )
    );

    let check_sum = &bs58[bs58.len() - CHECK_SUM_LEN..];
    let address = &bs58[bs58.len() - PUB_KEY_LENGTH - CHECK_SUM_LEN..bs58.len() - CHECK_SUM_LEN];

    if check_sum != &ss58_hash(&bs58[0..bs58.len() - CHECK_SUM_LEN])[0..CHECK_SUM_LEN] {
        return Err(anyhow!("Wrong address checksum"));
    }

    let mut addr = [0; PUB_KEY_LENGTH];
    addr.copy_from_slice(address);
    Ok(AccountAddress::new(addr))
}

/// Convert SS58 address to Move address string.
pub fn ss58_to_move_address_string(ss58: &str) -> Result<String> {
    Ok(format!("{:#X}", ss58_to_move_address(ss58)?))
}

// Helper function which calculates the BLAKE2b512 hash of the given data.
fn ss58_hash(data: &[u8]) -> [u8; HASH_LEN] {
    let mut hasher = Blake2b512::new();
    hasher.update(SS58_PREFIX);
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ss58_to_move_correct() {
        let substrate_address = "gkNW9pAcCHxZrnoVkhLkEQtsLsW5NWTC75cdAdxAMs9LNYCYg";
        let move_address = ss58_to_move_address_string(substrate_address).unwrap();

        assert_eq!(
            (move_address.len() - 2) / 2,   // 2 hex chars per byte
            PUB_KEY_LENGTH
        );

        assert_eq!(
            "0x8EAF04151687736326C9FEA17E25FC5287613693C912909CB226AA4794F26A48",
            move_address
        );

        let substrate_address = "G7UkJAutjbQyZGRiP8z5bBSBPBJ66JbTKAkFDq3cANwENyX";
        let move_address = ss58_to_move_address_string(substrate_address).unwrap();

        assert_eq!(
            (move_address.len() - 2) / 2,   // 2 hex chars per byte
            PUB_KEY_LENGTH
        );

        assert_eq!(
            "0x9C786090E2598AE884FF9D1F01D6A1A9BAF13A9E61F73633A8928F4D80BF7DFE",
            move_address
        );
    }

    #[test]
    fn test_ss58_to_move_fail() {
        let substrate_address = "G7UkJAutjbQyZGRiP8z5bBSBPBJ66JbTKAkFDq3c"; // too short
        assert!(ss58_to_move_address_string(substrate_address).is_err());
    }

    #[test]
    fn test_ss58hash() {
        let msg = b"hello, world!";
        let hash = ss58_hash(msg).to_vec();

        assert_eq!(hex_literal::hex!("656facfcf4f90cce9ec9b65c9185ea75346507c67e25133f5809b442487468a674973f9167193e86bee0c706f6766f7edf638ed3e21ad12c2908ea62924af4d7").to_vec(), hash);
    }
}
