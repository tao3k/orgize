// @generated from Parser grammar metadata.
use crate::org_product::{KindCategory, KindSpec, LanguageSpec};

static KINDS: &[KindSpec] = &[
    KindSpec { name: "ContractSource", category: KindCategory::Node },
    KindSpec { name: "ContractList", category: KindCategory::Node },
    KindSpec { name: "ContractAtom", category: KindCategory::Node },
    KindSpec { name: "ContractString", category: KindCategory::Node },
    KindSpec { name: "Whitespace", category: KindCategory::Token },
    KindSpec { name: "Comment", category: KindCategory::Token },
    KindSpec { name: "OpenParen", category: KindCategory::Token },
    KindSpec { name: "CloseParen", category: KindCategory::Token },
    KindSpec { name: "ContractStringToken", category: KindCategory::Token },
    KindSpec { name: "ContractAtomToken", category: KindCategory::Token },
];

pub static LANGUAGE: LanguageSpec = LanguageSpec {
    language: "org-contract",
    version: "v1",
    contract: "org-contract-expression.v1",
    grammar_digest: "sha256:f61908067852f4c14d9c96c1758afce1d967249c258fe66d75e91d495761537b",
    case_insensitive: false,
    root_kind: 0,
    kinds: KINDS,
};
