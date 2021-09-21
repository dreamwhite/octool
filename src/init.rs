use std::error::Error;
use std::io::{Stdout, Write};
use std::path::{Path, PathBuf};

use plist::Value;
use termion::raw::RawTerminal;

use crate::draw::Position;
use crate::res::{self, Resources};

pub fn init(
    config_plist: &PathBuf,
    resources: &mut Resources,
    position: &mut Position,
    stdout: &mut RawTerminal<Stdout>,
) -> Result<(), Box<dyn Error>> {
    resources.octool_config = res::get_serde_json("tool_config_files/octool_config.json", stdout)?;
    position.build_type = resources.octool_config["build_type"]
        .as_str()
        .unwrap_or("release")
        .to_string();
    write!(
        stdout,
        "\x1B[32mbuild_version set to\x1B[0m {}\r\n",
        position.build_type
    )?;
    let config_res_sections: Vec<(String, String, String, String)> =
        serde_json::from_value(resources.octool_config["resource_sections"].clone()).unwrap();
    for (mut sec, sub, _, _) in config_res_sections {
        sec.push_str(&sub);
        position.resource_sections.push(sec);
    }

    write!(
        stdout,
        "\n\x1B[32mchecking local\x1B[0m acidanthera OpenCorePkg source\r\n"
    )?;
    let path = Path::new(
        resources.octool_config["opencorepkg_path"]
            .as_str()
            .unwrap(),
    );
    let url = resources.octool_config["opencorepkg_url"].as_str().unwrap();
    let branch = resources.octool_config["opencorepkg_branch"]
        .as_str()
        .unwrap();
    res::clone_or_pull(url, path, branch, stdout)?;

    resources.config_plist = Value::from_file(&config_plist)
        .expect(format!("Didn't find valid plist at {:?}", config_plist).as_str());
    resources.sample_plist = Value::from_file(&"tool_config_files/OpenCorePkg/Docs/Sample.plist")
        .expect(format!("Didn't find Sample.plist at {:?}", config_plist).as_str());

    resources.acidanthera =
        res::get_serde_json("tool_config_files/acidanthera_config.json", stdout)?;

    write!(
        stdout,
        "\n\x1B[32mchecking local\x1B[0m dortania/build_repo/config.json\r\n"
    )?;
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
    res::clone_or_pull(url, path, branch, stdout)?;

    resources.dortania = res::get_serde_json(
        path.parent().unwrap().join("config.json").to_str().unwrap(),
        stdout,
    )?;
    resources.resource_list = res::get_serde_json("tool_config_files/resource_list.json", stdout)?;

    write!(stdout, "\r\n")?;
    let path = res::get_or_update_local_parent(
        "OpenCorePkg",
        &resources.dortania,
        &position.build_type,
        stdout,
    )?;

    match path {
        Some(p) => resources.open_core_pkg = p.parent().unwrap().to_path_buf(),
        _ => panic!("no OpenCorePkg found"),
    }

    write!(
        stdout,
        "\n\x1B[32mValidating\x1B[0m {:?} with latest acidanthera/ocvalidate\r\n",
        config_plist
    )?;
    validate_plist(&config_plist, &resources, stdout)?;

    position.config_file_name = config_plist.to_str().unwrap().to_owned();
    position.sec_length[0] = resources.config_plist.as_dictionary().unwrap().keys().len();
    let mut found_key = false;
    let keys: Vec<String> = resources
        .config_plist
        .as_dictionary()
        .unwrap()
        .keys()
        .map(|s| s.to_string())
        .collect();
    for (i, k) in keys.iter().enumerate() {
        if !found_key {
            if !k.starts_with('#') {
                position.sec_num[0] = i;
                found_key = true;
            }
        }
    }
    Ok(())
}

pub fn validate_plist(
    config_plist: &PathBuf,
    resources: &Resources,
    stdout: &mut RawTerminal<Stdout>,
) -> Result<bool, Box<dyn Error>> {
    let mut config_okay = true;
    let out = res::status(
        resources
            .open_core_pkg
            .join("Utilities/ocvalidate/ocvalidate")
            .to_str()
            .unwrap(),
        &[&config_plist.to_str().unwrap()],
    )?;
    stdout.suspend_raw_mode()?;
    write!(stdout, "{}\r\n", String::from_utf8(out.stdout).unwrap())?;
    stdout.activate_raw_mode()?;
    if out.status.code().unwrap() != 0 {
        config_okay = false;
        write!(
            stdout,
            "\x1B[31mERROR: Problems(s) found in config.plist!\x1B[0m\r\n"
        )?;
        write!(stdout, "{}\r\n", String::from_utf8(out.stderr).unwrap())?;
    }
    Ok(config_okay)
}
