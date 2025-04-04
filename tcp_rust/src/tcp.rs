use std::collections::VecDeque;
use std::io;
use std::io::Write;
use bitflags::bitflags;

use etherparse::IpNumber;

enum State {
    Listen,
    SynRcvd,
    Estab,
    TimeWait,
    FinWait1,
    FinWait2,
}

pub struct Connection {
    state: State,
    send: SendSeqBlock,
    recv: RecvSeqBlock,
    ip: etherparse::Ipv4Header,
    tcp: etherparse::TcpHeader,

    pub(crate) incoming: VecDeque<u8>,
    pub(crate) unacked: VecDeque<u8>,
    pub(crate) closed: bool,
    closed_at: Option<u32>,
}


bitflags! {
    pub(crate) struct Available: u8 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
    }
}


impl Connection {
    pub(crate) fn is_rcv_closed(&self) -> bool {
        if let State::TimeWait = self.state {
            // TODO: any state after rcvd FIN, so also CLOSE-WAIT, LAST-ACK, CLOSED, CLOSING
            true
        } else {
            false
        }
    }

     fn availability(&self) -> Available {
        let mut a = Available::empty();
        if self.is_rcv_closed() || !self.incoming.is_empty() {
            a |= Available::READ;
        }
        // TODO: take into account self.state
        // TODO: set Available::WRITE
        a
    }
}

//    0                   1                   2                   3
//    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//   |          Source Port          |       Destination Port        |
//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//   |                        Sequence Number                        |
//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//   |                    Acknowledgment Number                      |
//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//   |  Data |           |U|A|P|R|S|F|                               |
//   | Offset| Reserved  |R|C|S|S|Y|I|            Window             |
//   |       |           |G|K|H|T|N|N|                               |
//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//   |           Checksum            |         Urgent Pointer        |
//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//   |                    Options                    |    Padding    |
//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//   |                             data                              |
//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

//                            TCP Header Format


//   TCB impl suggestion from RPC procotol
//   The maintenance of a TCP
//   connection requires the remembering of several variables.  We conceive
//   of these variables being stored in a connection record called a
//   Transmission Control Block or TCB.  Among the variables stored in the TCB are：
//
//   1 the local and remote socket numbers
//   2 the security and precedence of the connection
//   3 pointers to the user's send and receive buffers
//   4 pointers to the retransmit queue and to the current segment.
//   5 In addition several variables relating to the send and receive sequence numbers


//    Send Sequence Variables
//       SND.UNA - send unacknowledged
//       SND.NXT - send next
//       SND.WND - send window
//       SND.UP  - send urgent pointer
//       SND.WL1 - segment sequence number used for last window update
//       SND.WL2 - segment acknowledgment number used for last window
//                 update
//       ISS     - initial send sequence number

/// State of the Send Sequence Space (RFC 793 S3.2 F4)
///
/// ```
///            1         2          3          4
///       ----------|----------|----------|----------
///              SND.UNA    SND.NXT    SND.UNA
///                                   +SND.WND
///
/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers of unacknowledged data
/// 3 - sequence numbers allowed for new data transmission
/// 4 - future sequence numbers which are not yet allowed
/// ```
struct SendSeqBlock {
    /// send unacknowledged
    una: u32,
    /// send next
    nxt: u32,
    /// send window
    wnd: u16,
    /// send urgent pointer
    up: u16,
    /// segment sequence number used for last window update
    wl1: usize,
    /// segment acknowledgment number used for last window update
    wl2: usize,
    /// initial send sequence number
    iss: u32,
}
//     Receive Sequence Variables

//       RCV.NXT - receive next
//       RCV.WND - receive window
//       RCV.UP  - receive urgent pointer
//       IRS     - initial receive sequence number


