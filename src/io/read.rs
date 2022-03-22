use super::op::Op;
use super::EmmaBuf;
use std::os::unix::io::RawFd;

pub(crate) struct Read<'read, T: EmmaBuf> {
    /// currently raw fd
    fd: RawFd,
    /// buf reference
    buf: &'read mut T
}
