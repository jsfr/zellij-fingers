use std::collections::{BTreeMap, HashMap};

use crate::ansi;

pub struct Config {
    pub action: String,
    pub hint_position: String,
    pub hint_style: String,
    pub highlight_style: String,
    pub selected_hint_style: String,
    pub selected_highlight_style: String,
    pub backdrop_style: String,
    pub clipboard_command: Option<String>,
    pub open_command: Option<String>,
    pub alphabet: Vec<String>,
    pub patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let alphabet = alphabet_for("qwerty");
        let patterns = all_builtin_patterns();

        Self {
            action: ":copy:".to_string(),
            hint_position: "left".to_string(),
            hint_style: ansi::format_style("fg=green,bold"),
            highlight_style: ansi::format_style("fg=yellow"),
            selected_hint_style: ansi::format_style("fg=blue,bold"),
            selected_highlight_style: ansi::format_style("fg=blue"),
            backdrop_style: ansi::format_style("dim"),
            clipboard_command: None,
            open_command: None,
            alphabet,
            patterns,
        }
    }
}

impl Config {
    pub fn from_kdl(config: &BTreeMap<String, String>) -> Self {
        let keyboard_layout = config
            .get("keyboard_layout")
            .cloned()
            .unwrap_or_else(|| "qwerty".to_string());

        let alphabet = alphabet_for(&keyboard_layout);

        let enabled_builtin_patterns = config
            .get("enabled_builtin_patterns")
            .cloned()
            .unwrap_or_else(|| "all".to_string());

        let mut patterns = resolve_builtin_patterns(&enabled_builtin_patterns);

        // Collect user patterns (pattern_0, pattern_1, ...)
        let mut user_patterns = Vec::new();
        for i in 0..20 {
            if let Some(p) = config.get(&format!("pattern_{i}")) {
                user_patterns.push(p.clone());
                patterns.push(p.clone());
            }
        }

        let hint_style = config
            .get("hint_style")
            .map(|s| ansi::format_style(s))
            .unwrap_or_else(|| ansi::format_style("fg=green,bold"));

        let highlight_style = config
            .get("highlight_style")
            .map(|s| ansi::format_style(s))
            .unwrap_or_else(|| ansi::format_style("fg=yellow"));

        let selected_hint_style = config
            .get("selected_hint_style")
            .map(|s| ansi::format_style(s))
            .unwrap_or_else(|| ansi::format_style("fg=blue,bold"));

        let selected_highlight_style = config
            .get("selected_highlight_style")
            .map(|s| ansi::format_style(s))
            .unwrap_or_else(|| ansi::format_style("fg=blue"));

        let backdrop_style = config
            .get("backdrop_style")
            .map(|s| ansi::format_style(s))
            .unwrap_or_default();

        Self {
            action: config
                .get("action")
                .cloned()
                .unwrap_or_else(|| ":copy:".to_string()),
            hint_position: config
                .get("hint_position")
                .cloned()
                .unwrap_or_else(|| "left".to_string()),
            hint_style,
            highlight_style,
            selected_hint_style,
            selected_highlight_style,
            backdrop_style,
            clipboard_command: config.get("clipboard_command").cloned(),
            open_command: config.get("open_command").cloned(),
            alphabet,
            patterns,
        }
    }
}

fn resolve_builtin_patterns(enabled: &str) -> Vec<String> {
    if enabled == "all" {
        return all_builtin_patterns();
    }

    let builtins = builtin_patterns();
    enabled
        .split(',')
        .filter_map(|name| builtins.get(name.trim()).cloned())
        .collect()
}

pub fn all_builtin_patterns() -> Vec<String> {
    builtin_patterns().values().cloned().collect()
}

