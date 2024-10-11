use std::ffi::CString;

use nix::sched::{clone, CloneFlags};
use nix::sys::signal::Signal;
use nix::unistd::execve;
use nix::unistd::{close, Pid};

use crate::capabilities::setcapabilities;
use crate::mounts::setmountpoint;
use crate::namespace::userns;
use crate::syscalls::setsyscalls;
use crate::{config::ContainerOpts, errors::Errcode, hostname::set_container_hostname};

const STACK_SIZE: usize = 1024 * 1024;

fn set_container_configurations(config: &ContainerOpts) -> Result<(), Errcode> {
    set_container_hostname(&config.hostname)?;
    setmountpoint(&config.mount_dir, &config.addpaths)?;
    userns(config.fd, config.uid)?;
    setcapabilities()?;
    setsyscalls()?;
    Ok(())
}

fn child(config: ContainerOpts) -> isize {
    match set_container_configurations(&config) {
        Ok(_) => log::info!("Container set up successfully"),
        Err(e) => {
            log::error!("Error while configuring container: {:?}", e);
            return -1;
        }
    }

    if let Err(_) = close(config.fd) {
        log::error!("Error while closing socket ...");
        return -1;
    }
    log::info!(
        "Starting container with command {} and args {:?}",
        config.path.to_str().unwrap(),
        config.argv
    );

    let recode = match execve::<CString, CString>(&config.path, &config.argv, &[]) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("Error while trying to perform execve: {:?}", e);
            -1
        }
    };
    recode
}

pub fn generate_child_process(config: ContainerOpts) -> Result<Pid, Errcode> {
    let mut tmp_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS);
    flags.insert(CloneFlags::CLONE_NEWCGROUP);
    flags.insert(CloneFlags::CLONE_NEWPID);
    flags.insert(CloneFlags::CLONE_NEWIPC);
    flags.insert(CloneFlags::CLONE_NEWNET);
    flags.insert(CloneFlags::CLONE_NEWUTS);

    match clone(
        Box::new(|| child(config.clone())),
        &mut tmp_stack,
        flags,
        Some(Signal::SIGCHLD as i32),
    ) {
        Ok(pid) => Ok(pid),
        Err(_) => Err(Errcode::ChildProcessError(0)),
    }
}
