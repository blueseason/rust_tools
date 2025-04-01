use std::io;

enum State {
    Listen,
    SynRcvd,
    Estab,
}

pub struct Connection {
    state: State,
}


impl Connection {
    pub fn on_packet(&mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8]) -> io::Result<()> {
        Ok(())
    }
}
