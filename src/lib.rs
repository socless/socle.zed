//! Socle in Zed: starts the Socle language server (`socle-lsp`) on the
//! Scala files of Mill projects. The server does the rest over plain LSP:
//! the View design / Simulate code lenses and the live panel in the browser.

mod server;

use std::fs;

use zed_extension_api::{
    self as zed, serde_json, settings::LspSettings, LanguageServerId, LanguageServerInstallationStatus, Result,
};

/// The name of the server in Zed's `lsp` settings and in extension.toml.
const SERVER: &str = "socle";

struct Socle {
    /// The pinned jar once it is there and its sha256 checked.
    jar: Option<String>,
}

impl Socle {
    /// The pinned server jar, downloaded into the extension's directory and
    /// checked against its sha256 the first time.
    fn jar(&mut self, id: &LanguageServerId) -> Result<String> {
        if let Some(jar) = &self.jar {
            return Ok(jar.clone());
        }
        let name = server::jar_name(server::PINNED);
        if !matches(&name) {
            let url = server::jar_url(server::PINNED);
            let part = format!("{name}.part");
            zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::Downloading);
            let fetched = zed::download_file(&url, &part, zed::DownloadedFileType::Uncompressed)
                .map_err(|e| server::download_failed(&url, &e))
                .and_then(|()| {
                    if matches(&part) {
                        fs::rename(&part, &name).map_err(|e| format!("Socle: cannot keep {name}: {e}"))
                    } else {
                        Err(format!("Socle: {url} does not have the expected sha256 {}", server::PINNED_SHA256))
                    }
                });
            let _ = fs::remove_file(&part);
            if let Err(e) = fetched {
                zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::Failed(e.clone()));
                return Err(e);
            }
            zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::None);
        }
        let jar = std::env::current_dir().map_err(|e| e.to_string())?.join(&name).to_string_lossy().into_owned();
        self.jar = Some(jar.clone());
        Ok(jar)
    }
}

/// Whether `path` (in the extension's directory) is the pinned jar.
fn matches(path: &str) -> bool {
    fs::read(path).is_ok_and(|bytes| server::sha256_hex(&bytes) == server::PINNED_SHA256)
}

impl zed::Extension for Socle {
    fn new() -> Self {
        Socle { jar: None }
    }

    fn language_server_command(&mut self, id: &LanguageServerId, worktree: &zed::Worktree) -> Result<zed::Command> {
        if worktree.read_text_file("build.mill").is_err() {
            return Err(server::NOT_MILL.into());
        }
        let binary = LspSettings::for_worktree(SERVER, worktree).ok().and_then(|s| s.binary);
        let (path, arguments, env) = match binary {
            Some(b) => (b.path, b.arguments, b.env),
            None => (None, None, None),
        };
        let java_home = worktree.shell_env().into_iter().find(|(k, _)| k == "JAVA_HOME").map(|(_, v)| v);
        let command = server::java(path, java_home, worktree.which("java")).ok_or(server::NO_JAVA)?;
        let args = match arguments {
            Some(args) => args,
            None => server::default_args(&self.jar(id)?),
        };
        Ok(zed::Command { command, args, env: env.map(|e| e.into_iter().collect()).unwrap_or_default() })
    }

    /// `lsp.socle.initialization_options`, flat: the wave settings.
    fn language_server_initialization_options(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER, worktree).ok().and_then(|s| s.initialization_options))
    }

    /// The same options under `socle`, the section the server reads from
    /// `workspace/didChangeConfiguration`: a settings change applies without a restart.
    fn language_server_workspace_configuration(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(self
            .language_server_initialization_options(id, worktree)?
            .map(|o| serde_json::json!({ "socle": o })))
    }
}

zed::register_extension!(Socle);
