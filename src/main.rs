use std::net::Ipv4Addr;

use windows::Win32::NetworkManagement::IpHelper::{
    GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses,
    IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::Networking::WinSock::{
    AF_INET, AF_UNSPEC, SOCKADDR_IN,
};

fn get_dns_servers(
    adapter_address: &IP_ADAPTER_ADDRESSES_LH,
) -> Vec<Ipv4Addr> {
    let mut addresses: Vec<Ipv4Addr> = vec![];
    unsafe {
        let mut dns_server =
            adapter_address.FirstDnsServerAddress; // Head of dns linked list
        while !dns_server.is_null() {
            let dns = &*dns_server;
            let addr = dns.Address;
            if !addr.lpSockaddr.is_null() {
                let sock_addr = &*(addr.lpSockaddr
                    as *const SOCKADDR_IN);
                if sock_addr.sin_family.0 == AF_INET.0 {
                    let ip = Ipv4Addr::from(u32::from_be(
                        sock_addr.sin_addr.S_un.S_addr,
                    ));
                    addresses.push(ip);
                }
            }
            dns_server = dns.Next;
        }
        addresses
    }
}

fn get_adapter_name(
    adapter_address: &IP_ADAPTER_ADDRESSES_LH,
) -> String {
    unsafe {
        return adapter_address
            .FriendlyName
            .display()
            .to_string();
    }
}

fn cal_buffer_size_for_adapters(size: &mut u32) -> u32 {
    let family = AF_UNSPEC;
    let flags = GAA_FLAG_INCLUDE_PREFIX;

    // First call to get the required buffer size
    let result = unsafe {
        GetAdaptersAddresses(
            family.0 as u32,
            flags,
            None,
            None,
            size,
        )
    };
    result
}

fn get_adapter_ptr(
    size: &mut u32,
    adapters_addresses_ptr: *mut IP_ADAPTER_ADDRESSES_LH,
) -> u32 {
    let family = AF_UNSPEC;
    let flags = GAA_FLAG_INCLUDE_PREFIX;

    let result = unsafe {
        GetAdaptersAddresses(
            family.0 as u32,
            flags,
            None,
            Some(adapters_addresses_ptr),
            size,
        )
    };
    result
}

struct AdapterData {
    name: String,
    adapter_address: IP_ADAPTER_ADDRESSES_LH,
    dns_servers_addresses: Vec<Ipv4Addr>,
}

impl AdapterData {
    fn new(
        name: String,
        adapter_address: IP_ADAPTER_ADDRESSES_LH,
        dns_servers_addresses: Vec<Ipv4Addr>,
    ) -> Self {
        Self {
            adapter_address,
            dns_servers_addresses,
            name,
        }
    }
}

fn get_adapter_data(
    adapter_address: &IP_ADAPTER_ADDRESSES_LH,
) -> AdapterData {
    // Get adapter name
    let adapter_name = get_adapter_name(adapter_address);

    // Get DNS servers
    let dns_server_addresess =
        get_dns_servers(adapter_address);

    let adapter_data = AdapterData::new(
        adapter_name,
        *adapter_address,
        dns_server_addresess,
    );

    adapter_data
}

fn get_adapters_data(
    adapters_addresses_ptr: *mut IP_ADAPTER_ADDRESSES_LH,
) -> Vec<AdapterData> {
    let adapters_data: Vec<AdapterData> = vec![];

    unsafe {
        // adapters_addresses_ptr is the raw pointer to the very beginning of the buffer
        // that GetAdaptersAddresses filled
        let mut current_adapter_addresses_ptr =
            adapters_addresses_ptr;

        while !current_adapter_addresses_ptr.is_null() {
            let adapter_address: &IP_ADAPTER_ADDRESSES_LH =
                &*current_adapter_addresses_ptr;

            let adapter_data =
                get_adapter_data(adapter_address);

            println!(
                "{}: , servers: {:#?}",
                adapter_data.name,
                adapter_data.dns_servers_addresses
            );

            current_adapter_addresses_ptr =
                adapter_address.Next;
        }
    }

    adapters_data
}

fn handle_get_adapters_data(
    adapters_data: &mut Vec<AdapterData>,
) -> windows::core::Result<()> {
    let mut size: u32 = 0;
    let result_of_cal_size =
        cal_buffer_size_for_adapters(&mut size);

    if result_of_cal_size != 111u32 {
        // ERROR_BUFFER_OVERFLOW
        return Err(windows::core::Error::from_win32());
    }

    // Allocate buffer
    let mut buffer = vec![0u8; size as usize];
    let adapters_addresses_ptr =
        buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

    // Second call to get the actual data

    let result_of_get_adapter =
        get_adapter_ptr(&mut size, adapters_addresses_ptr);

    if result_of_get_adapter != 0 {
        return Err(windows::core::Error::from_win32());
    }

    *adapters_data =
        get_adapters_data(adapters_addresses_ptr);

    Ok(())
}

fn main() -> windows::core::Result<()> {
    let mut adapters_data: Vec<AdapterData> = vec![];
    handle_get_adapters_data(&mut adapters_data)?;

    Ok(())
}
