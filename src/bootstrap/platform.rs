use core::ptr;
use remdb::platform::Platform;
use std::fs::OpenOptions;
use std::io::{Read, Seek, Write};
use std::time::SystemTime;

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn get_timestamp(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        now.as_millis() as u64
    }

    fn get_timestamp_us(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        now.as_micros() as u64
    }

    fn spin_lock(&self, lock: &mut u32) {
        while unsafe {
            core::sync::atomic::AtomicU32::from_ptr(lock as *mut u32)
                .compare_exchange(
                    0,
                    1,
                    core::sync::atomic::Ordering::Acquire,
                    core::sync::atomic::Ordering::Relaxed,
                )
                .is_err()
        } {
            core::hint::spin_loop();
        }
    }

    fn spin_unlock(&self, lock: &mut u32) {
        unsafe {
            core::sync::atomic::AtomicU32::from_ptr(lock as *mut u32)
                .store(0, core::sync::atomic::Ordering::Release);
        }
    }

    fn compiler_barrier(&self) {
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }

    fn full_memory_barrier(&self) {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }

    fn memcpy(&self, dest: *mut u8, src: *const u8, size: usize) {
        unsafe {
            ptr::copy_nonoverlapping(src, dest, size);
        }
    }

    fn memset(&self, dest: *mut u8, value: u8, size: usize) {
        unsafe {
            ptr::write_bytes(dest, value, size);
        }
    }

    fn delay_ms(&self, ms: u32) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }

    fn delay_us(&self, us: u32) {
        std::thread::sleep(std::time::Duration::from_micros(us as u64));
    }

    fn file_open(
        &self,
        path: &str,
        mode: remdb::platform::FileMode,
    ) -> remdb::platform::FileResult<remdb::platform::FileHandle> {
        let mut options = OpenOptions::new();
        match mode {
            remdb::platform::FileMode::Read => {
                options.read(true);
            }
            remdb::platform::FileMode::Write => {
                options.write(true).create(true).truncate(true);
            }
            remdb::platform::FileMode::ReadWrite => {
                options.read(true).write(true).create(true);
            }
            remdb::platform::FileMode::Append => {
                options.write(true).create(true).append(true);
            }
        }

        match options.open(path) {
            Ok(file) => {
                let boxed_file = Box::new(file);
                Ok(Box::into_raw(boxed_file) as remdb::platform::FileHandle)
            }
            Err(_) => Err(()),
        }
    }

    fn file_close(&self, handle: remdb::platform::FileHandle) -> remdb::platform::FileResult<()> {
        unsafe {
            let _ = Box::from_raw(handle as *mut std::fs::File);
        }
        Ok(())
    }

    fn file_write(
        &self,
        handle: remdb::platform::FileHandle,
        buffer: *const u8,
        size: usize,
    ) -> remdb::platform::FileResult<usize> {
        unsafe {
            let file = &mut *(handle as *mut std::fs::File);
            let slice = core::slice::from_raw_parts(buffer, size);
            match file.write(slice) {
                Ok(bytes_written) => {
                    file.flush().map_err(|_| ())?;
                    Ok(bytes_written)
                }
                Err(_) => Err(()),
            }
        }
    }

    fn file_read(
        &self,
        handle: remdb::platform::FileHandle,
        buffer: *mut u8,
        size: usize,
    ) -> remdb::platform::FileResult<usize> {
        unsafe {
            let file = &mut *(handle as *mut std::fs::File);
            let slice = core::slice::from_raw_parts_mut(buffer, size);
            match file.read(slice) {
                Ok(bytes_read) => Ok(bytes_read),
                Err(_) => Err(()),
            }
        }
    }

    fn file_seek(
        &self,
        handle: remdb::platform::FileHandle,
        offset: i64,
        whence: remdb::platform::SeekWhence,
    ) -> remdb::platform::FileResult<u64> {
        unsafe {
            let file = &mut *(handle as *mut std::fs::File);
            let seek_from = match whence {
                remdb::platform::SeekWhence::SeekSet => std::io::SeekFrom::Start(offset as u64),
                remdb::platform::SeekWhence::SeekCur => std::io::SeekFrom::Current(offset),
                remdb::platform::SeekWhence::SeekEnd => std::io::SeekFrom::End(offset),
            };
            match file.seek(seek_from) {
                Ok(new_pos) => Ok(new_pos),
                Err(_) => Err(()),
            }
        }
    }

    fn file_remove(&self, path: &str) -> remdb::platform::FileResult<()> {
        match std::fs::remove_file(path) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn file_size(&self, path: &str) -> remdb::platform::FileResult<usize> {
        use std::fs::metadata;
        match metadata(path) {
            Ok(metadata) => Ok(metadata.len() as usize),
            Err(_) => Err(()),
        }
    }

    fn crc32(&self, data: *const u8, size: usize) -> u32 {
        const CRC32_POLY: u32 = 0xEDB88320;
        let mut crc_table = [0u32; 256];
        for i in 0..256 {
            let mut crc = i as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ CRC32_POLY;
                } else {
                    crc >>= 1;
                }
            }
            crc_table[i] = crc;
        }
        let mut crc = 0xFFFFFFFFu32;
        let data_slice = unsafe { core::slice::from_raw_parts(data, size) };
        for &byte in data_slice {
            let index = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ crc_table[index];
        }
        crc ^ 0xFFFFFFFFu32
    }
}

static WINDOWS_PLATFORM: WindowsPlatform = WindowsPlatform;

pub fn init_platform() {
    remdb::platform::init_platform(&WINDOWS_PLATFORM);
}
