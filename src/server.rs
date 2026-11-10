//! The server command, without Zed: the pinned release, its jar and how
//! `java` is chosen and started. Unit-tested on the host.

use sha2::{Digest, Sha256};

/// The GitHub repository the server releases come from.
pub const REPO: &str = "socless/socle";

/// The Socle release this extension is tested against.
pub const PINNED: &str = "0.25.1";

/// The sha256 of the PINNED release's socle-lsp jar. Bump it together with PINNED.
pub const PINNED_SHA256: &str = "546425886acbe57b7fb4432fd0b7af32f3df1e5c0eec0c04a8128631fa173cf4";

pub fn jar_name(version: &str) -> String {
    format!("socle-lsp-{version}.jar")
}

pub fn jar_url(version: &str) -> String {
    format!("https://github.com/{REPO}/releases/download/v{version}/{}", jar_name(version))
}

/// The sha256 of `bytes`, in lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// The arguments that start the server in `jar` (as socle.nvim does).
pub fn default_args(jar: &str) -> Vec<String> {
    vec!["-Djava.net.preferIPv4Stack=true".into(), "-jar".into(), jar.into()]
}

/// The `java` to run: the `lsp.socle.binary.path` setting, else
/// `$JAVA_HOME/bin/java`, else `java` on `PATH`.
pub fn java(setting: Option<String>, java_home: Option<String>, on_path: Option<String>) -> Option<String> {
    setting
        .or_else(|| java_home.filter(|h| !h.is_empty()).map(|h| format!("{}/bin/java", h.trim_end_matches('/'))))
        .or(on_path)
}

/// Why there is no server without a jar: the download failed (offline, a
/// proxy). Say how to use a local jar instead.
pub fn download_failed(url: &str, err: &str) -> String {
    format!(
        "Socle: cannot download {url} ({err}). To use a local jar, build it \
         (./mill socle-lsp.assembly) and set lsp.socle.binary.arguments to \
         [\"-jar\", \"/path/to/socle-lsp.jar\"] in Zed's settings (see the README)"
    )
}

pub const NO_JAVA: &str =
    "Socle: no java found (Java 21 or newer): set JAVA_HOME, put java on PATH or set lsp.socle.binary.path";

pub const NOT_MILL: &str =
    "Socle: no build.mill at the root of this worktree; the Socle server runs only in Mill projects";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_of_the_pinned_jar() {
        assert_eq!(
            jar_url(PINNED),
            "https://github.com/socless/socle/releases/download/v0.25.1/socle-lsp-0.25.1.jar"
        );
    }

    #[test]
    fn sha256_in_hex() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(PINNED_SHA256.len(), 64);
    }

    #[test]
    fn java_setting_then_java_home_then_path() {
        let s = |v: &str| Some(v.to_string());
        assert_eq!(java(s("/opt/j/java"), s("/jdk"), s("/usr/bin/java")), s("/opt/j/java"));
        assert_eq!(java(None, s("/jdk/"), s("/usr/bin/java")), s("/jdk/bin/java"));
        assert_eq!(java(None, s(""), s("/usr/bin/java")), s("/usr/bin/java"));
        assert_eq!(java(None, None, None), None);
    }

    #[test]
    fn default_args_run_the_jar() {
        assert_eq!(default_args("/x/s.jar"), ["-Djava.net.preferIPv4Stack=true", "-jar", "/x/s.jar"]);
    }

    #[test]
    fn download_error_names_the_setting() {
        let m = download_failed("https://u", "404");
        assert!(m.contains("https://u (404)") && m.contains("lsp.socle.binary.arguments"));
    }
}
