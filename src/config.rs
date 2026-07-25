use std::{env, ffi::OsString, fmt, net::SocketAddr, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub root: PathBuf,
    pub bind_addr: SocketAddr,
    pub spa: bool,
}

#[derive(Debug)]
pub struct ConfigError(String);

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_values(
            env::var_os("SITEVIK_ROOT"),
            env::var_os("BIND_ADDR"),
            env::var_os("SITEVIK_SPA"),
        )
    }

    pub fn from_values(
        root_value: Option<OsString>,
        bind_addr_value: Option<OsString>,
        spa_value: Option<OsString>,
    ) -> Result<Self, ConfigError> {
        let root = root_from_value(root_value);
        let metadata = std::fs::read_dir(&root)
            .and_then(|_| std::fs::metadata(&root))
            .map_err(|_| ConfigError("invalid SITEVIK_ROOT".into()))?;
        if !metadata.is_dir() {
            return Err(ConfigError("invalid SITEVIK_ROOT".into()));
        }

        let bind_addr = bind_addr_value
            .unwrap_or_else(|| "0.0.0.0:8080".into())
            .into_string()
            .map_err(|_| ConfigError("invalid BIND_ADDR".into()))?
            .parse()
            .map_err(|_| ConfigError("invalid BIND_ADDR".into()))?;

        let spa = match spa_value {
            None => false,
            Some(value) => match value.to_str() {
                Some("false") => false,
                Some("true") => true,
                _ => return Err(ConfigError("invalid SITEVIK_SPA".into())),
            },
        };

        Ok(Self {
            root,
            bind_addr,
            spa,
        })
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

fn root_from_value(value: Option<OsString>) -> PathBuf {
    value
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./dist"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[cfg(unix)]
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    use super::{Config, root_from_value};

    #[test]
    fn root_value_defaults_to_dist() {
        assert_eq!(root_from_value(None), PathBuf::from("./dist"));
    }

    #[test]
    fn bind_and_spa_defaults_are_applied() {
        let root = tempfile::tempdir().unwrap();
        let config =
            Config::from_values(Some(root.path().as_os_str().to_owned()), None, None).unwrap();

        assert_eq!(config.root, root.path());
        assert_eq!(config.bind_addr, "0.0.0.0:8080".parse().unwrap());
        assert!(!config.spa);
    }

    #[test]
    fn spa_boolean_is_strict() {
        let root = tempfile::tempdir().unwrap();
        let error = Config::from_values(
            Some(root.path().as_os_str().to_owned()),
            None,
            Some("yes".into()),
        )
        .unwrap_err();

        assert!(error.to_string().contains("SITEVIK_SPA"));
    }

    #[test]
    fn spa_true_is_accepted() {
        let root = tempfile::tempdir().unwrap();
        let config = Config::from_values(
            Some(root.path().as_os_str().to_owned()),
            None,
            Some("true".into()),
        )
        .unwrap();

        assert!(config.spa);
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_spa_value_is_rejected() {
        let root = tempfile::tempdir().unwrap();
        let error = Config::from_values(
            Some(root.path().as_os_str().to_owned()),
            None,
            Some(OsString::from_vec(vec![0xff])),
        )
        .unwrap_err();

        assert!(error.to_string().contains("SITEVIK_SPA"));
    }

    #[test]
    fn invalid_bind_address_is_rejected() {
        let root = tempfile::tempdir().unwrap();
        let error = Config::from_values(
            Some(root.path().as_os_str().to_owned()),
            Some("localhost".into()),
            None,
        )
        .unwrap_err();

        assert!(error.to_string().contains("BIND_ADDR"));
    }

    #[test]
    fn non_directory_root_is_rejected() {
        let root = tempfile::NamedTempFile::new().unwrap();
        let error =
            Config::from_values(Some(root.path().as_os_str().to_owned()), None, None).unwrap_err();

        assert!(error.to_string().contains("SITEVIK_ROOT"));
    }
}
