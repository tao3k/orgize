use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{PublishingOptionKind, PublishingSettings},
};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/publishing-settings.org");

#[test]
fn semantic_ast_projects_publishing_settings_without_executing_export() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let settings = doc.publishing_settings();
    assert_eq!(
        settings
            .export_file_name
            .as_ref()
            .map(|keyword| keyword.value.as_str()),
        Some("public/demo/index.html")
    );
    assert_eq!(settings.setup_files.len(), 1);
    assert_eq!(
        settings
            .binds
            .first()
            .map(|bind| (bind.name.as_str(), bind.value.as_str())),
        Some(("org-export-use-babel", "nil"))
    );
    assert!(settings.options.iter().any(|option| {
        option.key == "broken-links"
            && option.value == "mark"
            && option.kind == PublishingOptionKind::BrokenLinks
    }));
    assert!(
        settings
            .backend_keywords
            .iter()
            .any(|keyword| keyword.key == "HTML_HEAD")
    );
    assert!(settings.attributes.iter().any(|attribute| {
        attribute.backend == "html"
            && attribute
                .attributes
                .iter()
                .any(|item| item.key == "class" && item.value.as_deref() == Some("article-cover"))
    }));
    assert_eq!(settings.includes.len(), 1);

    insta::assert_debug_snapshot!(
        "semantic_ast__semantic_publishing_settings",
        settings_without_annotations(settings)
    );
}

fn settings_without_annotations(
    settings: PublishingSettings<orgize::ast::ParsedAnnotation>,
) -> PublishingSettings<()> {
    PublishingSettings {
        export_file_name: settings
            .export_file_name
            .map(|keyword| orgize::ast::PublishingKeyword {
                ann: (),
                key: keyword.key,
                value: keyword.value,
            }),
        setup_files: settings
            .setup_files
            .into_iter()
            .map(|keyword| orgize::ast::PublishingKeyword {
                ann: (),
                key: keyword.key,
                value: keyword.value,
            })
            .collect(),
        binds: settings
            .binds
            .into_iter()
            .map(|bind| orgize::ast::PublishingBind {
                ann: (),
                name: bind.name,
                value: bind.value,
                raw: bind.raw,
            })
            .collect(),
        options: settings
            .options
            .into_iter()
            .map(|option| orgize::ast::PublishingOption {
                ann: (),
                key: option.key,
                value: option.value,
                raw: option.raw,
                kind: option.kind,
            })
            .collect(),
        attributes: settings
            .attributes
            .into_iter()
            .map(|attribute| orgize::ast::PublishingAttribute {
                ann: (),
                backend: attribute.backend,
                optional: attribute.optional,
                attributes: attribute.attributes,
                raw: attribute.raw,
            })
            .collect(),
        backend_keywords: settings
            .backend_keywords
            .into_iter()
            .map(|keyword| orgize::ast::PublishingKeyword {
                ann: (),
                key: keyword.key,
                value: keyword.value,
            })
            .collect(),
        includes: settings
            .includes
            .into_iter()
            .map(|include| orgize::ast::IncludeDirective {
                ann: (),
                path: include.path,
                raw_path: include.raw_path,
                arguments: include.arguments,
                options: include.options,
                raw_value: include.raw_value,
            })
            .collect(),
    }
}
