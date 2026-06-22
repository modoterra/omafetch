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

pub fn mounted_disk_usages() -> Vec<MountUsage> {
    let Some(mounts) = read_to_string("/proc/mounts") else {
        return Vec::new();
    };
    let mut seen_devices = Vec::new();
    let mut usages = Vec::new();

    for line in mounts.lines() {
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

        if let Some(usage) = fs_usage(mount) {
            usages.push(MountUsage {
                mount: pretty_mount(mount),
                total_bytes: usage.total_bytes,
                available_bytes: usage.available_bytes,
            });
        }
    }

    usages
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
