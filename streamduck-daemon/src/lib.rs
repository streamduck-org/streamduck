//! Library that provides definitions for daemon related features in streamduck
pub mod daemon_data;

/// Name that is used for request pipe on Windows
pub const WINDOWS_REQUEST_PIPE_NAME: &'static str = "\\\\.\\pipe\\streamduck_requests";
/// Name that is used for event pipe on Windows
pub const WINDOWS_EVENT_PIPE_NAME: &'static str = "\\\\.\\pipe\\streamduck_events";

/// Path to unix domain socket on Unix
pub const UNIX_SOCKET_PATH: &'static str = "/tmp/streamduck.sock";