use bytes::BytesMut;
use netlink_packet_core::{NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST};
use netlink_packet_route::{AddressMessage, RtnlMessage};
use netlink_sys::constants::NETLINK_ROUTE;
use netlink_sys::{Socket, SocketAddr};

use crate::interface::InterfaceType;
use std::convert::TryFrom;
use std::fs::read_to_string;

fn interfaces_netlink() {
    let socket = Socket::new(NETLINK_ROUTE).unwrap();
    let mut req = NetlinkMessage::from(RtnlMessage::GetAddress(AddressMessage::default()));
    req.header.sequence_number = 1;
    req.header.flags = NLM_F_REQUEST | NLM_F_DUMP;
    req.finalize();
    let mut buf = vec![0; req.header.length as usize];
    req.serialize(&mut buf);
    let addr = SocketAddr::new(0, 0);
    let n = socket.send_to(&buf, &addr, 0).unwrap();
    assert_eq!(n, req.header.length as usize);
    let mut buf = BytesMut::with_capacity(4096);
    loop {
        let (n, addr) = socket.recv_from(&mut buf, 0).unwrap();
        let packet = NetlinkMessage::<RtnlMessage>::deserialize(&buf).unwrap();
        println!("{packet:?}");
        buf.clear();
        if matches!(packet.payload, NetlinkPayload::Done) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interfaces_netlink() {
        interfaces_netlink();
    }
}

// The async code that returns everything correctly:
//
// async fn get_addresses() {
//     use futures::stream::TryStreamExt;

//     let (conn, handle, _) = rtnetlink::new_connection().unwrap();
//     let task = tokio::spawn(conn);
//     // let mut links = handle.link().get().execute();
//     // while let Some(msg) = links.try_next().await? {
//     //     let mut addrs = handle.address().get().execute();
//     // }
//     let mut addrs = handle.address().get().execute();
//     while let Some(msg) = addrs.try_next().await.unwrap() {
//         println!("{msg:?}");
//     }
//     task.abort();
//     task.await.ok();
// }

pub fn get_interface_type(if_name: String) -> InterfaceType {
    let if_type_path: String = format!("/sys/class/net/{}/type", if_name);
    let r = read_to_string(if_type_path);
    match r {
        Ok(content) => {
            let if_type_string = content.trim().to_string();
            match if_type_string.parse::<u32>() {
                Ok(if_type) => {
                    return InterfaceType::try_from(if_type).unwrap_or(InterfaceType::Unknown);
                }
                Err(_) => {
                    return InterfaceType::Unknown;
                }
            }
        }
        Err(_) => {
            return InterfaceType::Unknown;
        }
    };
}

pub fn get_interface_speed(if_name: String) -> Option<u64> {
    let if_speed_path: String = format!("/sys/class/net/{}/speed", if_name);
    let r = read_to_string(if_speed_path);
    match r {
        Ok(content) => {
            let if_speed_string = content.trim().to_string();
            match if_speed_string.parse::<u64>() {
                Ok(if_speed) => {
                    // Convert Mbps to bps
                    return Some(if_speed * 1000000);
                }
                Err(_) => {
                    return None;
                }
            }
        }
        Err(_) => {
            return None;
        }
    };
}
