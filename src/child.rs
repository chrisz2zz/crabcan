use crate::mounts::setmountpoint;
use crate::{config::ContainerOpts, errors::Errcode, hostname::set_container_hostname};

fn set_container_configurations(config: &ContainerOpts) -> Result<(), Errcode> {
    set_container_hostname(&config.hostname)?;
    setmountpoint(&config.mount_dir)?;

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
    log::info!(
        "Starting container with command {} and args {:?}",
        config.path.to_str().unwrap(),
        config.argv
    );
    0
}
