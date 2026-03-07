pub const VERSION: &str = match option_env!("BB_BUILD_VERSION") {
    Some(value) if !value.is_empty() => value,
    _ => env!("CARGO_PKG_VERSION"),
};
pub const COMMIT: &str = match option_env!("BB_BUILD_COMMIT") {
    Some(value) => value,
    None => "unknown",
};
pub const BUILD_DATE: &str = match option_env!("BB_BUILD_DATE") {
    Some(value) => value,
    None => "unknown",
};

pub fn short_commit() -> &'static str {
    if COMMIT.trim().is_empty() || COMMIT == "unknown" {
        return "unknown";
    }
    if COMMIT.len() > 7 {
        &COMMIT[..7]
    } else {
        COMMIT
    }
}

pub fn display_version() -> String {
    if short_commit() == "unknown" || VERSION.contains('+') {
        return VERSION.to_string();
    }
    format!("{VERSION}+{}", short_commit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_version_uses_semver() {
        assert!(display_version().starts_with(VERSION));
    }
}
