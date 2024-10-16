use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};

pub fn redirect_stderr() -> io::Result<File> {
    unsafe {
        let stderr_fd = libc::STDERR_FILENO;
        let null_fd =
            libc::open(b"/dev/null\0".as_ptr() as *const _, libc::O_WRONLY);

        if null_fd == -1 {
            return Err(io::Error::last_os_error());
        }

        // Backup the original stderr file descriptor
        let backup_fd = libc::dup(stderr_fd);
        if backup_fd == -1 {
            libc::close(null_fd);
            return Err(io::Error::last_os_error());
        }

        // Redirect stderr to /dev/null
        libc::dup2(null_fd, stderr_fd);
        libc::close(null_fd);

        // Return the backup file descriptor as a File so it can be restored later
        Ok(File::from_raw_fd(backup_fd))
    }
}

pub fn restore_stderr(backup: File) {
    unsafe {
        let stderr_fd = libc::STDERR_FILENO;
        libc::dup2(backup.as_raw_fd(), stderr_fd);
    }
}

pub fn redirect_stdout() -> io::Result<File> {
    unsafe {
        let stdout_fd = libc::STDOUT_FILENO;
        let null_fd =
            libc::open(b"/dev/null\0".as_ptr() as *const _, libc::O_WRONLY);

        if null_fd == -1 {
            return Err(io::Error::last_os_error());
        }

        // Backup the original stdout file descriptor
        let backup_fd = libc::dup(stdout_fd);
        if backup_fd == -1 {
            libc::close(null_fd);
            return Err(io::Error::last_os_error());
        }

        // Redirect stdout to /dev/null
        libc::dup2(null_fd, stdout_fd);
        libc::close(null_fd);

        // Return the backup file descriptor as a File
        Ok(File::from_raw_fd(backup_fd))
    }
}

pub fn restore_stdout(backup: File) {
    unsafe {
        let stdout_fd = libc::STDOUT_FILENO;
        libc::dup2(backup.as_raw_fd(), stdout_fd);
    }
}

pub fn get_files_by_ext(directory: &str, extensions: &[&str]) -> Vec<String> {
    use std::fs;
    match fs::read_dir(directory) {
        Ok(entries) => entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        if extensions.contains(&ext.as_str()) {
                            return path.file_name().map(|name| {
                                name.to_string_lossy().into_owned()
                            });
                        }
                    }
                }
                None
            })
            .collect(),
        Err(_) => vec![],
    }
}
