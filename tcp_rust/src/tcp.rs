use std::collections::{BTreeMap, VecDeque};
use std::{io, time};
use std::io::Write;
use bitflags::bitflags;
use etherparse::IpNumber;

enum State {
//    Listen,
    SynRcvd,
    Estab,
    TimeWait,
    FinWait1,
    FinWait2,
}

struct Timers {
    send_times: BTreeMap<u32, time::Instant>,
    srtt: f64,
}

pub struct Connection {
    state: State,
    send: SendSeqBlock,
    recv: RecvSeqBlock,
    ip: etherparse::Ipv4Header,
    tcp: etherparse::TcpHeader,
    timers: Timers,

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
//            let buf =[0u8;1500];
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
                    wnd,
                    up: 0,

                    wl1: 0,
                    wl2: 0,
                },
                recv: RecvSeqBlock {
                    irs: tcph.sequence_number(),
                    nxt: tcph.sequence_number() + 1,
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
                timers: Timers {
                    send_times: Default::default(),
                    srtt: time::Duration::from_secs(1 * 60).as_secs_f64(),
                },
                incoming: Default::default(),
                unacked: Default::default(),
                closed: false,
                closed_at: None,
                
            };
            c.tcp.syn = true;
            c.tcp.ack = true;
            c.write(nic,c.send.nxt,0)?;
            Ok(Some(c))
        }

    pub fn write(&mut self,
        nic: &mut tun_tap::Iface,
        seq: u32,
        mut limit: usize) -> io::Result<usize> {
            let mut buf = [0u8; 1500];
            //setup sequence
            // self.tcp.sequence_number = self.send.nxt;
            self.tcp.sequence_number = seq;
            self.tcp.acknowledgment_number = self.recv.nxt;
            //if !self.tcp.syn && !self.tcp.fin {
            //    self.tcp.psh = true;
            //}

            // TODO: return +1 for SYN/FIN
            println!(
                "write(ack: {}, seq: {}, limit: {}) syn {:?} fin {:?}",
                self.recv.nxt - self.recv.irs, seq, limit,self.tcp.syn, self.tcp.fin,
            );

            let mut offset = seq.wrapping_sub(self.send.una) as usize;
            // we need to special-case the two "virtual" bytes SYN and FIN
            println!("FIN close {:?}", self.closed_at);
            if let Some(closed_at) = self.closed_at {
                if seq == closed_at.wrapping_add(1) {
                    // trying to write following FIN
                    offset = 0;
                    limit = 0;
                }
            }
            println!(
                "using offset {} base {} in {:?}",
                offset,
                self.send.una,
                self.unacked.as_slices()
            );
            let (mut h, mut t) = self.unacked.as_slices();
            if h.len() >= offset {
                h = &h[offset..];
            } else {
                let skipped = h.len();
                h = &[];
                t = &t[(offset - skipped)..];
            }

            let max_data = std::cmp::min(limit, h.len() + t.len());
            let size = std::cmp::min(
                buf.len(),
                self.tcp.header_len() + self.ip.header_len() + max_data,
            );

            let _ = self.ip.set_payload_len(size - self.ip.header_len());

            // write out the headers and the payload
            let buf_len = buf.len();
            let mut unwritten = &mut buf[..];
            //            println!("unwritten size:{}",unwritten.len());
            //ip.write_all()方法会自动更新offset
            let _ = self.ip.write(&mut unwritten);
            //println!("unwritten size after write :{}",unwritten.len());
            let ip_header_ends_at = buf_len - unwritten.len();

            // postpone writing the tcp header because we need the payload as one contiguous slice to calculate the tcp checksum
            unwritten = &mut unwritten[self.tcp.header_len() as usize..];
            let tcp_header_ends_at = buf_len - unwritten.len();
            
            // write out the payload          
            let payload_bytes = {
                let mut written = 0;
                let mut limit = max_data;

                // first, write as much as we can from h
                let p1l = std::cmp::min(limit, h.len());
                written += unwritten.write(&h[..p1l])?;
                limit -= written;

                // then, write more (if we can) from t
                let p2l = std::cmp::min(limit, t.len());
                written += unwritten.write(&t[..p2l])?;
                written
            };

            let payload_ends_at = buf_len - unwritten.len();
             // finally we can calculate the tcp checksum and write out the tcp header
            self.tcp.checksum = self
                .tcp
                .calc_checksum_ipv4(&self.ip, &buf[tcp_header_ends_at..payload_ends_at])

                .expect("failed to compute checksum");

            let mut tcp_header_buf = &mut buf[ip_header_ends_at..tcp_header_ends_at];
            let _ = self.tcp.write(&mut tcp_header_buf);
            
            let mut next_seq = seq.wrapping_add(payload_bytes as u32);
            if self.tcp.syn {
                next_seq = next_seq.wrapping_add(1);
                self.tcp.syn = false;
            }
            if self.tcp.fin {
                next_seq = next_seq.wrapping_add(1);
                self.tcp.fin = false;
            }

            if Self::wrapping_lt(self.send.nxt, next_seq) {
                self.send.nxt = next_seq;
            }
            self.timers.send_times.insert(seq, time::Instant::now());

            nic.send(&buf[..payload_ends_at])?;
            Ok(payload_bytes)
        }

    pub(crate) fn on_tick(&mut self, nic: &mut tun_tap::Iface) -> io::Result<()> {
        if let State::FinWait2 | State::TimeWait = self.state {
            // we have shutdown our write side and the other side acked, no need to (re)transmit anything
            return Ok(());
        }

        // eprintln!("ON TICK: state {:?} una {} nxt {} unacked {:?}",
        //           self.state, self.send.una, self.send.nxt, self.unacked);

        // if has closed use that seq number
        let nunacked_data = self.closed_at.unwrap_or(self.send.nxt).wrapping_sub(self.send.una);
        let nunsent_data = self.unacked.len() as u32 - nunacked_data;


        let waited_for = self
            .timers
            .send_times
            .range(self.send.una..)
            .next()
            .map(|t| t.1.elapsed());

        // 等待时间大于1秒或者大于1.5倍 SRTT
        let should_retransmit = if let Some(waited_for) = waited_for {
            waited_for > time::Duration::from_secs(1)
                && waited_for.as_secs_f64() > 1.5 * self.timers.srtt
        } else {
            false
        };

        if should_retransmit {
            let resend = std::cmp::min(self.unacked.len() as u32, self.send.wnd as u32);
            if resend < self.send.wnd as u32 && self.closed {
                // can we include the FIN?
                self.tcp.fin = true;
                self.closed_at = Some(self.send.una.wrapping_add(self.unacked.len() as u32));
            }
            self.write(nic, self.send.una, resend as usize)?;
        } else {
            // we should send new data if we have new data and space in the window
            if nunsent_data == 0 && self.closed_at.is_some() {
                return Ok(());
            }

            let allowed = self.send.wnd as u32 - nunacked_data;
            if allowed == 0 {
                return Ok(());
            }

            let send = std::cmp::min(nunsent_data, allowed);
            if send < allowed && self.closed && self.closed_at.is_none() {
                self.tcp.fin = true;
                self.closed_at = Some(self.send.una.wrapping_add(self.unacked.len() as u32));
            }

            self.write(nic, self.send.nxt, send as usize)?;
        }

        // if FIN, enter FIN-WAIT-1
        Ok(())
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
                eprintln!("NOT OKAY");
                self.write(nic,self.send.nxt,0)?;
                return Ok(self.availability());
            }
            // move to handle in estab/fin_wait state
            //self.recv.nxt = seqn.wrapping_add(slen);

            if !tcph.ack() {
                // got SYN part of initial handshake
                assert!(data.is_empty());
                self.recv.nxt = seqn.wrapping_add(1);
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
                if Self::is_between_wrapped(self.send.una, ackn, self.send.nxt.wrapping_add(1)) {
                    println!(
                        "ack for {} (last: {}); prune in {:?}",
                        ackn, self.send.una, self.unacked
                    );
                    if ! self.unacked.is_empty() {
                        let data_start = if self.send.una == self.send.iss {
                            // send.una hasn't been updated yet with ACK for our SYN, so data starts just beyond it
                            self.send.una.wrapping_add(1)
                        } else {
                            self.send.una
                        };
                        let acked_data_end = std::cmp::min(ackn.wrapping_sub(data_start) as usize, self.unacked.len());
                        self.unacked.drain(..acked_data_end);
                        
                        let old = std::mem::replace(&mut self.timers.send_times,BTreeMap::new());
                        let una = self.send.una;
                        let mut srtt =& mut self.timers.srtt;
                        self.timers.send_times.extend(old.into_iter().filter_map(|(seq,sent)|{
                            if Self::is_between_wrapped(una, seq, ackn) {
                                *srtt = 0.8 * *srtt + (1.0 - 0.8) * sent.elapsed().as_secs_f64();
                                None
                            } else {
                                Some((seq, sent))
                            }
                        }));
                        
                    }
                    self.send.una = ackn;
                }

                // TODO: prune self.unacked
                // TODO: if unacked empty and waiting flush, notify
                // TODO: update window

            }

            if let Some(closed_at) = self.closed_at {
                if self.send.una == closed_at.wrapping_add(1) {
                    // our FIN has been ACKed!
                    self.state = State::FinWait2;
                }

            }

            if !data.is_empty() {
                if let State::Estab | State::FinWait1 | State::FinWait2 = self.state {
                    let mut unread_data_at = (self.recv.nxt.wrapping_sub(seqn)) as usize;
                    if unread_data_at > data.len() {
                        // we must have received a re-transmitted FIN that we have already seen
                        // nxt points to beyond the fin, but the fin is not in data!
                        assert_eq!(unread_data_at, data.len() + 1);
                        unread_data_at = 0;
                    }

//                    eprintln!("read data {} of {:?}",unread_data_at,data);
                    self.incoming.extend(&data[unread_data_at..]);

                    // Once the TCP takes responsibility for the data it advances
                    // RCV.NXT over the data accepted, and adjusts RCV.WND as
                    // apporopriate to the current buffer availability.  The total of
                    // RCV.NXT and RCV.WND should not be reduced.
                    self.recv.nxt = seqn.wrapping_add(data.len() as u32);
                    // Send an acknowledgment of the form: <SEQ=SND.NXT><ACK=RCV.NXT><CTL=ACK>
                    self.write(nic, self.send.nxt,0)?;
                }
            }
            
            if tcph.fin() {
                match self.state {
                    State::FinWait2 => {
                        // we're done with the connection!
                        self.recv.nxt = self.recv.nxt.wrapping_add(1);
                        self.write(nic,self.send.nxt,0)?;
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

    fn wrapping_lt(lhs: u32, rhs: u32) -> bool {
    // From RFC1323:
    //     TCP determines if a data segment is "old" or "new" by testing
    //     whether its sequence number is within 2**31 bytes of the left edge
    //     of the window, and if it is not, discarding the data as "old".  To
    //     insure that new data is never mistakenly considered old and vice-
    //     versa, the left edge of the sender's window has to be at most
    //     2**31 away from the right edge of the receiver's window.
    lhs.wrapping_sub(rhs) > (1 << 31)
}

    fn is_between_wrapped(start: u32, x: u32, end: u32) -> bool {
        Self::wrapping_lt(start, x) && Self::wrapping_lt(x, end)
    }
    fn is_between_wrapped_old(start: u32, x: u32, end: u32) -> bool {
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
