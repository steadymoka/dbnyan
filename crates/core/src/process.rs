//! Subprocess helpers reused by anything that shells out long-running CLIs
//! (ssh tunnel, aws ssm, claude -p streaming).

use tokio::process::{Child, ChildStderr, ChildStdout};

/// Owns a `tokio::process::Child` and SIGTERM's its whole process group on
/// drop. Used to guarantee child + grandchild cleanup when an HTTP handler
/// stream is aborted (client disconnect, server shutdown, explicit cancel).
///
/// Requires the child to have been spawned via
/// [`spawn_in_new_process_group`](crate::spawn_in_new_process_group); without
/// it `pgid` is `Some` but `kill(-pgid)` targets the launcher's group, which
/// is Not What You Want.
pub struct ChildGuard {
    child: Child,
    pgid: Option<i32>,
}

impl ChildGuard {
    pub fn new(child: Child) -> Self {
        let pgid = child.id().map(|p| p as i32);
        Self { child, pgid }
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(pgid) = self.pgid {
            // SAFETY: libc::kill with a valid pgid and signal; no UB.
            unsafe {
                libc::kill(-pgid, libc::SIGTERM);
            }
        }
        let _ = self.child.start_kill();
    }
}
