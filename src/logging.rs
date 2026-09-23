use chrono::Local;
use std::io::{ Error, Write };
use std::fs::{ File };

use env_logger;
use log::LevelFilter;

use crate::utilities::rotate_log_files;
use crate::FILE_SYSTEM;
use crate::ARGUMENTS;

use crate::constants::{ ERROR_PICT, WARN_PICT, INFO_PICT, DEBUG_PICT, TRACE_PICT };

pub fn error(s: &str) -> std::io::Result<()> {
    let message = format!("{ERROR_PICT}{s}");
    log::error!("{}", message);
    // write(&FILE_SYSTEM.get().unwrap().log_file, &message);
    let mut f = File::options().create(true).append(true)
        .open(&FILE_SYSTEM.get().unwrap().log_file)?;
    writeln!(&mut f, "{}", message)?;
    Ok(())
}

pub fn warn(s: &str) -> std::io::Result<()>  {
    let message = format!("{WARN_PICT}{s}");
    log::warn!("{}", message);
    // write(&FILE_SYSTEM.get().unwrap().log_file, &message);
    let mut f = File::options().create(true).append(true)
        .open(&FILE_SYSTEM.get().unwrap().log_file)?;
    writeln!(&mut f, "{}", message)?;
    Ok(())
}

pub fn info(s: &str) -> std::io::Result<()>  {
    let message = format!("{INFO_PICT}{s}");
    log::info!("{}", message);
    // write(&FILE_SYSTEM.get().unwrap().log_file, &message);
    let mut f = File::options().create(true).append(true)
        .open(&FILE_SYSTEM.get().unwrap().log_file)?;
    writeln!(&mut f, "{}", message)?;
    Ok(())
}

pub fn debug(s: &str) -> std::io::Result<()>  {
    let message = format!("{DEBUG_PICT}{s}");
    log::debug!("{}", message);
    // write(&FILE_SYSTEM.get().unwrap().log_file, &message);
    let mut f = File::options().create(true).append(true)
        .open(&FILE_SYSTEM.get().unwrap().log_file)?;
    writeln!(&mut f, "{}", message)?;
    Ok(())
}

pub fn trace(s: &str) -> std::io::Result<()> {
    let message = format!("{TRACE_PICT}{s}");
    log::trace!("{}", message);
    // write(&FILE_SYSTEM.get().unwrap().log_file, &message);
    let mut f = File::options().create(true).append(true)
        .open(&FILE_SYSTEM.get().unwrap().log_file)?;
    writeln!(&mut f, "{}", message)?;
    Ok(())
}

pub fn log_file_name() -> String {
    format!("{}.log", Local::now().format("%Y%m%d_%H:%M:%S"))
}

pub fn init_log() -> Result<(), Error> {
    let level = if ARGUMENTS.get().unwrap().trace {
        LevelFilter::Trace
    } else if ARGUMENTS.get().unwrap().debug {
        LevelFilter::Debug
    } else if ARGUMENTS.get().unwrap().verbose {
        LevelFilter::Info
    } else if ARGUMENTS.get().unwrap().warnings {
        LevelFilter::Warn
    } else {
        LevelFilter::Error
    };

    // env_logger::init();
    env_logger::Builder::new().filter_level(level).format(|buf, record| {
        writeln!(buf, "{} {}", record.level(), record.args())
    }).init();


    File::create_new(FILE_SYSTEM.get().unwrap().log_file.clone())?;
        rotate_log_files();
    Ok(())
}
