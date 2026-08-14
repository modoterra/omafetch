use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct FsUsage {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MountUsage {
    pub mount: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

pub fn read_to_string(path: impl AsRef<Path>) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn count_dirs(path: impl AsRef<Path>) -> Option<usize> {
    let entries = std::fs::read_dir(path).ok()?;
    Some(
        entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .count(),
    )
}

pub fn fs_usage(path: impl AsRef<Path>) -> Option<FsUsage> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path = CString::new(path.as_ref().as_os_str().as_bytes()).ok()?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();

    // SAFETY: statvfs writes to the provided valid pointer and does not retain it.
    let result = unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) };
    if result != 0 {
        return None;
    }

    // SAFETY: statvfs returned success, so the structure has been initialized.
    let stat = unsafe { stat.assume_init() };
    let block_size = stat.f_frsize;

    Some(FsUsage {
        total_bytes: stat.f_blocks.saturating_mul(block_size),
        available_bytes: stat.f_bavail.saturating_mul(block_size),
    })
}

pub fn path_age_days(path: impl AsRef<Path>) -> Option<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path = CString::new(path.as_ref().as_os_str().as_bytes()).ok()?;
    let mut stat = std::mem::MaybeUninit::<libc::statx>::uninit();

    // SAFETY: statx writes to the provided valid pointer and does not retain it.
    let result = unsafe {
        libc::statx(
            libc::AT_FDCWD,
            path.as_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
            libc::STATX_BTIME,
            stat.as_mut_ptr(),
        )
    };
    if result != 0 {
        return None;
    }

    // SAFETY: statx returned success, so the structure has been initialized.
    let stat = unsafe { stat.assume_init() };
    if stat.stx_btime.tv_sec <= 0 {
        return None;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let created = stat.stx_btime.tv_sec as u64;

    Some(now.saturating_sub(created) / 86_400)
}

#[derive(Debug, PartialEq, Eq)]
struct DiskCandidate {
    device: String,
    mount: String,
    fs: String,
}

pub fn mounted_disk_usages() -> Vec<MountUsage> {
    let Some(mounts) = read_to_string("/proc/mounts") else {
        return Vec::new();
    };

    disk_candidates_from(&mounts)
        .into_iter()
        .filter_map(|candidate| {
            fs_usage(&candidate.mount).map(|usage| MountUsage {
                mount: pretty_mount(&candidate.mount),
                total_bytes: usage.total_bytes,
                available_bytes: usage.available_bytes,
            })
        })
        .collect()
}

fn disk_candidates_from(input: &str) -> Vec<DiskCandidate> {
    let mut seen_devices = Vec::new();
    let mut candidates = Vec::new();

    for line in input.lines() {
        let mut fields = line.split_whitespace();
        let Some(device) = fields.next() else {
            continue;
        };
        let Some(mount) = fields.next() else {
            continue;
        };
        let Some(fs) = fields.next() else {
            continue;
        };

        if !is_disk_fs(fs)
            || mount == "/boot"
            || !device.starts_with("/dev/")
            || seen_devices.contains(&device)
        {
            continue;
        }
        seen_devices.push(device);

        candidates.push(DiskCandidate {
            device: device.to_string(),
            mount: mount.to_string(),
            fs: fs.to_string(),
        });
    }

    candidates
}

fn is_disk_fs(fs: &str) -> bool {
    matches!(fs, "btrfs" | "ext4" | "xfs" | "f2fs" | "vfat" | "exfat")
}

fn pretty_mount(mount: &str) -> String {
    match mount {
        "/" => "root".to_string(),
        "/home" => "home".to_string(),
        "/boot" => "boot".to_string(),
        mount => mount
            .trim_start_matches("/mnt/")
            .trim_start_matches('/')
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_disk_candidates_from_mounts() {
        let input = concat!(
            "/dev/mapper/root / btrfs rw,relatime 0 0\n",
            "/dev/mapper/home /home ext4 rw,relatime 0 0\n",
            "/dev/nvme0n1p1 /boot vfat rw,relatime 0 0\n",
            "tmpfs /tmp tmpfs rw 0 0\n",
            "proc /proc proc rw 0 0\n",
            "/dev/sdb1 /mnt/data ext4 rw,relatime 0 0\n",
            "/dev/sdb1 /mnt/data-dup ext4 rw,relatime 0 0\n",
        );

        let candidates = disk_candidates_from(input);
        let summarized = candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.device.as_str(),
                    pretty_mount(&candidate.mount),
                    candidate.fs.as_str(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            summarized,
            vec![
                ("/dev/mapper/root", "root".to_string(), "btrfs"),
                ("/dev/mapper/home", "home".to_string(), "ext4"),
                ("/dev/sdb1", "data".to_string(), "ext4"),
            ]
        );
    }

    #[test]
    fn names_known_mounts() {
        assert_eq!(pretty_mount("/"), "root");
        assert_eq!(pretty_mount("/home"), "home");
        assert_eq!(pretty_mount("/boot"), "boot");
        assert_eq!(pretty_mount("/mnt/data"), "data");
    }
}
