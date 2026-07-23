use std::ffi::CString;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::time::{Duration, Instant};

use crate::config::RpmsgConnection;
use crate::traits::from_config::FromConfig;

const RPMSG_NAME_SIZE: usize = 32;
const RPMSG_ADDR_ANY: u32 = 0xFFFFFFFF;

/// 40-byte struct matching `struct rpmsg_endpoint_info` from the kernel UAPI.
/// Used for `RPMSG_CREATE_EPT_IOCTL`.
#[repr(C)]
struct RpmsgEndpointInfo {
    name: [u8; RPMSG_NAME_SIZE],
    src: u32,
    dst: u32,
}

const _: () = assert!(
    std::mem::size_of::<RpmsgEndpointInfo>() == 40,
    "RpmsgEndpointInfo must be 40 bytes"
);

/// 36-byte struct matching Elegoo's `struct rpmsg_ept_info`.
/// Used for `RPMSG_DESTROY_EPT_IOCTL`.
#[repr(C)]
struct RpmsgEptId {
    name: [u8; RPMSG_NAME_SIZE],
    id: i32,
}

const _: () = assert!(
    std::mem::size_of::<RpmsgEptId>() == 36,
    "RpmsgEptId must be 36 bytes"
);

pub struct RpmsgEndpoint {
    data_fd: Option<std::fs::File>,
    ctrl_path: String,
    channel_name: String,
    remoteproc_state_path: Option<String>,
    settle: Duration,
    timeout: Duration,
    ept_id: i32,
    ept_path: String,
}

impl RpmsgEndpoint {
    pub fn new(
        ctrl_path: &str,
        channel_name: &str,
        remoteproc_state_path: Option<&str>,
        settle: Duration,
        timeout: Duration,
    ) -> anyhow::Result<Self> {
        let mut this = Self {
            data_fd: None,
            ctrl_path: ctrl_path.to_owned(),
            channel_name: channel_name.to_owned(),
            remoteproc_state_path: remoteproc_state_path.map(|s| s.to_owned()),
            settle,
            timeout,
            ept_id: -1,
            ept_path: String::new(),
        };
        this.connect()?;
        Ok(this)
    }

    pub fn ept_path(&self) -> &str {
        &self.ept_path
    }

    pub fn reconnect(&mut self) -> anyhow::Result<()> {
        self.teardown();
        self.connect()
    }

    fn teardown(&mut self) {
        // Destroy endpoint BEFORE closing the data fd — closing the data fd
        // first causes the kernel to clean up the endpoint, making the
        // subsequent DESTROY_EPT ioctl fail with ENXIO.
        if self.ept_id >= 0 {
            if let Err(e) = destroy_endpoint(&self.ctrl_path, self.ept_id) {
                warn!("Failed to destroy RPMSG endpoint {}: {e}", self.ept_id);
            }
            self.ept_id = -1;
        }
        self.data_fd = None;
    }

    fn connect(&mut self) -> anyhow::Result<()> {
        if let Some(state_path) = &self.remoteproc_state_path {
            if let Err(e) = restart_remoteproc(state_path, self.settle) {
                warn!("Failed to restart DSP via remoteproc: {e}");
            }
        }

        debug!("Waiting for RPMSG ctrl device: {}", self.ctrl_path);
        wait_for_path(&self.ctrl_path, self.timeout)?;
        debug!("RPMSG ctrl device found");

        let before = list_rpmsg_devices()?;

        // Retry endpoint creation — right after a DSP restart the ctrl
        // device node can appear before the remote firmware has finished
        // booting, so the ioctl may fail briefly.
        let create_deadline = Instant::now() + self.timeout;
        let ept_id = loop {
            match create_endpoint(&self.ctrl_path, &self.channel_name) {
                Ok(id) => break id,
                Err(e) if Instant::now() < create_deadline => {
                    debug!("RPMSG_CREATE_EPT not ready yet, retrying: {e}");
                    std::thread::sleep(Duration::from_millis(250));
                }
                Err(e) => return Err(e),
            }
        };
        debug!("RPMSG endpoint created (kernel id={ept_id})");

        // Use the kernel-assigned ID to open /dev/rpmsg{id} directly,
        // matching Elegoo's dsp_helper.h approach. Fall back to scanning
        // if the ID is unknown (kernel didn't write back).
        let ept_path = if ept_id >= 0 {
            let path = format!("/dev/rpmsg{ept_id}");
            wait_for_path(&path, self.timeout)?;
            path
        } else {
            debug!("Kernel did not return endpoint ID, scanning /dev");
            wait_for_new_rpmsg(&before, self.timeout)?
        };
        debug!("RPMSG data endpoint: {ept_path}");

        let data_fd = open_data_endpoint(&ept_path)?;

        self.data_fd = Some(data_fd);
        self.ept_id = ept_id;
        self.ept_path = ept_path;
        Ok(())
    }
}

