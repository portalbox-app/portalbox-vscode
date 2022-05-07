use std::{
    env,
    path::{Path, PathBuf},
};

use xshell::{cmd, Shell};

fn main() -> Result<(), anyhow::Error> {
    let task = env::args()
        .nth(1)
        .ok_or(anyhow::anyhow!("No sub command"))?;
    match task.as_str() {
        "release" => release()?,
        "unrelease" => unrelease()?,
        _ => return Err(anyhow::anyhow!("Unexpected sub command")),
    }

    Ok(())
}

fn release() -> Result<(), anyhow::Error> {
    let version = version()?;
    println!("Making a release version = {version}");

    let project_dir = project_root();

    let sh = Shell::new()?;
    sh.change_dir(&project_dir);
    let tag_msg = format!("Version {version}");

    cmd!(sh, "git tag -a v{version} -m {tag_msg}").run()?;
    cmd!(sh, "git push --tags").run()?;

    Ok(())
}

fn unrelease() -> Result<(), anyhow::Error> {
    let version = version()?;
    println!("Removing a release version = {version}");

    let project_dir = project_root();

    let sh = Shell::new()?;
    sh.change_dir(&project_dir);

    cmd!(sh, "git tag -d v{version}").run()?;
    cmd!(sh, "git push --delete origin v{version}").run()?;

    Ok(())
}

fn version() -> Result<String, anyhow::Error> {
    let project_dir = project_root();
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(cargo_toml_path)?;
    let value: toml::Value = toml::from_str(&cargo_toml_content)?;

    let version = value["package"]["version"].clone();
    let ret = version
        .as_str()
        .map(|val| val.to_string())
        .ok_or(anyhow::anyhow!("No version found"))?;

    Ok(ret)
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}
