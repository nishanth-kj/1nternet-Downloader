use crate::config::{IPC_BUFFER_SIZE, IPC_RETRY_COUNT, IPC_RETRY_DELAY};
use crate::service::ipc::IpcService;
use tracing::{error, info};

pub const MUTEX_NAME: &str = "InternetDownloader_SingleInstance_Mutex";
pub const PIPE_NAME: &str = r"\\.\pipe\InternetDownloader_IPC";

pub fn decode_idr_url(raw: &str) -> Option<String> {
    let clean = raw.trim();
    if let Some(rest) = clean.strip_prefix("idr://") {
        urlencoding::decode(rest).ok().map(|s| s.into_owned())
    } else {
        Some(clean.to_string())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsIpcService;

impl IpcService for WindowsIpcService {
    fn decode_idr_url(&self, raw: &str) -> Option<String> {
        decode_idr_url(raw)
    }

    fn another_instance_is_running(&self) -> bool {
        windows::another_instance_is_running()
    }

    fn send_url_to_running_instance(&self, url: &str) -> bool {
        windows::send_url_to_running_instance(url)
    }
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::atomic::{AtomicIsize, Ordering};
    use tokio::io::AsyncReadExt;
    use tokio::net::windows::named_pipe::ServerOptions;
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    static SINGLE_INSTANCE_HANDLE: AtomicIsize = AtomicIsize::new(0);

    pub fn another_instance_is_running() -> bool {
        let wide: Vec<u16> = OsStr::new(super::MUTEX_NAME)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let handle = CreateMutexW(std::ptr::null(), 0, wide.as_ptr());
            if handle.is_null() {
                return false;
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                true
            } else {
                SINGLE_INSTANCE_HANDLE.store(handle as isize, Ordering::SeqCst);
                false
            }
        }
    }

    pub fn send_url_to_running_instance(url: &str) -> bool {
        use std::io::Write;
        for _ in 0..IPC_RETRY_COUNT {
            if let Ok(mut pipe) = std::fs::OpenOptions::new().write(true).open(super::PIPE_NAME) {
                let _ = pipe.write_all(url.as_bytes());
                let _ = pipe.flush();
                return true;
            }
            std::thread::sleep(IPC_RETRY_DELAY);
        }
        false
    }

    pub fn start_server<F>(callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to build tokio runtime for IPC: {}", e);
                    return;
                }
            };

            rt.block_on(async move {
                let mut server = match ServerOptions::new().first_pipe_instance(true).create(super::PIPE_NAME) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to create IPC pipe server: {}", e);
                        return;
                    }
                };

                loop {
                    if let Ok(()) = server.connect().await {
                        let mut buf = Vec::new();
                        let mut temp = [0u8; IPC_BUFFER_SIZE];
                        loop {
                            match server.read(&mut temp).await {
                                Ok(0) => break,
                                Ok(n) => buf.extend_from_slice(&temp[..n]),
                                Err(_) => break,
                            }
                        }

                        if let Ok(msg) = String::from_utf8(buf) {
                            if !msg.is_empty() {
                                info!("IPC server received URL: {}", msg);
                                callback(msg);
                            }
                        }
                    }

                    match ServerOptions::new().create(super::PIPE_NAME) {
                        Ok(next) => server = next,
                        Err(e) => {
                            error!("Failed to re-create IPC pipe server: {}", e);
                            break;
                        }
                    }
                }
            });
        });
    }
}

#[cfg(not(windows))]
pub mod windows {
    pub fn another_instance_is_running() -> bool { false }
    pub fn send_url_to_running_instance(_url: &str) -> bool { false }
    pub fn start_server<F>(_callback: F) where F: Fn(String) + Send + Sync + 'static {}
}

pub use windows::*;