impl crate::connections::Stream for RpmsgEndpoint {
    fn reconnect(&mut self) -> anyhow::Result<()> {
        RpmsgEndpoint::reconnect(self)
    }

    fn read_timeout(&self) -> Duration {
        self.timeout
    }
}

impl Drop for RpmsgEndpoint {
    fn drop(&mut self) {
        self.teardown();
    }
}

fn io_err_is_recoverable(e: &std::io::Error) -> bool {
    use std::io::ErrorKind::*;
    matches!(
        e.kind(),
        BrokenPipe | ConnectionReset | ConnectionAborted | NotConnected
    ) || e.raw_os_error() == Some(libc::ENOTTY)
}

impl Read for RpmsgEndpoint {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let fd = self.data_fd.as_mut().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "RPMSG not connected")
        })?;
        match poll_then_read(fd, self.timeout, buf) {
            Ok(n) => Ok(n),
            Err(e) if io_err_is_recoverable(&e) => {
                warn!("RPMSG read failed ({e}), reconnecting");
                self.reconnect()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                let fd = self.data_fd.as_mut().unwrap();
                poll_then_read(fd, self.timeout, buf)
            }
            Err(e) => Err(e),
        }
    }
}

fn write_with_timeout(
    fd: &mut std::fs::File,
    buf: &[u8],
    deadline: Instant,
) -> std::io::Result<usize> {
    let mut written = 0;
    let poll_timeout = Duration::from_millis(100);
    while written < buf.len() {
        let remaining = deadline.checked_duration_since(Instant::now())
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "RPMSG write timed out: kernel never consumed data",
            ))?;
        match fd.write(&buf[written..]) {
            Ok(n) => {
                written += n;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let _ = poll_ready(fd.as_raw_fd(), libc::POLLOUT, remaining.min(poll_timeout));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(written)
}

impl Write for RpmsgEndpoint {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.data_fd.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "RPMSG not connected",
            ));
        }
        let deadline = Instant::now() + self.timeout;
        let result = write_with_timeout(self.data_fd.as_mut().unwrap(), buf, deadline);
        match result {
            Ok(n) => Ok(n),
            Err(e) if io_err_is_recoverable(&e) => {
                warn!("RPMSG write failed ({e}), reconnecting");
                self.reconnect()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                let deadline = Instant::now() + self.timeout;
                write_with_timeout(self.data_fd.as_mut().unwrap(), buf, deadline)
            }
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(fd) = self.data_fd.as_mut() {
            fd.flush()?;
        }
        Ok(())
    }
}

/// Poll for POLLIN then read — the rpmsg chardev does report POLLIN.
fn poll_then_read(fd: &std::fs::File, timeout: Duration, buf: &mut [u8]) -> std::io::Result<usize> {
    poll_ready(fd.as_raw_fd(), libc::POLLIN, timeout)?;
    let mut fd = fd;
    fd.read(buf)
}

impl FromConfig for RpmsgEndpoint {
    type ConfigType = RpmsgConnection;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let timeout = Duration::from_secs(config.timeout);
        let settle = Duration::from_secs(config.settle);
        Self::new(
            &config.ctrl_path,
            &config.channel_name,
            Some(&config.remoteproc_state_path),
            settle,
            timeout,
        )
    }
}

// --- Low-level helpers ---

fn wait_for_path(path: &str, timeout: Duration) -> anyhow::Result<()> {
    let deadline = Instant::now() + timeout;
    while !fs::metadata(path).is_ok() {
        if Instant::now() >= deadline {
            anyhow::bail!("Timed out waiting for {path}");
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Ok(())
}

fn list_rpmsg_devices() -> anyhow::Result<Vec<String>> {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("rpmsg") && name.chars().skip(5).all(|c| c.is_ascii_digit()) {
                devices.push(entry.path().display().to_string());
            }
        }
    }
    devices.sort();
    Ok(devices)
}

