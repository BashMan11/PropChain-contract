// Error types for the governance contract (Issue #101 - extracted from lib.rs)

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    Unauthorized,
    ProposalNotFound,
    AlreadyVoted,
    ProposalClosed,
    ThresholdNotMet,
    TimelockActive,
    InvalidThreshold,
    SignerExists,
    SignerNotFound,
    MinSigners,
    MaxProposals,
    NotASigner,
    ProposalExpired,
    /// Signer roster changes are blocked while proposals are actively voting.
    SignerChangesLocked,
    /// The signer roster is at its configured maximum (Issue #1124).
    MaxSigners,
    /// The caller cannot vote because they have an active delegation (Issue #1123).
    StillDelegated,
    /// Self-delegation is not allowed (Issue #1123).
    InvalidDelegationTarget,
    /// The admin rotation target is the current admin or the zero address (Issue #1126).
    InvalidRotationTarget,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "Caller is not authorized"),
            Error::ProposalNotFound => write!(f, "Proposal not found"),
            Error::AlreadyVoted => write!(f, "Already voted on this proposal"),
            Error::ProposalClosed => write!(f, "Proposal is closed"),
            Error::ThresholdNotMet => write!(f, "Approval threshold not met"),
            Error::TimelockActive => write!(f, "Timelock period has not elapsed"),
            Error::InvalidThreshold => write!(f, "Invalid threshold value"),
            Error::SignerExists => write!(f, "Signer already exists"),
            Error::SignerNotFound => write!(f, "Signer not found"),
            Error::MinSigners => write!(f, "Cannot go below minimum signers"),
            Error::MaxProposals => write!(f, "Maximum active proposals reached"),
            Error::NotASigner => write!(f, "Caller is not a signer"),
            Error::ProposalExpired => write!(f, "Proposal has expired"),
            Error::SignerChangesLocked => {
                write!(f, "Signer roster is locked while proposals are actively voting")
            }
            Error::MaxSigners => write!(f, "Signer roster is at its configured maximum"),
            Error::StillDelegated => write!(f, "Delegators cannot vote directly"),
            Error::InvalidDelegationTarget => write!(f, "Self-delegation is not allowed"),
            Error::InvalidRotationTarget => {
                write!(f, "Admin rotation target must differ from the admin and be non-zero")
            }
        }
    }
}

impl ContractError for Error {
    fn error_code(&self) -> u32 {
        match self {
            Error::Unauthorized => governance_codes::GOVERNANCE_UNAUTHORIZED,
            Error::ProposalNotFound => governance_codes::GOVERNANCE_PROPOSAL_NOT_FOUND,
            Error::AlreadyVoted => governance_codes::GOVERNANCE_ALREADY_VOTED,
            Error::ProposalClosed => governance_codes::GOVERNANCE_PROPOSAL_CLOSED,
            Error::ThresholdNotMet => governance_codes::GOVERNANCE_THRESHOLD_NOT_MET,
            Error::TimelockActive => governance_codes::GOVERNANCE_TIMELOCK_ACTIVE,
            Error::InvalidThreshold => governance_codes::GOVERNANCE_INVALID_THRESHOLD,
            Error::SignerExists => governance_codes::GOVERNANCE_SIGNER_EXISTS,
            Error::SignerNotFound => governance_codes::GOVERNANCE_SIGNER_NOT_FOUND,
            Error::MinSigners => governance_codes::GOVERNANCE_MIN_SIGNERS,
            Error::MaxProposals => governance_codes::GOVERNANCE_MAX_PROPOSALS,
            Error::NotASigner => governance_codes::GOVERNANCE_NOT_A_SIGNER,
            Error::ProposalExpired => governance_codes::GOVERNANCE_PROPOSAL_EXPIRED,
            Error::SignerChangesLocked => governance_codes::GOVERNANCE_SIGNER_CHANGES_LOCKED,
            Error::MaxSigners => governance_codes::GOVERNANCE_MAX_SIGNERS,
            Error::StillDelegated => governance_codes::GOVERNANCE_STILL_DELEGATED,
            Error::InvalidDelegationTarget => {
                governance_codes::GOVERNANCE_INVALID_DELEGATION_TARGET
            }
            Error::InvalidRotationTarget => {
                governance_codes::GOVERNANCE_INVALID_ROTATION_TARGET
            }
        }
    }

    fn error_description(&self) -> &'static str {
        match self {
            Error::Unauthorized => "Caller does not have governance permissions",
            Error::ProposalNotFound => "The governance proposal does not exist",
            Error::AlreadyVoted => "Caller has already voted on this proposal",
            Error::ProposalClosed => "The proposal is no longer accepting votes",
            Error::ThresholdNotMet => "Not enough votes to meet the approval threshold",
            Error::TimelockActive => "The timelock period has not elapsed yet",
            Error::InvalidThreshold => "Threshold must be between 1 and signer count",
            Error::SignerExists => "This account is already a signer",
            Error::SignerNotFound => "This account is not a registered signer",
            Error::MinSigners => "Cannot remove signer: minimum signer count reached",
            Error::MaxProposals => "Cannot create proposal: active limit reached",
            Error::NotASigner => "Only signers can perform this action",
            Error::ProposalExpired => "The proposal voting period has expired",
            Error::SignerChangesLocked => {
                "Signer roster is locked while proposals are actively voting"
            }
            Error::MaxSigners => "Signer roster is at its configured maximum",
            Error::StillDelegated => "Delegators cannot vote directly",
            Error::InvalidDelegationTarget => "Self-delegation is not allowed",
            Error::InvalidRotationTarget => {
                "Admin rotation target must differ from the admin and be non-zero"
            }
        }
    }

    fn error_category(&self) -> ErrorCategory {
        ErrorCategory::Governance
    }
}
