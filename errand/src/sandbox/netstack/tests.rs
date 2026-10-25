use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::socket::tcp;
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr};

use super::{LinkDevice, Phy};

/// A shared queue of frames travelling one direction.
type Wire = Rc<RefCell<VecDeque<Vec<u8>>>>;

/// An in-process link: it reads frames from one wire and writes to the other.
/// Two of them, cross-wired, stand in for the guest-to-host transport.
struct ChannelLink {
    inbound: Wire,
    outbound: Wire,
}

impl LinkDevice for ChannelLink {
    fn recv_frame(&mut self) -> Option<Vec<u8>> {
        self.inbound.borrow_mut().pop_front()
    }
    fn send_frame(&mut self, frame: Vec<u8>) {
        self.outbound.borrow_mut().push_back(frame);
    }
}

fn interface(ip: IpAddress, device: &mut Phy<ChannelLink>, at: Instant) -> Interface {
    let mut iface = Interface::new(Config::new(HardwareAddress::Ip), device, at);
    iface.update_ip_addrs(|addrs| {
        addrs.push(IpCidr::new(ip, 24)).unwrap();
    });
    iface
}

fn socket() -> tcp::Socket<'static> {
    tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0u8; 4096]),
        tcp::SocketBuffer::new(vec![0u8; 4096]),
    )
}

/// A TCP handshake and a two-way transfer complete over the in-process link, so
/// the smoltcp stack and the `LinkDevice` seam work end to end (T-090 task 2).
#[test]
fn a_tcp_connection_round_trips_over_an_in_process_link() {
    let a2b: Wire = Rc::new(RefCell::new(VecDeque::new()));
    let b2a: Wire = Rc::new(RefCell::new(VecDeque::new()));

    let mut client_dev = Phy::new(ChannelLink {
        inbound: Rc::clone(&b2a),
        outbound: Rc::clone(&a2b),
    });
    let mut server_dev = Phy::new(ChannelLink {
        inbound: Rc::clone(&a2b),
        outbound: Rc::clone(&b2a),
    });

    let client_ip = IpAddress::v4(10, 66, 0, 1);
    let server_ip = IpAddress::v4(10, 66, 0, 2);
    let mut clock = Instant::from_millis(0);

    let mut client = interface(client_ip, &mut client_dev, clock);
    let mut server = interface(server_ip, &mut server_dev, clock);

    let mut client_sockets = SocketSet::new(vec![]);
    let mut server_sockets = SocketSet::new(vec![]);
    let client_handle = client_sockets.add(socket());
    let server_handle = server_sockets.add(socket());

    server_sockets
        .get_mut::<tcp::Socket>(server_handle)
        .listen(7777)
        .unwrap();
    client_sockets
        .get_mut::<tcp::Socket>(client_handle)
        .connect(client.context(), (server_ip, 7777), 49000)
        .unwrap();

    let mut client_sent = false;
    let mut server_echoed = false;
    let mut got_back: Vec<u8> = Vec::new();

    for _ in 0..200 {
        client.poll(clock, &mut client_dev, &mut client_sockets);
        server.poll(clock, &mut server_dev, &mut server_sockets);

        let c = client_sockets.get_mut::<tcp::Socket>(client_handle);
        if !client_sent && c.can_send() {
            c.send_slice(b"ping").unwrap();
            client_sent = true;
        }
        if c.can_recv() {
            got_back = c.recv(|data| (data.len(), data.to_vec())).unwrap();
        }

        let s = server_sockets.get_mut::<tcp::Socket>(server_handle);
        if !server_echoed && s.can_recv() {
            let seen = s.recv(|data| (data.len(), data.to_vec())).unwrap();
            if seen == b"ping" && s.can_send() {
                s.send_slice(b"pong").unwrap();
                server_echoed = true;
            }
        }

        if got_back == b"pong" {
            break;
        }
        clock += Duration::from_millis(5);
    }

    assert!(client_sent, "the client never reached a sendable state");
    assert!(server_echoed, "the server never saw the ping");
    assert_eq!(got_back, b"pong", "the client never got the echo back");
}
