use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::io::prelude::*;
use std::{io, thread};
use std::sync::{Arc, Condvar, Mutex};
use std::net::Ipv4Addr;
use etherparse::IpNumber;
use std::os::unix::io::AsRawFd;

// impl design:
// 1. a seperate thread hold  the nic to read
// 2. each connection hold the whole connection manager pointer and use condvar to notify
// 3. use VecDeq as the circular buffer to read/write the incoming and outgoing data
//    - use extend() to add to buffer
//    - copy_from_slice() from the head and tail
// 4. on_tick() to process the retransmission packet, use srtt to determin if need retrans

mod tcp;

const SENDQUEUE_SIZE: usize = 1024;

#[derive(Debug,Clone,Copy,Eq,Hash,PartialEq)]
struct Quad {
    src: (Ipv4Addr,u16),
    dst: (Ipv4Addr,u16),
}

#[derive(Default)]
struct TcpHandle {
    manager: Mutex<ConnectionManager>,
    pending_var: Condvar,
    rcv_var: Condvar,
}

#[derive(Default)]
struct ConnectionManager {
    terminate: bool,
    connections: HashMap<Quad, tcp::Connection>,
    pending: HashMap<u16, VecDeque<Quad>>,
}

type InterfaceHandle = Arc<TcpHandle>;

pub struct TcpStream {
    quad: Quad,
    ih: InterfaceHandle,
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        let cm = self.ih.manager.lock().unwrap();
        // TODO: send FIN on cm.connections[quad]
        // TODO: _eventually_ remove self.quad from cm.connections
    }
}


impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut cm = self.ih.manager.lock().unwrap();
        loop {
            let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "stream was terminated unexpectedly",)
            })?;

            if c.is_rcv_closed() && c.incoming.is_empty() {
                // no more data to read, and no need to block, because there won't be any more
                return Ok(0);
            }

            if !c.incoming.is_empty() {
                let mut nread = 0;
                let (head, tail) = c.incoming.as_slices();
                let hread = std::cmp::min(buf.len(), head.len());
                buf[..hread].copy_from_slice(&head[..hread]);
                nread += hread;
                let tread = std::cmp::min(buf.len() - nread, tail.len());
                buf[hread..(hread + tread)].copy_from_slice(&tail[..tread]);
                nread += tread;
                drop(c.incoming.drain(..nread));
                return Ok(nread);
            }

            cm = self.ih.rcv_var.wait(cm).unwrap();
        }
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut cm = self.ih.manager.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "stream was terminated unexpectedly",
            )
        })?;

        if c.unacked.len() >= SENDQUEUE_SIZE {
            // TODO: block
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "too many bytes buffered",
            ));
        }

        let nwrite = std::cmp::min(buf.len(), SENDQUEUE_SIZE - c.unacked.len());
        c.unacked.extend(buf[..nwrite].iter());

        Ok(nwrite)
    }
    fn flush(&mut self) -> io::Result<()> {
        let mut cm = self.ih.manager.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "stream was terminated unexpectedly",
            )
        })?;

        if c.unacked.is_empty() {
            Ok(())
        } else {
            // TODO: block
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "too many bytes buffered",
            ))
        }
    }
}

impl TcpStream {
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        let mut cm = self.ih.manager.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "stream was terminated unexpectedly",
            )
        })?;

        c.close()
    }
}
pub struct Interface {
    ih: Option<InterfaceHandle>,
    jh: Option<thread::JoinHandle<io::Result<()>>>,
}

