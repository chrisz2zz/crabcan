use nix::{
    mount::{mount, umount2, MntFlags, MsFlags},
    unistd::{chdir, pivot_root},
};

use crate::errors::Errcode;
use std::{
    fs::{create_dir_all, remove_dir},
    path::PathBuf,
};

pub fn setmountpoint(
    mount_dir: &PathBuf,
    addpahts: &Vec<(PathBuf, PathBuf)>,
) -> Result<(), Errcode> {
    log::debug!("Setting mount points ...");
    let new_root = PathBuf::from(format!("/tmp/crabcan.{}", random_string(12)));
    log::debug!(
        "Mounting temp directory {}",
        new_root.as_path().to_str().unwrap()
    );
    create_directory(&new_root)?;
    mount_directory(
        Some(&mount_dir),
        &new_root,
        vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE],
    )?;

    log::debug!("Mounting additional paths");
    for (inpath, mntpath) in addpahts.iter() {
        let outpath = new_root.join(mntpath);
        create_directory(&outpath)?;
        mount_directory(Some(inpath), &outpath, vec![MsFlags::MS_PRIVATE, MsFlags::MS_BIND])?;
    }

    log::debug!("Pivoting root");
    let old_root_tail = format!("oldroot.{}", random_string(6));
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));
    create_directory(&put_old)?;
    if let Err(_) = pivot_root(&new_root, &put_old) {
        return Err(Errcode::MountsError(4));
    }
    log::debug!("Unmounting old root");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));
    if let Err(_) = chdir(&PathBuf::from("/")) {
        return Err(Errcode::MountsError(5));
    }
    unmount_path(&old_root)?;
    delete_dir(&old_root)?;
    Ok(())
}

pub fn clean_mounts(_rootpath: &PathBuf) -> Result<(), Errcode> {
    Ok(())
}

pub fn mount_directory(
    path: Option<&PathBuf>,
    mount_point: &PathBuf,
    flags: Vec<MsFlags>,
) -> Result<(), Errcode> {
    let mut ms_flags = MsFlags::empty();
    for f in flags.iter() {
        ms_flags.insert(*f);
    }

    match mount::<PathBuf, PathBuf, PathBuf, PathBuf>(path, mount_point, None, ms_flags, None) {
        Ok(_) => Ok(()),
        Err(e) => {
            if let Some(p) = path {
                log::error!(
                    "Cannot mount {} to {}: {}",
                    p.to_str().unwrap(),
                    mount_point.to_str().unwrap(),
                    e
                );
            } else {
                log::error!("Cannot remount {}: {}", mount_point.to_str().unwrap(), e);
            }

            Err(Errcode::MountsError(3))
        }
    }
}

pub fn random_string(n: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| {
            let idx = rand::Rng::gen_range(&mut rng, 0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn create_directory(path: &PathBuf) -> Result<(), Errcode> {
    match create_dir_all(path) {
        Err(e) => {
            log::error!("Cannot create directory {}: {}", path.to_str().unwrap(), e);
            Err(Errcode::MountsError(2))
        }
        Ok(_) => Ok(()),
    }
}

pub fn unmount_path(path: &PathBuf) -> Result<(), Errcode> {
    match umount2(path, MntFlags::MNT_DETACH) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Unable to umount {}: {}", path.to_str().unwrap(), e);
            Err(Errcode::MountsError(0))
        }
    }
}

pub fn delete_dir(path: &PathBuf) -> Result<(), Errcode> {
    match remove_dir(path.as_path()) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!(
                "Unable to delete directory {}: {}",
                path.to_str().unwrap(),
                e
            );
            Err(Errcode::MountsError(1))
        }
    }
}
