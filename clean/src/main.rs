use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use path_absolutize::Absolutize;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use subprocess::{Exec, Redirection};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct BuildAppInfoV1 {
    #[serde(rename = "component")]
    pub components: Vec<RawComponentManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct RawComponentManifest {
    pub id: String,
    pub build: Option<RawBuildConfig>,
    pub clean: Option<RawCleanConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct RawBuildConfig {
    pub command: String,
    pub workdir: Option<PathBuf>,
    pub watch: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct RawCleanConfig {
    pub command: String,
    pub workdir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let manifest_file = current_dir.as_path().join("spin-clean.toml");
    let manifest_text = tokio::fs::read_to_string(&manifest_file)
        .await
        .with_context(|| format!("Cannot read manifest file from {}", manifest_file.display()))?;
    let app: BuildAppInfoV1 = toml::from_str(&manifest_text)?;
    let app_dir = parent_dir(manifest_file)?;
    let component_ids: Vec<String> = vec![];
    let components_to_clean = if component_ids.is_empty() {
        app.components
    } else {
        let all_ids: HashSet<_> = app.components.iter().map(|c| &c.id).collect();
        let unknown_component_ids: Vec<_> = component_ids
            .iter()
            .filter(|id| !all_ids.contains(id))
            .map(|s| s.as_str())
            .collect();

        if !unknown_component_ids.is_empty() {
            bail!("Unknown component(s) {}", unknown_component_ids.join(", "));
        }

        app.components
            .into_iter()
            .filter(|c| component_ids.contains(&c.id))
            .collect()
    };

    if components_to_clean.iter().all(|c| c.clean.is_none()) {
        println!("None of the components have a build clean_command.");
        return Ok(());
    }

    components_to_clean
        .into_iter()
        .map(|c| clean_component(c, &app_dir))
        .collect::<Result<Vec<_>, _>>()?;

    // TODO: Reimplement this the hard way.
    // terminal::step!("Finished", "cleaning all Spin components");
    println!("Finished - cleaning all Spin components");

    Ok(())
}

/// Run the clean command of the component.
fn clean_component(raw: RawComponentManifest, app_dir: &Path) -> Result<()> {
    match raw.clean {
        Some(b) => {
            println!("Cleaning - component {} with `{}`", raw.id, b.command);
            let workdir = construct_workdir(app_dir, b.workdir.as_ref())?;
            let exit_status = Exec::shell(&b.command)
                .cwd(workdir)
                .stdout(Redirection::None)
                .stderr(Redirection::None)
                .stdin(Redirection::None)
                .popen()
                .map_err(|err| {
                    anyhow!(
                        "Cannot spawn build process '{:?}' for component {}: {}",
                        &b.command,
                        raw.id,
                        err
                    )
                })?
                .wait()?;

            if !exit_status.success() {
                bail!(
                    "Build command for component {} failed with status {:?}",
                    raw.id,
                    exit_status,
                );
            }

            Ok(())
        }
        _ => Ok(()),
    }
}

/// Constructs the absolute working directory in which to run the build command.
fn construct_workdir(app_dir: &Path, workdir: Option<impl AsRef<Path>>) -> Result<PathBuf> {
    let mut cwd = app_dir.to_owned();

    if let Some(workdir) = workdir {
        // Using `Path::has_root` as `is_relative` and `is_absolute` have
        // surprising behavior on Windows, see:
        // https://doc.rust-lang.org/std/path/struct.Path.html#method.is_absolute
        if workdir.as_ref().has_root() {
            bail!("The workdir specified in the application file must be relative.");
        }
        cwd.push(workdir);
    }

    Ok(cwd)
}

pub fn parent_dir(file: impl AsRef<Path>) -> Result<PathBuf> {
    let path_buf = file.as_ref().parent().ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to get containing directory for file '{}'",
            file.as_ref().display()
        )
    })?;

    absolutize(path_buf)
}

/// Returns absolute path to the file
pub fn absolutize(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();

    Ok(path
        .absolutize()
        .with_context(|| format!("Failed to resolve absolute path to: {}", path.display()))?
        .to_path_buf())
}