pub fn builtin_patterns() -> HashMap<&'static str, String> {
    HashMap::from([
        ("ip", r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}".to_string()),
        (
            "uuid",
            r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}".to_string(),
        ),
        ("sha", r"[0-9a-f]{7,128}".to_string()),
        ("digit", r"[0-9]{4,}".to_string()),
        (
            "url",
            r"((https?://|git@|git://|ssh://|ftp://|file:///)[^\s()\x22']+)".to_string(),
        ),
        (
            "path",
            r"(([.\w\-~\$@]+)?(/[.\w\-@]+)+/?)".to_string(),
        ),
        ("hex", r"(0x[0-9a-fA-F]+)".to_string()),
        (
            "kubernetes",
            concat!(
                r"(deployment\.app|binding|componentstatuse|configmap|endpoint|event|",
                r"limitrange|namespace|node|persistentvolumeclaim|persistentvolume|pod|",
                r"podtemplate|replicationcontroller|resourcequota|secret|serviceaccount|",
                r"service|mutatingwebhookconfiguration\.admissionregistration\.k8s\.io|",
                r"validatingwebhookconfiguration\.admissionregistration\.k8s\.io|",
                r"customresourcedefinition\.apiextension\.k8s\.io|",
                r"apiservice\.apiregistration\.k8s\.io|controllerrevision\.apps|",
                r"daemonset\.apps|deployment\.apps|replicaset\.apps|statefulset\.apps|",
                r"tokenreview\.authentication\.k8s\.io|",
                r"localsubjectaccessreview\.authorization\.k8s\.io|",
                r"selfsubjectaccessreviews\.authorization\.k8s\.io|",
                r"selfsubjectrulesreview\.authorization\.k8s\.io|",
                r"subjectaccessreview\.authorization\.k8s\.io|",
                r"horizontalpodautoscaler\.autoscaling|cronjob\.batch|job\.batch|",
                r"certificatesigningrequest\.certificates\.k8s\.io|",
                r"events\.events\.k8s\.io|daemonset\.extensions|deployment\.extensions|",
                r"ingress\.extensions|networkpolicies\.extensions|",
                r"podsecuritypolicies\.extensions|replicaset\.extensions|",
                r"networkpolicie\.networking\.k8s\.io|",
                r"poddisruptionbudget\.policy|",
                r"clusterrolebinding\.rbac\.authorization\.k8s\.io|",
                r"clusterrole\.rbac\.authorization\.k8s\.io|",
                r"rolebinding\.rbac\.authorization\.k8s\.io|",
                r"role\.rbac\.authorization\.k8s\.io|",
                r"storageclasse\.storage\.k8s\.io)",
                r"[a-zA-Z0-9_#$%&+=/@-]+"
            )
            .to_string(),
        ),
        (
            "git-status",
            r"(modified|deleted|deleted by us|new file): +(?P<match>.+)".to_string(),
        ),
        (
            "git-status-branch",
            r"Your branch is up to date with '(?P<match>.*)'\.".to_string(),
        ),
        (
            "diff",
            r"(---|\+\+\+) [ab]/(?P<match>.*)".to_string(),
        ),
    ])
}

pub fn alphabet_map() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("qwerty", "asdfqwerzxcvjklmiuopghtybn"),
        ("qwerty-homerow", "asdfjklgh"),
        ("qwerty-left-hand", "asdfqwerzcxv"),
        ("qwerty-right-hand", "jkluiopmyhn"),
        ("azerty", "qsdfazerwxcvjklmuiopghtybn"),
        ("azerty-homerow", "qsdfjkmgh"),
        ("azerty-left-hand", "qsdfazerwxcv"),
        ("azerty-right-hand", "jklmuiophyn"),
        ("qwertz", "asdfqweryxcvjkluiopmghtzbn"),
        ("qwertz-homerow", "asdfghjkl"),
        ("qwertz-left-hand", "asdfqweryxcv"),
        ("qwertz-right-hand", "jkluiopmhzn"),
        ("dvorak", "aoeuqjkxpyhtnsgcrlmwvzfidb"),
        ("dvorak-homerow", "aoeuhtnsid"),
        ("dvorak-left-hand", "aoeupqjkyix"),
        ("dvorak-right-hand", "htnsgcrlmwvz"),
        ("colemak", "arstqwfpzxcvneioluymdhgjbk"),
        ("colemak-homerow", "arstneiodh"),
        ("colemak-left-hand", "arstqwfpzxcv"),
        ("colemak-right-hand", "neioluymjhk"),
    ])
}

