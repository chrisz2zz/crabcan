use std::os::fd::RawFd;
use std::path::PathBuf;

use crate::child::generate_child_process;
use crate::cli::Args;
use crate::config::ContainerOpts;
use crate::errors::Errcode;
use crate::mounts::clean_mounts;
use crate::namespace::handle_child_uid_map;
use crate::resources::{clean_cgroups, restrict_resources};
use nix::sys::utsname::uname;
use nix::unistd::Pid;
use scan_fmt::scan_fmt;
pub struct Container {
    sockets: (RawFd, RawFd),
    config: ContainerOpts,
    child_pid: Option<Pid>,
}

impl Container {
    pub fn new(args: Args) -> Result<Container, Errcode> {
        let mut addpaths = vec![];
        for ap_pair in args.addpaths.iter() {
            let mut pair = ap_pair.to_str().unwrap().split(":");
            let frompath = PathBuf::from(pair.next().unwrap())
                .canonicalize()
                .expect("Cannot canonicalize path")
                .to_path_buf();
            let mntpath = PathBuf::from(pair.next().unwrap())
                .strip_prefix("/")
                .expect("Cannot strip prefix from path")
                .to_path_buf();
            addpaths.push((frompath, mntpath));
        }

        let (config, sockets) =
            ContainerOpts::new(args.command, args.uid, args.mount_dir, addpaths)?;
        Ok(Container {
            sockets,
            config,
            child_pid: None,
        })
    }

    pub fn create(&mut self) -> Result<(), Errcode> {
        let pid = generate_child_process(self.config.clone())?;
        restrict_resources(&self.config.hostname, pid)?;
        handle_child_uid_map(pid, self.sockets.0)?;
        log::debug!("Creation finished");
        Ok(())
    }

    pub fn clean_exit(&mut self) -> Result<(), Errcode> {
        log::debug!("Cleaning container");
        clean_mounts(&self.config.mount_dir)?;
        if let Err(e) = clean_cgroups(&self.config.hostname) {
            log::error!("Cgroups cleaning failed: {}", e);
            return Err(e);
        }
        Ok(())
    }
}

pub const MINIMAL_KERNEL_VERSION: f32 = 4.8;

pub fn check_linux_version() -> Result<(), Errcode> {
    let host = uname();
    log::debug!("Linux release: {}", host.release());
    if let Ok(version) = scan_fmt!(host.release(), "{f}.{}", f32) {
        if version < MINIMAL_KERNEL_VERSION {
            return Err(Errcode::NotSupported(0));
        }
    } else {
        return Err(Errcode::ContainerError(0));
    }
    if host.machine() != "x86_64" {
        return Err(Errcode::NotSupported(1));
    }
    Ok(())
}

pub fn start(args: Args) -> Result<(), Errcode> {
    check_linux_version()?;
    let mut container = Container::new(args)?;
    if let Err(e) = container.create() {
        container.clean_exit()?;
        log::error!("Error while creating container: {:?}", e);
        return Err(e);
    }

    log::debug!("Finished, cleaning & exit");
    container.clean_exit()
}
