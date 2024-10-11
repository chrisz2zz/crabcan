use std::fmt::Display;
use std::process::exit;

#[derive(Debug)]
pub enum Errcode {
    ContainerError(u8),
    NotSupported(u8),
    ArgumentInvalid(&'static str),
    HostnameError(u8),
    RngError,
    MountsError(u8),
    NamespaceError(u8),
    SocketError(u8),
    ChildProcessError(u8),
    CapabilitiesError(u8),
    SyscallsError(u8),
    ResourcesError(u8),
}

impl Errcode {
    pub fn get_retcode(&self) -> i32 {
        1
    }
}

#[allow(unreachable_patterns)]
impl Display for Errcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errcode::ArgumentInvalid(msg) => write!(f, "ArgumentInvalid: {msg}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

pub fn exit_with_retcode(res: Result<(), Errcode>) {
    match res {
        Ok(_) => {
            log::debug!("Exit without any error, returning 0");
            exit(0);
        }
        Err(e) => {
            let retcode = e.get_retcode();
            log::error!("Error on exit:\n\t{}\n\tReturning {}", e, retcode);
            exit(retcode);
        }
    }
}
