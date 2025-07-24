use std::net::Ipv4Addr;

use windows::Win32::NetworkManagement::IpHelper::{
    GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses,
    IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::Networking::WinSock::{
    ADDRESS_FAMILY, AF_INET, AF_UNSPEC, SOCKADDR_IN,
};

fn main() -> windows::core::Result<()> {
    let mut size: u32 = 0;
    let family = AF_UNSPEC;
    let flags = GAA_FLAG_INCLUDE_PREFIX;

    // First call to get the required buffer size
    let result = unsafe {
        GetAdaptersAddresses(
            family.0 as u32,
            flags,
            None,
            None,
            &mut size,
        )
    };

    if result != 111u32 {
        // ERROR_BUFFER_OVERFLOW
        return Err(windows::core::Error::from_win32());
    }

    // Allocate buffer
    let mut buffer = vec![0u8; size as usize];
    let addresses =
        buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

    // Second call to get the actual data
    let result = unsafe {
        GetAdaptersAddresses(
            family.0 as u32,
            flags,
            None,
            Some(addresses),
            &mut size,
        )
    };

    if result != 0 {
        return Err(windows::core::Error::from_win32());
    }

    unsafe {
        let mut current = addresses;
        while !current.is_null() {
            let adapter = &*current;

            // Get adapter name
            let friendly_name =
                adapter.FriendlyName.display();
            println!("\nAdapter: {}", friendly_name);

            // Get DNS servers
            let mut dns_server =
                adapter.FirstDnsServerAddress;
            while !dns_server.is_null() {
                let dns = &*dns_server;
                let addr = dns.Address;
                if !addr.lpSockaddr.is_null() {
                    let sock_addr = &*(addr.lpSockaddr
                        as *const SOCKADDR_IN);
                    if sock_addr.sin_family.0 == AF_INET.0 {
                        let ip =
                            Ipv4Addr::from(u32::from_be(
                                sock_addr
                                    .sin_addr
                                    .S_un
                                    .S_addr,
                            ));
                        println!("DNS Server: {}", ip);
                    }
                }
                dns_server = dns.Next;
            }

            current = adapter.Next;
        }
    }

    Ok(())
}