/// State of the Receive Sequence Space (RFC 793 S3.2 F5)
///
/// ```
///                1          2          3
///            ----------|----------|----------
///                   RCV.NXT    RCV.NXT
///                             +RCV.WND
///
/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers allowed for new reception
/// 3 - future sequence numbers which are not yet allowed
/// ```
struct RecvSeqBlock {
    /// receive next
    nxt: u32,
    /// receive window
    wnd: u16,
    /// receive urgent pointer
    up: u16,
    /// initial receive sequence number
    irs: u32,
}
impl Connection {
    pub fn accept(nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8]) -> io::Result<Option<Self>> {
            // process the SYN and send ACK/SYN
            let mut buf =[0u8;1500];
            if !tcph.syn() {
                return Ok(None);
            }

            let iss = 0; // actual iss need be some random value, here just use 0
            let wnd = 1024;
            let mut c = Connection {
                state: State::SynRcvd,
                send: SendSeqBlock {
                    iss,
                    una: iss,
                    nxt: iss,
                    wnd: wnd,
                    up: 0,

                    wl1: 0,
                    wl2: 0,
                },
                recv: RecvSeqBlock {
                    irs: tcph.sequence_number(),
                    nxt: tcph.sequence_number() +1,
                    wnd: tcph.window_size(),
                    up: 0,
                },
                tcp: etherparse::TcpHeader::new(tcph.destination_port(), tcph.source_port(), iss, wnd),
                ip: etherparse::Ipv4Header::new(0, 64, IpNumber::TCP,
                    [
                        iph.destination()[0],
                        iph.destination()[1],
                        iph.destination()[2],
                        iph.destination()[3],
                    ],
                    [
                        iph.source()[0],
                        iph.source()[1],
                        iph.source()[2],
                        iph.source()[3],
                    ]).unwrap(),
                incoming: Default::default(),
                unacked: Default::default(),
                closed: false,
                closed_at: None,
                
            };
            c.tcp.syn = true;
            c.tcp.ack = true;
            c.write(nic, &[])?;
            Ok(Some(c))
        }

    pub fn write(&mut self,
        nic: &mut tun_tap::Iface,
        payload: &[u8]) -> io::Result<usize> {
            let mut buf = [0u8; 1500];
            //setup sequence
            self.tcp.sequence_number = self.send.nxt;
            self.tcp.acknowledgment_number = self.recv.nxt;

            let size = std::cmp::min(
                buf.len(),
                self.tcp.header_len() + self.ip.header_len() + payload.len(),
            );

            let _ = self.ip.set_payload_len(size - self.ip.header_len());

            self.tcp.checksum = self.tcp.calc_checksum_ipv4(&self.ip, &[])
                .expect("fail to calc tcp checksum");

            // write header
            let mut unwritten = &mut buf[..];
            let _ = self.ip.write(&mut unwritten);
            let _ = self.tcp.write(&mut unwritten);
            let payload_bytes = unwritten.write(payload)?;
            let unwritten = unwritten.len();
            self.send.nxt = self.send.nxt.wrapping_add(payload_bytes as u32);
            if self.tcp.syn {
                self.send.nxt = self.send.nxt.wrapping_add(1);
                self.tcp.syn = false;
            }
            if self.tcp.fin {
                self.send.nxt = self.send.nxt.wrapping_add(1);
                self.tcp.fin = false;
            }
            nic.send(&buf[..buf.len() - unwritten])?;
            Ok(payload_bytes)
        }
    
    pub fn on_packet(&mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8]) -> io::Result<Available> {
            // first, check that sequence numbers are valid (RFC 793 S3.3)
            let seqn = tcph.sequence_number();
            let mut slen = data.len() as u32;
            // make it diff from zero length segment
            // Due to zero  windows and zero length segments 
            if tcph.fin() || tcph.syn() {
                slen += 1;
            }
            
            let wend = self.recv.nxt.wrapping_add(self.recv.wnd as u32);

            let okay = if slen == 0 {
                // zero-length segment has separate rules for acceptance
                if self.recv.wnd == 0 {
                    if seqn != self.recv.nxt {
                        false
                    } else {
                        true
                    }
                } else if !Self::is_between_wrapped(self.recv.nxt.wrapping_sub(1), seqn, wend) {
                    false
                } else {
                    true
                }
            } else {
                if self.recv.wnd == 0 {
                    false
                } else if !Self::is_between_wrapped(self.recv.nxt.wrapping_sub(1), seqn, wend)
                    && !Self::is_between_wrapped(
                        self.recv.nxt.wrapping_sub(1),
                        seqn.wrapping_add(slen - 1),
                        wend,
                    ) // check both window start and end is valid
            {
                false
            } else {
                true
            }
            };

            // seq check not valid    
            if !okay {
                self.write(nic, &[])?;
                return Ok(self.availability());
            }
            self.recv.nxt = seqn.wrapping_add(slen);

            if !tcph.ack() {
                return Ok(self.availability());
            }

            let ackn = tcph.acknowledgment_number();
            if let State::SynRcvd = self.state {
                if Self::is_between_wrapped(
                    self.send.una.wrapping_sub(1),
                    ackn,
                    self.send.nxt.wrapping_add(1),
                ) {
                    // must have ACKed our SYN, since we detected at least one acked byte,
                    // and we have only sent one byte (the SYN).
                    self.state = State::Estab;
                } else {
                    // TODO: <SEQ=SEG.ACK><CTL=RST>
                }
            }

            if let State::Estab | State::FinWait1 | State::FinWait2 = self.state {
                if !Self::is_between_wrapped(self.send.una, ackn, self.send.nxt.wrapping_add(1)) {
                    return Ok(self.availability());
                }
                self.send.una = ackn;
                // TODO
                assert!(data.is_empty());

                if let State::Estab = self.state {
                    // now let's terminate the connection!
                    // TODO: needs to be stored in the retransmission queue!
                    self.tcp.fin = true;
                    self.write(nic, &[])?;
                    self.state = State::FinWait1;
                }
            }

            if let State::FinWait1 = self.state {
                if self.send.una == self.send.iss + 2 {
                    // our FIN has been ACKed!
                    self.state = State::FinWait2;
                }
            }

            if tcph.fin() {
                match self.state {
                    State::FinWait2 => {
                        // we're done with the connection!
                        self.write(nic, &[])?;
                        self.state = State::TimeWait;
                    }
                    _ => unimplemented!(),
                }
            }
            Ok(self.availability())
        }

    pub(crate) fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        match self.state {
            State::SynRcvd | State::Estab => {
                self.state = State::FinWait1;
            }
            State::FinWait1 | State::FinWait2 => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "already closing",
                ))
            }
        };
        Ok(())
    }
    
    fn is_between_wrapped(start: u32, x: u32, end: u32) -> bool {
        use std::cmp::Ordering;
        match start.cmp(&x) {
            Ordering::Equal => return false,
            Ordering::Less => {
                // we have:
                //
                //   0 |-------------S------X---------------------| (wraparound)
                //
                // X is between S and E (S < X < E) in these cases:
                //
                //   0 |-------------S------X---E-----------------| (wraparound)
                //
                //   0 |----------E--S------X---------------------| (wraparound)
                //
                // but *not* in these cases
                //
                //   0 |-------------S--E---X---------------------| (wraparound)
                //
                //   0 |-------------|------X---------------------| (wraparound)
                //                   ^-S+E
                //
                //   0 |-------------S------|---------------------| (wraparound)
                //                      X+E-^
                //
                // or, in other words, iff !(S <= E <= X)
                if end >= start && end <= x {
                    return false;
                }
            }
            Ordering::Greater => {
                // we have the opposite of above:
                //
                //   0 |-------------X------S---------------------| (wraparound)
                //
                // X is between S and E (S < X < E) *only* in this case:
                //
                //   0 |-------------X--E---S---------------------| (wraparound)
                //
                // but *not* in these cases
                //
                //   0 |-------------X------S---E-----------------| (wraparound)
                //
                //   0 |----------E--X------S---------------------| (wraparound)
                //
                //   0 |-------------|------S---------------------| (wraparound)
                //                   ^-X+E
                //
                //   0 |-------------X------|---------------------| (wraparound)
                //                      S+E-^
                //
                // or, in other words, iff S < E < X
                if end < start && end > x {
                } else {
                    return false;
                }
            }
        }
        true
    }
}
