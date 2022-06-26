//! Library that provides definitions for daemon related features in streamduck
pub mod daemon_data;

/// Name that is used for named pipe on Windows
pub const WINDOWS_PIPE_NAME: &'static str = "\\\\.\\pipe\\streamduck";

/// Path to unix domain socket on Unix
pub const UNIX_SOCKET_PATH: &'static str = "/tmp/streamduck.sock";