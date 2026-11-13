use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{PublishingOptionKind, PublishingSettings},
};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/publishing-settings.org");

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

    let edge = Org::parse(
        "#+EXPORT_FILE_NAME: first\n#+EXPORT_FILE_NAME: 　路径.html \n\
         #+BIND: 　name (lambda () (error x))　\n#+BIND: lone\n#+BIND: 　\n\
         #+OPTIONS: H:3　num:t :empty key: key:a:b bad H:4 h:2\n",
    )
    .document()
    .publishing_settings();
    assert_eq!(edge.export_file_name.unwrap().value, "路径.html");
    assert_eq!(edge.binds.len(), 2);
    assert_eq!(edge.binds[0].name, "name");
    assert_eq!(edge.binds[0].value, "(lambda () (error x))");
    assert_eq!(edge.binds[1].value, "");
    assert_eq!(
        edge.options
            .iter()
            .map(|v| (v.key.as_str(), v.value.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("H", "3"),
            ("num", "t"),
            ("", "empty"),
            ("key", ""),
            ("key", "a:b"),
            ("H", "4"),
            ("h", "2")
        ]
    );
    assert_eq!(edge.options[0].kind, PublishingOptionKind::HeadlineLevels);
    assert_eq!(edge.options[6].kind, PublishingOptionKind::Other);
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

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[(
    "semantic_ast::semantic_ast_projects_publishing::semantic_ast_projects_publishing_settings_without_executing_export",
    semantic_ast_projects_publishing_settings_without_executing_export,
)];
