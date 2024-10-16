use crate::errors::Errcode;
use crate::hostname::generate_hostname;
use crate::ipc::generate_socketpair;
use std::ffi::CString;
use std::os::fd::RawFd;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ContainerOpts {
    pub path: CString,
    pub argv: Vec<CString>,
    pub uid: u32,
    pub mount_dir: PathBuf,
    pub hostname: String,
    pub fd: RawFd,
    pub addpaths: Vec<(PathBuf, PathBuf)>,
}

impl ContainerOpts {
    pub fn new(
        command: String,
        uid: u32,
        mount_dir: PathBuf,
        addpaths: Vec<(PathBuf, PathBuf)>,
    ) -> Result<(ContainerOpts, (RawFd, RawFd)), Errcode> {
        let sockets = generate_socketpair()?;
        let argv: Vec<CString> = command
            .split_ascii_whitespace()
            .map(|s| CString::new(s).expect("Cannot read arg"))
            .collect();
        let path = argv[0].clone();
        Ok((
            ContainerOpts {
                path,
                argv,
                uid,
                mount_dir,
                hostname: generate_hostname()?,
                fd: sockets.1.clone(),
                addpaths,
            },
            sockets,
        ))
    }
}
