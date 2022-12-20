use std::io;

enum State {
    // Closed,
    // Listen,
    SynRcvd,
    Estab,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
    ip: etherparse::Ipv4Header,
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
            ip: etherparse::Ipv4Header::new(
                0,
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
            ),
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
        c.ip.set_payload_len(syn_ack.header_len() as usize + 0);
        // syn_ack.checksum = syn_ack
        //     .calc_checksum_ipv4(&c.ip, &[])
        //     .expect("failed to compute checksum");

        let unwritten = {
            let mut unwritten = &mut buf[..];
            c.ip.write(&mut unwritten);
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
        // Acceptable ACK check
        // SND>UNA < SEG>ACK =< SND.NXT
        let ackn = tcph.acknowledgment_number();
        if self.send.una < ackn {
            // check is violated iff n is between u and a
            if self.send.nxt >= self.send.una && self.send.nxt < ackn {
                return Ok(());
            }
        } else {
            //check is okay iff n is between u and a
            if self.send.nxt >= ackn && self.send.nxt < self.send.una {
            } else {
                return Ok(());
            }
        }

        // valid segment check

        match self.state {
            State::SynRcvd => {}
            State::Estab => {
                unimplemented!();
            }
        }

        Ok(())
    }
}

fn is_between_wrapped(start: usize, x: usize, end: usize) -> bool {
    use std::cmp::{Ord, Ordering};
    match start.cmp(x) {
        Ordering::Equal => return false,
        Ordering::Less => {
            // check is violated iff end is between start and x
            if end >= start && end <= x {
                return false;
            }
        },
        Ordering::Greater => {
            //check is okay iff end is between start and x
            if end >= x && end < start {
            } else {
                return false;
            }
        }
    }
    true
}
