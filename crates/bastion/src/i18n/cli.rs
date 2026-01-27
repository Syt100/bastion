use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use clap::{Arg, Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliLocale {
    EnUs,
    ZhCn,
}

impl CliLocale {
    // Intentionally no `as_str` for now: keep the API minimal and avoid dead-code warnings.
}

pub fn resolve_cli_locale() -> CliLocale {
    resolve_cli_locale_from_vars(
        std::env::var("BASTION_LANG").ok().as_deref(),
        std::env::var("LC_ALL").ok().as_deref(),
        std::env::var("LC_MESSAGES").ok().as_deref(),
        std::env::var("LANG").ok().as_deref(),
    )
}

fn resolve_cli_locale_from_vars(
    bastion_lang: Option<&str>,
    lc_all: Option<&str>,
    lc_messages: Option<&str>,
    lang: Option<&str>,
) -> CliLocale {
    if let Some(v) = bastion_lang.and_then(parse_bastion_lang) {
        return v;
    }

    for v in [lc_all, lc_messages, lang] {
        if v.is_some_and(locale_indicates_zh) {
            return CliLocale::ZhCn;
        }
    }

    CliLocale::EnUs
}

fn parse_bastion_lang(v: &str) -> Option<CliLocale> {
    let v = normalize_lang(v);
    if v.is_empty() {
        return None;
    }

    if v.starts_with("zh") {
        return Some(CliLocale::ZhCn);
    }
    if v.starts_with("en") {
        return Some(CliLocale::EnUs);
    }

    None
}

fn locale_indicates_zh(v: &str) -> bool {
    normalize_lang(v).starts_with("zh")
}

fn normalize_lang(v: &str) -> String {
    // Examples:
    // - zh_CN.UTF-8 -> zh-cn
    // - zh-Hans-CN  -> zh-hans-cn
    // - en_US       -> en-us
    let trimmed = v.trim();
    let stripped = trimmed
        .split_once('.')
        .map(|(head, _)| head)
        .unwrap_or(trimmed)
        .split_once('@')
        .map(|(head, _)| head)
        .unwrap_or(trimmed);
    stripped.replace('_', "-").to_ascii_lowercase()
}

pub fn localize_command(cmd: Command, locale: CliLocale) -> Command {
    match locale {
        CliLocale::EnUs => cmd,
        CliLocale::ZhCn => localize_command_zh(cmd),
    }
}

fn localize_command_zh(cmd: Command) -> Command {
    let zh = zh_translations();
    let root = cmd.get_name().to_string();
    localize_command_inner(cmd, &root, zh)
}

fn localize_command_inner(mut cmd: Command, path: &str, zh: &BTreeMap<String, String>) -> Command {
    if let Some(about) = zh.get(&format!("{path}.about")) {
        cmd = cmd.about(about.clone()).long_about(about.clone());
    }

    let template = zh_help_template(&cmd);
    cmd = cmd.help_template(template);

    cmd = cmd.mut_args(|arg| localize_arg(arg, path, zh));

    cmd = cmd.mut_subcommands(|sub| {
        let name = sub.get_name().to_string();
        let sub_path = format!("{path}.{name}");
        localize_command_inner(sub, &sub_path, zh)
    });

    cmd
}

fn localize_arg(mut arg: Arg, cmd_path: &str, zh: &BTreeMap<String, String>) -> Arg {
    let id = arg.get_id().as_str();
    if id == "help" || id == "version" {
        return arg;
    }

    let base = format!("{cmd_path}.arg.{id}");

    if let Some(help) = zh.get(&format!("{base}.help")) {
        arg = arg.help(help.clone());
    }
    if let Some(long_help) = zh.get(&format!("{base}.long_help")) {
        arg = arg.long_help(long_help.clone());
    }

    arg
}

fn zh_help_template(cmd: &Command) -> String {
    let has_positionals = cmd
        .get_arguments()
        .any(|a| a.is_positional() && !a.is_hide_set());
    let has_options = cmd
        .get_arguments()
        .any(|a| !a.is_positional() && !a.is_hide_set());
    let has_subcommands = cmd
        .get_subcommands()
        .any(|sc| sc.get_name() != "help" && !sc.is_hide_set());

    // We avoid `{usage-heading}` / `{all-args}` because clap renders them with English headings.
    // Instead, we render our own headings and let clap render the bodies.
    let mut tmpl = String::from("{before-help}{about-with-newline}\n");
    tmpl.push_str("用法: {usage}\n");

    if has_positionals {
        tmpl.push_str("\n参数:\n{positionals}\n");
    }
    if has_options {
        tmpl.push_str("\n选项:\n{options}\n");
    }
    if has_subcommands {
        tmpl.push_str("\n命令:\n{subcommands}\n");
    }

    tmpl.push_str("{after-help}");
    tmpl
}

fn zh_translations() -> &'static BTreeMap<String, String> {
    static ZH: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    ZH.get_or_init(|| {
        let raw = include_str!("cli.zh-CN.json");
        serde_json::from_str::<BTreeMap<String, String>>(raw)
            .expect("invalid cli.zh-CN.json translation map")
    })
}

