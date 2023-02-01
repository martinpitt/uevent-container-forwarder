use netlink_sys::{protocols::NETLINK_KOBJECT_UEVENT, Socket, SocketAddr};

fn remove_seqnum(input: &[u8]) -> Vec<u8> {
    let seqnum = b"SEQNUM=";
    let seqnum_len = seqnum.len();

    let mut output = Vec::new();
    let mut i = 0;
    while i < input.len() {
        if input[i..].starts_with(seqnum) {
            if let Some(zero_pos) = input[i + seqnum_len..].iter().position(|&b| b == 0) {
                i += seqnum_len + zero_pos + 1;
            } else {
                panic!("did not find SEQNUM= end");
            }
        } else {
            output.push(input[i]);
            i += 1;
        }
    }
    output
}

fn main() {
    let mut host_socket = Socket::new(NETLINK_KOBJECT_UEVENT).unwrap();
    host_socket
        // multicast group 1: kernel, 2: udev
        .bind(&SocketAddr::new(0, 2))
        .expect("Failed to bind netlink socket");

    let mut buf = vec![0; 1024 * 8];
    loop {
        let len = host_socket.recv(&mut buf, 0).unwrap();
        println!(">> {:?}", &buf[0..len]);
        // the header contains non-UTF8 junk, don't try to print it
        let s = std::str::from_utf8(&buf[40..len]).unwrap();
        println!(">> {}", s);

        // drop sequence number
        let out = remove_seqnum(&buf[0..len]);
        let sout = std::str::from_utf8(&out[40..out.len()]).unwrap();
        println!("!SEQ>> {}", sout);

        // write it back as user
        let wlen = host_socket.send(&buf[0..len], 0).unwrap();
        if wlen != len {
            println!("WARNING: could only write {} out of {} bytes", wlen, len);
        }
    }
}
