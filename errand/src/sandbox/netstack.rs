//! The userspace TCP/IP stack for the virtual network layer (T-090).
//!
//! One smoltcp stack terminates every session flow. It is fed IP frames by a
//! per-backend [`LinkDevice`]: a TUN over the T-020 unix socket in the Incus
//! backend, or the operator's cfrs `AF_UNIX` interposition in the lite backend.
//! The stack does not care which; only the link differs. This module is the
//! seam and the smoltcp adapter over it; wiring flows to the gate is a later
//! task of T-090.

use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;

/// The largest frame the virtual link carries. cfrs floors its MTU at 1280; a
/// link may offer more, and a backend overrides [`LinkDevice::mtu`] when it
/// knows its own.
pub(crate) const LINK_MTU: usize = 1500;

/// The transport that carries the stack's IP frames to and from the guest.
///
/// A backend implements it over whatever it has: a TUN read/write in the Incus
/// backend, a framed `AF_UNIX` stream in the lite backend. The frames are raw
/// IP packets ([`Medium::Ip`], no ethernet header).
pub(crate) trait LinkDevice {
    /// A frame that arrived from the guest, or `None` when none is waiting.
    fn recv_frame(&mut self) -> Option<Vec<u8>>;
    /// Hand a frame to the guest.
    fn send_frame(&mut self, frame: Vec<u8>);
    /// The largest frame the link carries.
    fn mtu(&self) -> usize {
        LINK_MTU
    }
}

/// Adapts a [`LinkDevice`] to smoltcp's [`phy::Device`], so one stack can run
/// over any backend's link.
pub(crate) struct Phy<L: LinkDevice> {
    link: L,
}

impl<L: LinkDevice> Phy<L> {
    /// Wraps a backend's link so a smoltcp stack can run over it.
    pub(crate) fn new(link: L) -> Self {
        Self { link }
    }
}

/// Holds a frame that arrived, handed to smoltcp to parse.
pub(crate) struct RxToken(Vec<u8>);

/// Borrows the link so smoltcp can fill one frame and have it sent.
pub(crate) struct TxToken<'a, L: LinkDevice>(&'a mut L);

impl phy::RxToken for RxToken {
    fn consume<R, F: FnOnce(&[u8]) -> R>(self, f: F) -> R {
        f(&self.0)
    }
}

impl<L: LinkDevice> phy::TxToken for TxToken<'_, L> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, len: usize, f: F) -> R {
        let mut frame = vec![0u8; len];
        let result = f(&mut frame);
        self.0.send_frame(frame);
        result
    }
}

impl<L: LinkDevice> Device for Phy<L> {
    type RxToken<'a>
        = RxToken
    where
        Self: 'a;
    type TxToken<'a>
        = TxToken<'a, L>
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // The received frame is owned by the RxToken, so only the TxToken
        // borrows the link; there is no aliasing of `self.link`.
        let frame = self.link.recv_frame()?;
        Some((RxToken(frame), TxToken(&mut self.link)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(TxToken(&mut self.link))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ip;
        caps.max_transmission_unit = self.link.mtu();
        caps
    }
}

#[cfg(test)]
mod tests;
