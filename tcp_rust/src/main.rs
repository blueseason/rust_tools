use std::{collections::HashMap, io};
use std::collections::hash_map::Entry;
use etherparse::IpNumber;

use std::net::Ipv4Addr;
extern crate tun_tap;

mod tcp;

#[derive(Debug,Clone,Copy,Eq,Hash,PartialEq)]
struct Quad {
    src: (Ipv4Addr,u16),
    dst: (Ipv4Addr,u16),
}


fn main() -> io::Result<()>{
    let mut connections :HashMap<Quad,tcp::Connection> = Default::default();
    let mut nic = tun_tap::Iface::without_packet_info("tun0",tun_tap::Mode::Tun)?;
    let mut buf = [0u8;1504];
    loop {
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
                        match connections.entry(Quad{
                            src:(src,tcph.source_port()),
                            dst:(dst,tcph.destination_port()),
                        }) {
                            Entry::Occupied(mut c) => {
                                c.get_mut().on_packet(&mut nic,iph,tcph,&buf[data..nbytes])?;
                            }

                            Entry::Vacant(e) => {
                                if let Some(c) = tcp::Connection::accept(&mut nic, iph, tcph, &buf[data..nbytes])? {
                                    e.insert(c);
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
                eprintln!("igonring weird packet {:?}",e);
            }
        }
//        eprintln!("byte flags {:x}, proto {:x}",_eth_flags, eth_proto);
        //eprintln!("read {} bytes: {:x?}", nbytes, buf);
    }
}
