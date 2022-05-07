use std::fs::File;

use xshell::{cmd, Shell};

fn main() -> anyhow::Result<()> {
    let sh = Shell::new()?;

    let platform_arch = get_build_platform_arch();
    dbg!(&platform_arch);

    let repo = "openvscode-server";
    let tag = "openvscode-server-v1.67.0";
    cmd!(sh, "git clone https://github.com/gitpod-io/{repo}").run()?;

    let out_dir = format!("vscode-reh-web-{platform_arch}");

    let in_repo_dir = sh.push_dir(repo);
    {
        cmd!(sh, "git checkout {tag}").run()?;

        cmd!(sh, "git reset --hard").run()?;

        cmd!(sh, "git apply ../portalbox-patch.patch").run()?;
    }

    let version = {
        let package_json_file = format!("{repo}/package.json");
        let package_json = File::open(&package_json_file)?;
        let package_json: serde_json::Value = serde_json::from_reader(package_json)?;

        let version = package_json["version"].clone();
        version
            .as_str()
            .map(|val| val.to_string())
            .ok_or(anyhow::anyhow!("No version found"))?
    };
    dbg!(&version);
    let output_name = {
        let platform_arch = get_out_platform_arch();
        format!("portalbox-vscode-{version}-{platform_arch}")
    };
    dbg!(&output_name);

    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            cmd!(sh, "powershell yarn").run()?;
            cmd!(
                sh,
                "powershell yarn gulp vscode-reh-web-{platform_arch}-min"
            )
            .run()?;
        } else {
            cmd!(sh, "yarn").run()?;
            cmd!(
                sh,
                "yarn gulp vscode-reh-web-{platform_arch}-min"
            )
            .run()?;
        }
    }

    drop(in_repo_dir);

    cmd!(sh, "mv {out_dir} {output_name}").run()?;
    cmd!(sh, "tar -czf {output_name}.tar.gz {output_name}").run()?;

    Ok(())
}

fn get_build_arch() -> String {
    std::env::var("BUILD_ARCH").unwrap()
}

fn get_build_platform_arch() -> String {
    #[cfg(target_os = "macos")]
    static PLATFORM: &str = "darwin";
    #[cfg(target_os = "linux")]
    static PLATFORM: &str = "linux";
    #[cfg(target_os = "windows")]
    static PLATFORM: &str = "win32";

    let arch = get_build_arch();

    format!("{PLATFORM}-{arch}")
}

fn get_out_platform_arch() -> String {
    let os = std::env::consts::OS;
    let arch = get_build_arch();

    format!("{os}-{arch}")
}
