use zed_extension_api as zed;

const PACKAGE_NAME: &str = "silverstripe-language-server";
const SERVER_PATH: &str = "node_modules/.bin/silverstripe-language-server";

struct SilverStripeExtension;

impl zed::Extension for SilverStripeExtension {
    fn new() -> Self {
        SilverStripeExtension
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let server_exists = std::fs::metadata(SERVER_PATH).map_or(false, |stat| stat.is_file());

        let latest_version = zed::npm_package_latest_version(PACKAGE_NAME)?;
        let installed_version = zed::npm_package_installed_version(PACKAGE_NAME)?;

        let needs_install = !server_exists
            || installed_version.as_deref() != Some(latest_version.as_str());

        if needs_install {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::npm_install_package(PACKAGE_NAME, &latest_version)?;
        }

        // Verify the server binary exists after install
        if !std::fs::metadata(SERVER_PATH).map_or(false, |stat| stat.is_file()) {
            // Fall back to a globally installed binary
            if let Some(path) = worktree.which(PACKAGE_NAME) {
                return Ok(zed::Command {
                    command: path,
                    args: vec!["--stdio".to_string()],
                    env: Default::default(),
                });
            }
            return Err(format!(
                "installed package '{}' did not contain expected path '{}'",
                PACKAGE_NAME, SERVER_PATH
            ));
        }

        // Zed runs the server from the project folder, so pass an absolute path.
        let server_path = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(SERVER_PATH);

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![server_path.to_string_lossy().into_owned(), "--stdio".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(SilverStripeExtension);
