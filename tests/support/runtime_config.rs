//! Explicit qualification arguments; no ambient benchmark mode variables.
use std::path::PathBuf;

pub(crate) struct Configuration {
    pub(crate) mode: String,
    pub(crate) output: PathBuf,
    pub(crate) documents: Vec<usize>,
    pub(crate) callers: Vec<usize>,
    pub(crate) domains: Vec<usize>,
    pub(crate) repeats: usize,
}

fn counts(value: &str, max: usize) -> Result<Vec<usize>, String> {
    let values = value
        .split(',')
        .map(|part| {
            part.parse::<usize>()
                .ok()
                .filter(|&n| n > 0 && n <= max)
                .ok_or_else(|| format!("expected positive counts <= {max}: {value}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let unique: std::collections::BTreeSet<_> = values.iter().collect();
    if unique.len() != values.len() {
        return Err("duplicate matrix counts".into());
    }
    Ok(values)
}

impl Configuration {
    pub(crate) fn parse(args: &[String]) -> Result<Self, String> {
        if args.len() != 6 {
            return Err("usage: native_runtime_qualification MODE OUTPUT DOCUMENTS CALLERS DOMAINS REPEATS; MODE=std|tokio|qualification|performance".into());
        }
        let mode = &args[0];
        if !matches!(
            mode.as_str(),
            "std" | "tokio" | "qualification" | "performance"
        ) {
            return Err("unknown qualification mode".into());
        }
        let documents = counts(&args[2], 10_000)?;
        let callers = counts(&args[3], 1024)?;
        let domains = counts(&args[4], 64)?;
        let repeats = counts(&args[5], 10)?;
        if repeats.len() != 1 {
            return Err("one repetition count required".into());
        }
        if matches!(mode.as_str(), "std" | "tokio")
            && (domains != [1] || callers.iter().any(|&n| n > 64))
        {
            return Err("single-domain drivers require DOMAINS=1 and CALLERS<=64".into());
        }
        if domains
            .iter()
            .any(|&domain| callers.iter().any(|&caller| domain > caller))
        {
            return Err("every domain needs an admitted caller".into());
        }
        Ok(Self {
            mode: mode.clone(),
            output: PathBuf::from(&args[1]),
            documents,
            callers,
            domains,
            repeats: repeats[0],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(mode: &str, docs: &str, callers: &str, domains: &str, repeats: &str) -> Vec<String> {
        [mode, "/tmp/receipt", docs, callers, domains, repeats]
            .map(str::to_owned)
            .to_vec()
    }
    #[test]
    fn complete_matrix_is_explicit() {
        let config =
            Configuration::parse(&args("qualification", "1000,10000", "64", "1,4,8", "1")).unwrap();
        assert_eq!(config.documents, [1000, 10000]);
        assert_eq!(config.domains, [1, 4, 8]);
        assert_eq!(config.callers, [64]);
        assert_eq!(config.repeats, 1);
        assert_eq!(config.output, PathBuf::from("/tmp/receipt"));
        assert_eq!(config.mode, "qualification");
    }
    #[test]
    fn invalid_or_ambiguous_matrices_fail() {
        for input in [
            args("unknown", "1000", "64", "1", "1"),
            args("qualification", "0", "64", "1", "1"),
            args("qualification", "10001", "64", "1", "1"),
            args("qualification", "1000,1000", "64", "1", "1"),
            args("qualification", "1000", "4", "8", "1"),
            args("tokio", "1000", "64", "1,4", "1"),
            args("std", "1000", "65", "1", "1"),
            args("performance", "1000", "64", "1", "1,2"),
        ] {
            assert!(Configuration::parse(&input).is_err());
        }
        assert!(Configuration::parse(&[]).is_err());
    }
}
