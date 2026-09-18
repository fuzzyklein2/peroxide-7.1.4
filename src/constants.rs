use std::env::current_exe;
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::utilities::program_name;

pub const ERROR_PICT: &str = "🛑  ";
pub const WARN_PICT: &str = "⚠️  ";
pub const INFO_PICT: &str = "💬  ";
pub const DEBUG_PICT: &str = "🐞  ";
pub const TRACE_PICT: &str = "🔍  ";
pub const CHECK_PICT: &str = "✅  ";
pub const FAILURE_PICT: &str = "❌  ";
pub const FOLDER_PICT: &str = "📁  ";
pub const WORLD_PICT: &str = "🌎";

pub static THIS_PROCESS: LazyLock<PathBuf>
    = LazyLock::new(|| current_exe().unwrap());

pub static BASE_DIR: LazyLock<PathBuf>
    = LazyLock::new(|| THIS_PROCESS.parent().unwrap()
                                   .parent().unwrap()
                                   .parent().unwrap()
                                   .to_path_buf());

pub const CONF_DIR_NAME: &str = ".config";
pub const CONF_FILE_NAME: &str = "config.json";
pub const LOG_DIR_NAME: &str = ".log";
pub const DATA_DIR_NAME: &str = "data";

pub static DATA_FILE_NAME: LazyLock<String>
    = LazyLock::new(|| (program_name().expect("REASON") + ".json"));

// pub const CONFIG_FILE: &str = concat!(