/// Create an RPMSG endpoint via `RPMSG_CREATE_EPT_IOCTL` on the ctrl device.
///
/// Returns the kernel-assigned endpoint ID (written back by the driver into
/// `info.src` at offset 32, which overlaps with Elegoo's `rpmsg_ept_info.id`).
fn create_endpoint(ctrl_path: &str, channel_name: &str) -> anyhow::Result<i32> {
    let ctrl_fd = unsafe {
        let path = CString::new(ctrl_path)?;
        libc::open(path.as_ptr(), libc::O_RDWR)
    };
    if ctrl_fd < 0 {
        anyhow::bail!(
            "Failed to open RPMSG ctrl device {}: {}",
            ctrl_path,
            std::io::Error::last_os_error()
        );
    }

    let mut info = RpmsgEndpointInfo {
        name: [0u8; RPMSG_NAME_SIZE],
        src: RPMSG_ADDR_ANY,
        dst: RPMSG_ADDR_ANY,
    };

    let name_bytes = channel_name.as_bytes();
    let len = name_bytes.len().min(RPMSG_NAME_SIZE - 1);
    info.name[..len].copy_from_slice(&name_bytes[..len]);

    debug!(
        "ioctl RPMSG_CREATE_EPT: fd={}, struct size={}, name={channel_name}",
        ctrl_fd,
        std::mem::size_of::<RpmsgEndpointInfo>(),
    );

    let ret = unsafe {
        libc::ioctl(
            ctrl_fd,
            libc::_IOW::<RpmsgEndpointInfo>(0xb5, 0x1),
            &mut info,
        )
    };

    let err = if ret < 0 {
        let e = std::io::Error::last_os_error();
        unsafe { libc::close(ctrl_fd) };
        anyhow::bail!("ioctl RPMSG_CREATE_EPT failed: {e}");
    } else {
        unsafe { libc::close(ctrl_fd) };
        // The Elegoo-patched kernel writes the assigned endpoint address
        // back into info.src (offset 32 = rpmsg_ept_info.id). Read it.
        let assigned_id = info.src as i32;
        debug!("Kernel assigned endpoint id={assigned_id}");
        Ok(assigned_id)
    };

    err
}

fn destroy_endpoint(ctrl_path: &str, ept_id: i32) -> anyhow::Result<()> {
    let ctrl_fd = unsafe {
        let path = CString::new(ctrl_path)?;
        libc::open(path.as_ptr(), libc::O_RDWR)
    };
    if ctrl_fd < 0 {
        anyhow::bail!(
            "Failed to open RPMSG ctrl device for destroy: {}",
            std::io::Error::last_os_error()
        );
    }

    // Match Elegoo's rpmsg_free_ept(): pass rpmsg_ept_info with the endpoint id.
    let info = RpmsgEptId {
        name: [0u8; RPMSG_NAME_SIZE],
        id: ept_id,
    };
    let ret = unsafe { libc::ioctl(ctrl_fd, libc::_IO(0xb5, 0x2), &info) };
    unsafe {
        libc::close(ctrl_fd);
    }
    if ret < 0 {
        let err = std::io::Error::last_os_error();
        warn!("ioctl RPMSG_DESTROY_EPT failed: {err}");
    }

    Ok(())
}

fn wait_for_new_rpmsg(before: &[String], timeout: Duration) -> anyhow::Result<String> {
    let deadline = Instant::now() + timeout;
    loop {
        let current = list_rpmsg_devices()?;
        for dev in &current {
            if !before.contains(dev) {
                return Ok(dev.clone());
            }
        }
        if Instant::now() >= deadline {
            anyhow::bail!("Timed out waiting for new /dev/rpmsgN device");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Open the rpmsg endpoint data device in non-blocking mode.
///
/// Elegoo's serialqueue also sets O_NONBLOCK via `fd_set_non_blocking()`.
/// We need non-blocking so that writes return EAGAIN instead of hanging
/// indefinitely when the kernel's virtio TX buffer is full or the DSP
/// isn't consuming messages.  Our `Write::write()` impl handles EAGAIN
/// with a retry loop bounded by a deadline (see `write_with_timeout`).
fn open_data_endpoint(path: &str) -> anyhow::Result<std::fs::File> {
    let fd = unsafe {
        let c_path = CString::new(path)?;
        libc::open(c_path.as_ptr(), libc::O_RDWR)
    };
    if fd < 0 {
        anyhow::bail!(
            "Failed to open RPMSG data endpoint {path}: {}",
            std::io::Error::last_os_error()
        );
    }
    // Set non-blocking — writes return EAGAIN instead of blocking.
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }
    Ok(unsafe { std::fs::File::from_raw_fd(fd) })
}

fn poll_ready(fd: RawFd, events: i16, timeout: Duration) -> std::io::Result<()> {
    let mut pollfd = libc::pollfd {
        fd,
        events,
        revents: 0,
    };
    let ms = timeout.as_millis() as i32;
    let ret = unsafe { libc::poll(&mut pollfd, 1, ms) };
    if ret < 0 {
        return Err(std::io::Error::last_os_error());
    }
    if ret == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            format!("RPMSG poll timed out after {timeout:?}"),
        ));
    }
    if pollfd.revents & libc::POLLERR != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "POLLERR on RPMSG device",
        ));
    }
    Ok(())
}

fn restart_remoteproc(state_path: &str, settle: Duration) -> anyhow::Result<()> {
    debug!("Restarting DSP via remoteproc: {state_path}");

    if let Err(e) = fs::write(state_path, "stop") {
        warn!("Failed to write 'stop' to {state_path} (likely already stopped): {e}");
    }
    std::thread::sleep(Duration::from_secs(1));

    fs::write(state_path, "start")
        .map_err(|e| anyhow::anyhow!("Failed to write 'start' to {state_path}: {e}"))?;
    std::thread::sleep(settle);

    debug!("DSP restarted, waiting for settle");
    Ok(())
}


