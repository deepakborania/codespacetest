use std::io;

enum State {
    Closed,
    Listen,
    SynRcvd,
    // Estab,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
}

struct SendSequenceSpace {
    /// Send unacknowledged
    una: u32,
    /// Send next
    nxt: u32,
    /// Send window
    wnd: u16,
    /// Send urgent pointer
    up: bool,
    /// segment sequence number for last window update
    wl1: usize,
    /// segment acknowledge number for last window update
    wl2: usize,
    /// Initial send sequence number
    iss: u32,
}

struct RecvSequenceSpace {
    /// Receive next
    nxt: u32,
    /// Receive window
    wnd: u16,
    /// Receive urgent pointer
    up: bool,
    /// Initial Receive sequence number
    irs: u32,
}

// impl Default for Connection {
//     fn default() -> Self {
//         // State::Closed
//         Connection {
//             state: State::Listen,
//         }
//     }
// }

impl Connection {
    pub fn accept<'a>(
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<Option<Self>> {
        let mut buf = [0u8; 1500];

        if !tcph.syn() {
            // only expected SYN packet
            return Ok(None);
        }

        let iss = 0;
        let mut c = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0,
            },
            recv: RecvSequenceSpace {
                nxt: tcph.sequence_number() + 1,
                wnd: tcph.window_size(),
                irs: tcph.sequence_number(),
                up: false,
            },
        };

        //keep track of sender info

        //decide on the stuff we are sending them

        let mut syn_ack = etherparse::TcpHeader::new(
            tcph.destination_port(),
            tcph.source_port(),
            c.send.iss,
            c.send.wnd,
        );
        syn_ack.acknowledgment_number = c.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;

        let mut ip = etherparse::Ipv4Header::new(
            syn_ack.header_len(),
            64,
            etherparse::IpTrafficClass::Tcp,
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
            ],
        );

        // syn_ack.checksum = syn_ack
        //     .calc_checksum_ipv4(&ip, &[])
        //     .expect("failed to compute checksum");

        let unwritten = {
            let mut unwritten = &mut buf[..];
            ip.write(&mut unwritten);
            syn_ack.write(&mut unwritten);
            unwritten.len()
        };
        nic.send(&buf[..(buf.len() - unwritten)])?;
        Ok(Some(c))
    }

    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<()> {
        match self.state {
            
        }
    }
}
