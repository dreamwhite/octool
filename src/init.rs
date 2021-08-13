use plist::Value;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::draw::Position;
use crate::res::{self, Resources};

pub fn init(
    config_plist: &PathBuf,
    resources: &mut Resources,
    position: &mut Position,
) -> Result<(), Box<dyn Error>> {
    resources.octool_config = res::get_serde_json("tool_config_files/octool_config.json")?;
    position.build_type = resources.octool_config["build_version"]
        .as_str()
        .unwrap()
        .to_string();
    println!(
        "\x1B[32mbuild_version set to\x1B[0m {}",
        position.build_type
    );
    position.resource_sections =
        serde_json::from_value(resources.octool_config["resource_sections"].clone()).unwrap();
    println!(
        "\x1B[32mplist resource sections\x1B[0m {:?}",
        position.resource_sections
    );

    println!("\n\x1B[32mchecking\x1B[0m acidanthera OpenCorePkg source");
    let path = Path::new(
        resources.octool_config["opencorepkg_path"]
            .as_str()
            .unwrap(),
    );
    let url = resources.octool_config["opencorepkg_url"].as_str().unwrap();
    let branch = resources.octool_config["opencorepkg_branch"]
        .as_str()
        .unwrap();
    res::clone_or_pull(url, path, branch)?;

    resources.config_plist = Value::from_file(&config_plist)
        .expect(format!("Didn't find valid plist at {:?}", config_plist).as_str());

    resources.acidanthera = res::get_serde_json("tool_config_files/acidanthera_config.json")?;

    println!("\n\x1B[32mchecking\x1B[0m dortania/build_repo/config.json");
    let path = Path::new(
        resources.octool_config["dortania_config_path"]
            .as_str()
            .unwrap(),
    );
    let url = resources.octool_config["dortania_config_url"]
        .as_str()
        .unwrap();
    let branch = resources.octool_config["dortania_config_branch"]
        .as_str()
        .unwrap();
    res::clone_or_pull(url, path, branch)?;

    resources.dortania =
        res::get_serde_json(path.parent().unwrap().join("config.json").to_str().unwrap())?;
    resources.parents = res::get_serde_json("tool_config_files/parents.json")?;

    println!();
    let path =
        res::get_or_update_local_parent("OpenCorePkg", &resources.dortania, &position.build_type)?;

    match path {
        Some(p) => resources.open_core_pkg = p.parent().unwrap().to_path_buf(),
        _ => panic!("no OpenCorePkg found"),
    }

    println!(
        "\n\x1B[32mValidating\x1B[0m {:?} with latest acidanthera/ocvalidate",
        config_plist
    );

    let out = res::status(
        resources
            .open_core_pkg
            .join("Utilities/ocvalidate/ocvalidate")
            .to_str()
            .unwrap(),
        &[&config_plist.to_str().unwrap()],
    )?;
    println!("{}", String::from_utf8(out.stdout).unwrap());
    if out.status.code().unwrap() != 0 {
        println!("\x1B[31mWARNING: Error(s) found in config.plist!\x1B[0m");
        println!("{}", String::from_utf8(out.stderr).unwrap());
    }

    position.file_name = config_plist.to_str().unwrap().to_owned();
    position.sec_length[0] = resources.config_plist.as_dictionary().unwrap().keys().len();

    Ok(())
}
