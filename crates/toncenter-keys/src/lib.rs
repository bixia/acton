//! `TonCenter` API key resolution shared across Acton crates.

use ton_networks::Network;

/// Environment variable for the mainnet `TonCenter` API key.
pub const TONCENTER_MAINNET_API_KEY_ENV: &str = "TONCENTER_MAINNET_API_KEY";

/// Environment variable for the testnet `TonCenter` API key.
pub const TONCENTER_TESTNET_API_KEY_ENV: &str = "TONCENTER_TESTNET_API_KEY";

/// Shared TonCenter API key env var accepted for local `.env` files.
pub const TONCENTER_SHARED_API_KEY_ENV: &str = "TON_CENTER_API_KEY";

/// Returns the `TonCenter` API key env var name for the selected network.
#[must_use]
pub fn env_var_name(network: &Network) -> Option<String> {
    match network {
        Network::Mainnet => Some(TONCENTER_MAINNET_API_KEY_ENV.to_string()),
        Network::Testnet => Some(TONCENTER_TESTNET_API_KEY_ENV.to_string()),
        Network::Localnet => None,
        Network::Custom(name) => custom_env_var_name(name),
    }
}

/// Resolves the `TonCenter` API key for the selected network from process env or local `.env`.
#[must_use]
pub fn api_key(network: &Network) -> Option<String> {
    api_key_with(network, |name| std::env::var(name).ok(), dotenv_key)
}

fn api_key_with<P, D>(network: &Network, process_lookup: P, dotenv_lookup: D) -> Option<String>
where
    P: FnMut(&str) -> Option<String>,
    D: FnMut(&str) -> Option<String>,
{
    let names = api_key_env_names(network)?;
    lookup_first(&names, process_lookup).or_else(|| lookup_first(&names, dotenv_lookup))
}

fn api_key_env_names(network: &Network) -> Option<Vec<String>> {
    let mut names = vec![env_var_name(network)?];
    if matches!(network, Network::Mainnet | Network::Testnet) {
        names.push(TONCENTER_SHARED_API_KEY_ENV.to_string());
    }
    Some(names)
}

fn lookup_first<F>(names: &[String], mut lookup: F) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    names.iter().find_map(|name| {
        lookup(name)
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    })
}

fn dotenv_key(name: &str) -> Option<String> {
    dotenvy::from_filename_iter(".env").ok()?.find_map(|entry| {
        let (key, value) = entry.ok()?;
        (key == name).then_some(value)
    })
}

/// Returns the TonCenter-compatible API key env var name for a custom network.
///
/// `custom:foo` becomes `FOO_API_KEY`, and non-alphanumeric characters are normalized to `_`.
#[must_use]
pub fn custom_env_var_name(name: &str) -> Option<String> {
    let mut normalized = String::new();
    let mut last_was_separator = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_uppercase());
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        }
    }

    let normalized = normalized.trim_matches('_');
    if normalized.is_empty() {
        None
    } else {
        Some(format!("{normalized}_API_KEY"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_mainnet_key_from_mainnet_env() {
        let lookup = |name: &str| match name {
            TONCENTER_MAINNET_API_KEY_ENV => Some(" mainnet-key ".to_string()),
            _ => None,
        };

        assert_eq!(
            api_key_with(&Network::Mainnet, lookup, |_| None),
            Some("mainnet-key".to_string())
        );
        assert_eq!(api_key_with(&Network::Testnet, |_| None, |_| None), None);
    }

    #[test]
    fn resolves_testnet_key_from_testnet_env() {
        let lookup = |name: &str| match name {
            TONCENTER_TESTNET_API_KEY_ENV => Some("testnet-key".to_string()),
            _ => None,
        };

        assert_eq!(
            api_key_with(&Network::Testnet, lookup, |_| None),
            Some("testnet-key".to_string())
        );
        assert_eq!(api_key_with(&Network::Mainnet, |_| None, |_| None), None);
    }

    #[test]
    fn falls_back_to_shared_key_from_dotenv() {
        let dotenv_lookup = |name: &str| match name {
            TONCENTER_SHARED_API_KEY_ENV => Some(" shared-key ".to_string()),
            _ => None,
        };

        assert_eq!(
            api_key_with(&Network::Mainnet, |_| None, dotenv_lookup),
            Some("shared-key".to_string())
        );
    }

    #[test]
    fn prefers_network_specific_process_env_over_dotenv_alias() {
        let process_lookup = |name: &str| match name {
            TONCENTER_MAINNET_API_KEY_ENV => Some(" mainnet-key ".to_string()),
            _ => None,
        };
        let dotenv_lookup = |name: &str| match name {
            TONCENTER_SHARED_API_KEY_ENV => Some(" shared-key ".to_string()),
            _ => None,
        };

        assert_eq!(
            api_key_with(&Network::Mainnet, process_lookup, dotenv_lookup),
            Some("mainnet-key".to_string())
        );
    }

    #[test]
    fn does_not_resolve_for_localnet_or_custom_networks() {
        assert_eq!(api_key(&Network::Localnet), None);
    }

    #[test]
    fn resolves_custom_network_key_from_uppercase_env() {
        let lookup = |name: &str| match name {
            "SANDBOX_API_KEY" => Some("custom-key".to_string()),
            _ => None,
        };

        assert_eq!(
            api_key_with(&Network::Custom("sandbox".into()), lookup, |_| None),
            Some("custom-key".to_string())
        );
    }

    #[test]
    fn normalizes_custom_network_env_names() {
        assert_eq!(
            custom_env_var_name("mock-remote"),
            Some("MOCK_REMOTE_API_KEY".to_string())
        );
        assert_eq!(
            custom_env_var_name("alpha.beta/gamma"),
            Some("ALPHA_BETA_GAMMA_API_KEY".to_string())
        );
        assert_eq!(custom_env_var_name("---"), None);
    }

    #[test]
    fn ignores_empty_values_after_trimming() {
        assert_eq!(
            api_key_with(&Network::Mainnet, |_| Some("   ".into()), |_| None),
            None
        );
    }
}
