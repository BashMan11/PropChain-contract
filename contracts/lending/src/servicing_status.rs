//! Closes #808: replace the `String` `servicing_status` field (see
//! `LoanApplication.servicing_status` at `lending/src/lib.rs:151`) with a
//! single-byte discriminant. Wired into the contract storage type (#1090):
//! `LoanApplication` and `PackedLoanApplication` both store the enum exactly
//! once; the duplicate `String`/`Vec<u8>` representations are gone.

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    scale::Encode,
    scale::Decode,
    ink::storage::traits::StorageLayout,
)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[repr(u8)]
pub enum ServicingStatus {
    Pending = 0,
    Boarded = 1,
    InDefault = 2,
    PaidOff = 3,
}

impl ServicingStatus {
    /// Migrates the legacy free-text status strings to the new enum,
    /// defaulting to `Pending` for anything unrecognized.
    pub fn from_legacy_str(value: &str) -> Self {
        match value {
            "Boarded" => ServicingStatus::Boarded,
            "InDefault" => ServicingStatus::InDefault,
            "PaidOff" => ServicingStatus::PaidOff,
            _ => ServicingStatus::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use scale::Encode;

    use super::*;

    #[test]
    fn migrates_known_legacy_strings() {
        assert_eq!(
            ServicingStatus::from_legacy_str("Boarded"),
            ServicingStatus::Boarded
        );
        assert_eq!(
            ServicingStatus::from_legacy_str("InDefault"),
            ServicingStatus::InDefault
        );
    }

    #[test]
    fn defaults_unknown_strings_to_pending() {
        assert_eq!(
            ServicingStatus::from_legacy_str("Whatever"),
            ServicingStatus::Pending
        );
    }

    #[test]
    fn scale_round_trip_preserves_variant() {
        use scale::Decode;
        let variants = [
            ServicingStatus::Pending,
            ServicingStatus::Boarded,
            ServicingStatus::InDefault,
            ServicingStatus::PaidOff,
        ];
        for expected in variants {
            let encoded = expected.encode();
            assert_eq!(encoded.len(), 1, "must be a single-byte discriminant");
            let decoded = ServicingStatus::decode(&mut &encoded[..]).unwrap();
            assert_eq!(decoded, expected);
        }
    }
}