pub fn required_zh_translation_keys(cmd: &Command) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let root = cmd.get_name().to_string();
    collect_required_keys(cmd, &root, &mut keys);
    keys
}

fn collect_required_keys(cmd: &Command, path: &str, keys: &mut BTreeSet<String>) {
    if cmd.get_about().is_some() || cmd.get_long_about().is_some() {
        keys.insert(format!("{path}.about"));
    }

    for arg in cmd.get_arguments() {
        if arg.is_hide_set() {
            continue;
        }

        let id = arg.get_id().as_str();
        if id == "help" || id == "version" {
            continue;
        }

        let base = format!("{path}.arg.{id}");

        if arg.get_help().is_some() {
            keys.insert(format!("{base}.help"));
        }
        if arg.get_long_help().is_some() {
            keys.insert(format!("{base}.long_help"));
        }
    }

    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }

        let name = sub.get_name();
        let sub_path = format!("{path}.{name}");
        collect_required_keys(sub, &sub_path, keys);
    }
}

#[allow(dead_code)] // Used by the `docgen` binary (compiled as a separate target).
pub fn missing_zh_translation_keys(cmd: &Command) -> BTreeSet<String> {
    let required = required_zh_translation_keys(cmd);
    let have = zh_translations()
        .keys()
        .map(|k| k.to_string())
        .collect::<BTreeSet<_>>();
    required.difference(&have).cloned().collect::<BTreeSet<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction};

    #[test]
    fn resolve_cli_locale_prefers_bastion_lang() {
        assert_eq!(
            CliLocale::EnUs,
            resolve_cli_locale_from_vars(Some("en-US"), Some("zh_CN.UTF-8"), None, None)
        );
        assert_eq!(
            CliLocale::ZhCn,
            resolve_cli_locale_from_vars(Some("zh-CN"), Some("en_US.UTF-8"), None, None)
        );
    }

    #[test]
    fn resolve_cli_locale_uses_system_locale_fallback() {
        assert_eq!(
            CliLocale::ZhCn,
            resolve_cli_locale_from_vars(None, Some("zh_CN.UTF-8"), None, None)
        );
        assert_eq!(
            CliLocale::ZhCn,
            resolve_cli_locale_from_vars(None, None, Some("zh_CN.UTF-8"), None)
        );
        assert_eq!(
            CliLocale::ZhCn,
            resolve_cli_locale_from_vars(None, None, None, Some("zh_CN.UTF-8"))
        );
        assert_eq!(
            CliLocale::EnUs,
            resolve_cli_locale_from_vars(None, Some("en_US.UTF-8"), None, None)
        );
    }

    #[test]
    fn required_keys_skips_hidden_and_builtin_help_version() {
        let cmd = Command::new("bastion")
            .about("about")
            .arg(
                Arg::new("visible")
                    .long("visible")
                    .help("visible help")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("hidden")
                    .long("hidden")
                    .help("hidden help")
                    .hide(true)
                    .action(ArgAction::SetTrue),
            )
            // Simulate built-in ids which we intentionally ignore.
            .arg(
                Arg::new("help")
                    .long("help")
                    .help("help")
                    .action(ArgAction::HelpLong),
            )
            .arg(
                Arg::new("version")
                    .long("version")
                    .help("version")
                    .action(ArgAction::Version),
            );

        let keys = required_zh_translation_keys(&cmd);
        assert!(keys.contains("bastion.about"));
        assert!(keys.contains("bastion.arg.visible.help"));
        assert!(!keys.iter().any(|k| k.contains(".arg.hidden.")));
        assert!(!keys.iter().any(|k| k.contains(".arg.help.")));
        assert!(!keys.iter().any(|k| k.contains(".arg.version.")));
    }
}