pub fn alphabet_for(layout: &str) -> Vec<String> {
    let map = alphabet_map();
    let chars = map.get(layout).unwrap_or(&"asdfqwerzxcvjklmiuopghtybn");
    chars.chars().map(|c| c.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    fn matches_for(pattern_name: &str, input: &str) -> Vec<String> {
        let patterns = builtin_patterns();
        let pattern_str = patterns.get(pattern_name).unwrap();
        let re = Regex::new(pattern_str).unwrap();

        re.captures_iter(input)
            .map(|cap| {
                cap.name("match")
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| cap[0].to_string())
            })
            .collect()
    }

    #[test]
    fn ip_matches() {
        let input = "
      foo
        192.168.0.1
        127.0.0.1
        foofofo
      ";
        assert_eq!(
            matches_for("ip", input),
            vec!["192.168.0.1", "127.0.0.1"]
        );
    }

    #[test]
    fn uuid_matches() {
        let input = "
      foo
      d6f4b4ac-4b78-4d79-96a1-eb9ab72f2c59
      7a8e24d1-5a81-4f5a-bc6a-9d7f9818a8c4
      e5c3dcf0-9b01-45c2-8327-6d9d4bb8a0c8
      2fa5c6e9-33f9-46b7-ba89-3f17b12e59e5
      b882bfc5-6b24-43a7-ae1e-8f9ea14eeff2
      bar
      ";
        assert_eq!(
            matches_for("uuid", input),
            vec![
                "d6f4b4ac-4b78-4d79-96a1-eb9ab72f2c59",
                "7a8e24d1-5a81-4f5a-bc6a-9d7f9818a8c4",
                "e5c3dcf0-9b01-45c2-8327-6d9d4bb8a0c8",
                "2fa5c6e9-33f9-46b7-ba89-3f17b12e59e5",
                "b882bfc5-6b24-43a7-ae1e-8f9ea14eeff2",
            ]
        );
    }

    #[test]
    fn sha_matches() {
        let input = "
      foo
      fc4fea27210bc0d85b74f40866e12890e3788134
      fc4fea2
      bar
      ";
        assert_eq!(
            matches_for("sha", input),
            vec![
                "fc4fea27210bc0d85b74f40866e12890e3788134",
                "fc4fea2"
            ]
        );
    }

    #[test]
    fn digit_matches() {
        let input = "
      foo
      12345
      67891011
      bar
      ";
        assert_eq!(matches_for("digit", input), vec!["12345", "67891011"]);
    }

    #[test]
    fn url_matches() {
        let input = "
      foo
      https://geocities.com
      bar
      ";
        assert_eq!(matches_for("url", input), vec!["https://geocities.com"]);
    }

    #[test]
    fn path_matches() {
        let input = "
      absolute paths /foo/bar/lol
      relative paths ./foo/bar/lol
      home paths ~/foo/bar/lol
      bar
      ";
        assert_eq!(
            matches_for("path", input),
            vec!["/foo/bar/lol", "./foo/bar/lol", "~/foo/bar/lol"]
        );
    }

    #[test]
    fn hex_matches() {
        let input = "
      hello 0xcafe
      0xcaca
      0xdeadbeef hehehe 0xCACA
      ";
        assert_eq!(
            matches_for("hex", input),
            vec!["0xcafe", "0xcaca", "0xdeadbeef", "0xCACA"]
        );
    }

    #[test]
    fn git_status_matches() {
        let input = r#"
Your branch is up to date with 'origin/crystal-rewrite'.

Changes to be committed:
  (use "git restore --staged <file>..." to unstage)
        deleted:    CHANGELOG.md
        new file:   wat

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
        modified:   Makefile
        modified:   spec/lib/patterns_spec.cr
        modified:   src/fingers/config.cr
      "#;
        assert_eq!(
            matches_for("git-status", input),
            vec![
                "CHANGELOG.md",
                "wat",
                "Makefile",
                "spec/lib/patterns_spec.cr",
                "src/fingers/config.cr"
            ]
        );
    }

    #[test]
    fn git_status_branch_matches() {
        let input = "
Your branch is up to date with 'origin/crystal-rewrite'.

Changes to be committed:
      ";
        assert_eq!(
            matches_for("git-status-branch", input),
            vec!["origin/crystal-rewrite"]
        );
    }

    #[test]
    fn git_diff_matches() {
        let input = "
  diff --git a/spec/lib/patterns_spec.cr b/spec/lib/patterns_spec.cr
  index 5281097..6c9c18e 100644
  --- a/spec/lib/patterns_spec.cr
  +++ b/spec/lib/patterns_spec.cr
  ";
        assert_eq!(
            matches_for("diff", input),
            vec!["spec/lib/patterns_spec.cr", "spec/lib/patterns_spec.cr"]
        );
    }

    #[test]
    fn alphabet_for_qwerty() {
        let a = alphabet_for("qwerty");
        assert_eq!(a[0], "a");
        assert_eq!(a[1], "s");
        assert_eq!(a.len(), 26);
    }

    #[test]
    fn alphabet_for_unknown_falls_back() {
        let a = alphabet_for("unknown");
        assert_eq!(a.len(), 26); // falls back to qwerty
    }

    #[test]
    fn from_kdl_defaults() {
        let config = Config::from_kdl(&BTreeMap::new());
        assert_eq!(config.action, ":copy:");
        assert_eq!(config.hint_position, "left");
        assert!(!config.patterns.is_empty());
    }

    #[test]
    fn from_kdl_custom_values() {
        let mut map = BTreeMap::new();
        map.insert("keyboard_layout".to_string(), "dvorak".to_string());
        map.insert("action".to_string(), ":open:".to_string());
        map.insert("pattern_0".to_string(), r"\bfoo\b".to_string());

        let config = Config::from_kdl(&map);
        assert_eq!(config.action, ":open:");
        assert!(config.patterns.iter().any(|p| p == r"\bfoo\b"));
    }
}