fn packet_loop(mut nic: tun_tap::Iface, ih: InterfaceHandle) -> io::Result<()> {
    let mut buf = [0u8;1504];
    loop {
        let mut pfd =[nix::poll::PollFd::new(
            nic.as_raw_fd(),
            nix::poll::EventFlags::POLLIN,
        )];
        let n = nix::poll::poll(&mut pfd[..],10).map_err(|e| e.as_errno().unwrap())?;
        assert_ne!(n, -1);
        if n == 0 {
            let mut cmg = ih.manager.lock().unwrap();
            for connection in cmg.connections.values_mut() {
                // XXX: don't die on errors?
                connection.on_tick(&mut nic)?;
            }
            continue;
        }
        assert_eq!(n, 1);
        // &mut buf[..] 显示传递数组切片
        let nbytes = nic.recv(&mut buf[..])?;
        // netwotk endian is big endian

        // tun iface /without_packet_info/new/:
        
        // let _eth_flags = u16::from_be_bytes([buf[0],buf[1]]);
        // let eth_proto = u16::from_be_bytes([buf[2],buf[3]]);
        // if eth_proto != 0x800 {
        //     //not ipv4
        //     continue;
        // }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                eprintln!("protocol: {:x}", iph.protocol().0);
                
                if iph.protocol() != IpNumber::TCP {
                    //not tcp                        
                    eprintln!("BAD PROTOCOL");       
                    continue;                        
                }                                    
                

                match etherparse::TcpHeaderSlice::from_slice(&buf[iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        eprintln!("{} -> {} {}byte of tcp to port {}",
                            src,dst,tcph.slice().len(),tcph.destination_port());
                        let data = iph.slice().len() + tcph.slice().len();
                        let mut cmg = ih.manager.lock().unwrap();
                        let cm = &mut *cmg;
                        let q = Quad {
                            src: (src, tcph.source_port()),
                            dst: (dst, tcph.destination_port()),
                        };

                        match cm.connections.entry(q) {
                            Entry::Occupied(mut c) => {
                                  eprintln!("got packet for known quad {:?}", q);
                                let a = c.get_mut().on_packet(
                                    &mut nic,
                                    iph,
                                    tcph,
                                    &buf[data..nbytes],
                                )?;

                                // TODO: compare before/after
                                drop(cmg);
                                if a.contains(tcp::Available::READ) {
                                    ih.rcv_var.notify_all()
                                }
                                if a.contains(tcp::Available::WRITE) {
                                    // TODO: ih.snd_var.notify_all()
                                }
                            }

                            Entry::Vacant(e) => {
                                eprintln!("got packet for unknown quad {:?}", q);
                                if let Some(pending) = cm.pending.get_mut(&tcph.destination_port()) {
                                    eprintln!("listening and begin accept");
                                    if let Some(c) = tcp::Connection::accept(&mut nic, iph, tcph, &buf[data..nbytes])? {
                                        e.insert(c);
                                        pending.push_back(q);
                                        drop(cmg);
                                        ih.pending_var.notify_all()
                                    }
                                }else {
                                    eprintln!("None in the pending queue");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("igonring weird tcp packet {:?}",e);
                    }
                  
                }
            }
            Err(e) => {
              //  eprintln!("igonring weird packet {:?}",e);
            }
        }
//        eprintln!("byte flags {:x}, proto {:x}",_eth_flags, eth_proto);
        //eprintln!("read {} bytes: {:x?}", nbytes, buf);
    }
}
impl Interface {
    pub fn new() -> io::Result<Self> {
        let nic = tun_tap::Iface::without_packet_info("tun0",tun_tap::Mode::Tun)?;
        let ih: InterfaceHandle = Arc::default();

        // spwan a thread to process the nic packet
        let jh = {
            let ih = ih.clone();
            thread::spawn(move || packet_loop(nic,ih))
        };

        Ok(Interface{
            ih: Some(ih),
            jh: Some(jh),
        })
    }

    pub fn bind(&mut self, port: u16) -> io::Result<TcpListener> {
        let mut cm = self.ih.as_mut().unwrap().manager.lock().unwrap();
        match cm.pending.entry(port) {
            Entry::Vacant(v) => {
                v.insert(VecDeque::new());
            }
            Entry::Occupied(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "port already bound",
                ));
            }
        };

         drop(cm);
        Ok(TcpListener {
            port,
            ih: self.ih.as_mut().unwrap().clone(),
        })
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        self.ih.as_mut().unwrap().manager.lock().unwrap().terminate = true;

        drop(self.ih.take());
        self.jh.take().expect("double free").join().unwrap().unwrap()
    }
}
pub struct TcpListener {
    port: u16,
    ih: InterfaceHandle,
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        let mut cm = self.ih.manager.lock().unwrap();
        let pending = cm
            .pending
            .remove(&self.port)
            .expect("port closed while listener still active");

        for quad in pending {
            // TODO: terminate cm.connections[quad]
            unimplemented!();
        }

    }
}

impl TcpListener {
    pub fn accept(&mut self) -> io::Result<TcpStream> {
        let mut cm = self.ih.manager.lock().unwrap();
        loop {
            if let Some(quad) = cm
                .pending
                .get_mut(&self.port)
                .expect("port closed while listener still active")
                .pop_front()
            {
                return Ok(TcpStream {
                    quad,
                    ih: self.ih.clone(),
                });
            }

            cm = self.ih.pending_var.wait(cm).unwrap();
        }
    }    
}
