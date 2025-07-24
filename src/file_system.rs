use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use windows::{
    core::*, Win32::Foundation::FALSE, Win32::Storage::FileSystem as FS,
    Win32::System::WindowsProgramming as WP,
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Drive {
    pub name: String,
    pub letter: char,
    pub total_space: u64,
    pub used_space: u64,
    pub free_space: u64,
}

pub fn get_all_drives() -> Vec<Drive> {
    let mut drives = Vec::new();

    // Find out how big a buffer we need
    let buffer_size = unsafe { FS::GetLogicalDriveStringsA(None) };
    if buffer_size == 0 {
        // TODO: err
        todo!();
    }

    // TODO: Make this box expression.
    //       https://stackoverflow.com/questions/25805174/creating-a-fixed-size-array-on-heap-in-rust
    let mut drive_strings = vec![0u8; buffer_size as usize].into_boxed_slice();

    // Fetch all drive strings
    if unsafe { FS::GetLogicalDriveStringsA(Some(drive_strings.as_mut())) } == 0 {
        // TODO: err
        todo!();
    }

    // Loop until we find the final '\0'
    // drive_strings is a double null terminated list of null terminated strings)
    let mut single_drive_string = PCSTR::from_raw(drive_strings.as_ptr());
    while unsafe { single_drive_string.as_bytes() }.len() != 0 {
        let drive_type = unsafe { FS::GetDriveTypeA(single_drive_string) };

        let mut total_space: u64 = 0;
        let mut free_space: u64 = 0;
        if unsafe {
            FS::GetDiskFreeSpaceExA(
                single_drive_string,
                None,
                Some(&mut total_space),
                Some(&mut free_space),
            )
        } == FALSE
        {
            // TODO: err getlasterror
            todo!();
        }

        let drive_type_string = match drive_type {
            WP::DRIVE_REMOVABLE => "Removable",
            WP::DRIVE_FIXED => "Hard disk",
            WP::DRIVE_REMOTE => "Network",
            WP::DRIVE_CDROM => "CD/DVD",
            _ => "Unknown",
        };

        println!(
            "{} - {} - {} GiB free of {} GiB",
            unsafe { single_drive_string.display() },
            drive_type_string,
            free_space / 1024 / 1024 / 1024,
            total_space / 1024 / 1024 / 1024
        );

        drives.push(Drive {
            name: unsafe { single_drive_string.to_string().unwrap() },
            letter: unsafe { single_drive_string.0.read() as char },
            total_space,
            used_space: total_space - free_space,
            free_space,
        });

        // Move to next drive string
        // +1 is to move past the null at the end of the string.
        single_drive_string = PCSTR::from_raw(
            (single_drive_string.as_ptr() as usize + unsafe { strlen(single_drive_string) } + 1)
                as *const u8,
        );
    }

    drives
}

pub fn get_files_in_folder(path: &Path) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    for entry in fs::read_dir(path)? {
        file_names.push(entry?.file_name().to_string_lossy().to_string());
    }

    Ok(file_names)
}

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn open_file(path: &Path) -> windows::core::Result<()> {
    let mut quoted_path = OsString::from("\"");
    quoted_path.push(path);
    quoted_path.push("\"");

    let mut command = Command::new("cmd");
    command
        .args(["/c", "start"])
        .raw_arg("\"\"")
        .raw_arg(quoted_path)
        .creation_flags(CREATE_NO_WINDOW)
        .current_dir(path.parent().unwrap());

    command.spawn().expect("command failed to start");

    Ok(())
}

fn find_file_recursively(name: &Path, directory_path: &Path, result: &mut Vec<PathBuf>) {
    let entry_iterator = match fs::read_dir(directory_path) {
        Ok(entry_iterator) => entry_iterator,
        Err(error) => {
            // Skipping search for this directory
            println!(
                "Failed to read files from {:?}: {}",
                directory_path.display(),
                error
            );
            return;
        }
    };

    for entry in entry_iterator {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                // Skipping this entry
                println!(
                    "Failed to read a file in dir {:?}: {}",
                    directory_path, error
                );
                return;
            }
        };
        let entry_path = entry.path();

        if entry.metadata().unwrap().is_dir() {
            find_file_recursively(name, &entry_path, result);
        }

        if entry
            .file_name()
            .to_str()
            .unwrap()
            .contains(name.to_str().unwrap())
        {
            println!("Found {:?}", entry_path);
            result.push(entry_path)
        }
    }
}

pub fn find_file(name: &Path, directory_path: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();

    let start_time = Instant::now();
    find_file_recursively(name, directory_path, &mut result);
    let elapsed_time = start_time.elapsed();

    println!("Search took {} ms to complete", elapsed_time.as_millis());

    result
}
