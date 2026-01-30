use core_logic::fs::{FileSystem, VolumeMgr};
use embedded_sdmmc::VolumeManager;
use fatfs::{FatType, FormatVolumeOptions, format_volume};
use mbrman::{CHS, MBR};
use std::io::{Seek, Write};
use std::path::Path;
use std::sync::Mutex;
use std::{fs::File, io::SeekFrom};

use crate::fs_helper::{
    blockdevice::{Clock, LinuxBlockDevice},
    volume_mgr::VolumeMgrDuple,
};

pub const TORRENT_STRING: &'static [u8] = include_bytes!("../sample.torrent");

pub mod blockdevice;
pub mod volume_mgr;

// Mutex to ensure only one test creates the disk at a time
static DISK_INIT: Mutex<()> = Mutex::new(());

pub fn init_fs_duple() -> FileSystem<VolumeMgrDuple> {
    // Lock only during disk creation
    {
        let _lock = DISK_INIT.lock().unwrap();
        create_fat32_disk_with_files().unwrap();
    }

    FileSystem::new(VolumeMgrDuple::new(VolumeManager::new(
        LinuxBlockDevice::new("tests/disk.img", false).unwrap(),
        Clock,
    )))
}

/// sank you copilot <3
fn create_fat32_disk_with_files() -> std::io::Result<()> {
    let size_mb = 512;
    let path = "tests/disk.img";
    if Path::new(path).exists() {
        return Ok(());
    }
    let total_sectors = (size_mb * 1024 * 1024) / 512;

    // Create disk file
    let mut disk = File::create(path)?;
    disk.set_len((size_mb * 1024 * 1024) as u64)?;

    // Create MBR with one FAT32 partition
    let mut mbr = MBR::new_from(&mut disk, 512, [0xff; 4]).expect("Failed to create MBR");

    // Create partition starting at sector 2048 (standard alignment)
    let start_sector = 2048;
    let partition_sectors = total_sectors - start_sector;

    mbr[1] = mbrman::MBRPartitionEntry {
        boot: mbrman::BOOT_INACTIVE,
        first_chs: CHS::empty(),
        sys: 0x0C, // FAT32 LBA
        last_chs: CHS::empty(),
        starting_lba: start_sector,
        sectors: partition_sectors,
    };

    mbr.write_into(&mut disk).expect("Something weird occured");

    // Format the partition as FAT32
    disk.seek(SeekFrom::Start((start_sector * 512) as u64))?;

    let partition_size = (partition_sectors * 512) as u64;
    let mut partition = std::io::Cursor::new(vec![0u8; partition_size as usize]);

    format_volume(
        &mut partition,
        FormatVolumeOptions::new().fat_type(FatType::Fat32),
    )?;

    // Add files to the filesystem
    partition.seek(SeekFrom::Start(0))?;
    let fs = fatfs::FileSystem::new(&mut partition, fatfs::FsOptions::new())?;
    let root_dir = fs.root_dir();

    // Create directories
    root_dir.create_dir("torrents")?;

    // Create files
    let mut test_file = root_dir.create_file("test.txt")?;
    test_file.write_all(b"Hello from FAT32!")?;

    // Add file in subdirectory
    let torrents_dir = root_dir.open_dir("torrents")?;
    let mut torrent_file = torrents_dir.create_file("example.torrent")?;
    torrent_file.write_all(TORRENT_STRING)?;

    // Drop filesystem to flush changes
    drop(torrent_file);
    drop(torrents_dir);
    drop(test_file);
    drop(root_dir);
    drop(fs);

    // Write partition back to disk
    disk.write_all(partition.get_ref())?;

    Ok(())
}
