use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Connection {
    #[serde(rename = "serial")]
    Serial(SerialConnection),
    #[serde(rename = "socket")]
    Socket(SocketConnection),
    #[serde(rename = "rpmsg")]
    Rpmsg(RpmsgConnection),
}

fn default_baud() -> u32 {
    115_200
}

#[derive(Debug, Deserialize)]
pub struct SerialConnection {
    pub path: String,
    #[serde(default = "default_baud")]
    pub baud: u32,
}

#[derive(Debug, Deserialize)]
pub struct SocketConnection {
    pub path: String,
}

fn default_settle() -> u64 {
    4
}

fn default_timeout() -> u64 {
    10
}

#[derive(Debug, Deserialize)]
pub struct RpmsgConnection {
    #[serde(default = "default_rpmsg_ctrl_path")]
    pub ctrl_path: String,
    pub channel_name: String,
    #[serde(default = "default_remoteproc_path")]
    pub remoteproc_state_path: String,
    #[serde(default = "default_settle")]
    pub settle: u64,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_rpmsg_ctrl_path() -> String {
    "/dev/rpmsg_ctrl-dsp_rproc@0".to_string()
}

fn default_remoteproc_path() -> String {
    "/sys/class/remoteproc/remoteproc0/state".to_string()
}
