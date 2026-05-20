//! Org Crypt side-table projection.

use super::{
    CryptKey, CryptState, CryptTag, CryptWarning, CryptWarningKind, Document, ElementData,
    ParsedAnnotation, Property, Section, SectionIndexSource,
};

const DEFAULT_CRYPT_TAG: &str = "crypt";
const PGP_BEGIN: &str = "-----BEGIN PGP MESSAGE-----";
const PGP_END: &str = "-----END PGP MESSAGE-----";

impl Document<ParsedAnnotation> {
    /// Projects Org Crypt metadata without decrypting or mutating the source.
    ///
    /// Org Crypt encrypts subtree body text, not the headline or properties.
    /// This side table records the body as opaque when a section has the
    /// default `crypt` tag directly or through inherited tags.
    pub fn crypt_states(&self) -> Vec<CryptState> {
        let mut states = Vec::new();
        for section in &self.sections {
            collect_crypt_states(section, Vec::new(), &mut states);
        }
        states
    }
}

fn collect_crypt_states(
    section: &Section<ParsedAnnotation>,
    mut outline_path: Vec<String>,
    states: &mut Vec<CryptState>,
) {
    let title = section.raw_title.trim_end().to_string();
    outline_path.push(title.clone());
    if let Some(state) = crypt_state(section, outline_path.clone(), title) {
        states.push(state);
    }
    for child in &section.subsections {
        collect_crypt_states(child, outline_path.clone(), states);
    }
}

fn crypt_state(
    section: &Section<ParsedAnnotation>,
    outline_path: Vec<String>,
    title: String,
) -> Option<CryptState> {
    let has_direct_tag = has_tag(&section.tags, DEFAULT_CRYPT_TAG);
    let has_effective_tag = has_tag(&section.effective_tags, DEFAULT_CRYPT_TAG);
    let has_inherited_tag = has_effective_tag && !has_direct_tag;
    let crypt_key = crypt_key(section);
    let body_is_opaque = has_direct_tag || has_inherited_tag;
    if !body_is_opaque && crypt_key.is_none() {
        return None;
    }

    let encrypted_payload = has_encrypted_payload(section);
    let mut warnings = Vec::new();
    if has_inherited_tag {
        warnings.push(CryptWarning {
            kind: CryptWarningKind::InheritedCryptTag,
            message: "crypt tag is inherited; Org Crypt setups usually exclude it from inheritance"
                .to_string(),
        });
    }
    if body_is_opaque && !encrypted_payload {
        warnings.push(CryptWarning {
            kind: CryptWarningKind::PlaintextCryptBody,
            message:
                "crypt-tagged body is plaintext in source and should still be treated as sensitive"
                    .to_string(),
        });
    }
    if !body_is_opaque && crypt_key.is_some() {
        warnings.push(CryptWarning {
            kind: CryptWarningKind::CryptKeyWithoutCryptTag,
            message: "CRYPTKEY property has no effect without the crypt tag".to_string(),
        });
    }

    Some(CryptState {
        source: SectionIndexSource::from_annotation(&section.ann),
        outline_path,
        level: section.level,
        title,
        tag: CryptTag::default_org_crypt(),
        has_direct_tag,
        has_inherited_tag,
        crypt_key,
        encrypted_payload,
        body_is_opaque,
        warnings,
    })
}

fn crypt_key(section: &Section<ParsedAnnotation>) -> Option<CryptKey> {
    section
        .properties
        .iter()
        .find_map(|property| crypt_key_from_property(property, false))
        .or_else(|| {
            section
                .effective_properties
                .iter()
                .find_map(|property| crypt_key_from_property(property, true))
        })
}

fn crypt_key_from_property(
    property: &Property<ParsedAnnotation>,
    inherited: bool,
) -> Option<CryptKey> {
    if !property.key.eq_ignore_ascii_case("CRYPTKEY") {
        return None;
    }
    let value = property.value.trim();
    (!value.is_empty()).then(|| CryptKey {
        source: SectionIndexSource::from_annotation(&property.ann),
        value: value.to_string(),
        inherited,
    })
}

fn has_tag(tags: &[String], needle: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(needle))
}

fn has_encrypted_payload(section: &Section<ParsedAnnotation>) -> bool {
    section
        .children
        .iter()
        .filter(|element| !matches!(element.data, ElementData::PropertyDrawer(_)))
        .any(|element| element.ann.raw.contains(PGP_BEGIN) && element.ann.raw.contains(PGP_END))
}
