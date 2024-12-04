use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use rand::seq::SliceRandom;

const DNS_SEEDERS: &[&str] = &[
    "seeder1-mainnet.waglayla.com",
    "seeder2-mainnet.waglayla.com",
    "seeder3-mainnet.waglayla.com",
];
const DEFAULT_PORT: u16 = 8333;

/// Queries a single DNS seeder for up to `num_addresses` addresses.
fn dns_seed(seeder: &str, num_addresses: usize) -> Vec<SocketAddr> {
    println!("Querying DNS seeder: {}", seeder);

    // Perform DNS resolution for the seeder
    match (seeder, DEFAULT_PORT).to_socket_addrs() {
        Ok(addrs) => {
            let resolved: Vec<_> = addrs.collect();
            println!(
                "Retrieved {} addresses from seeder {}: {:?}",
                resolved.len(),
                seeder,
                resolved
            );
            resolved.into_iter().take(num_addresses).collect()
        }
        Err(e) => {
            println!("Error resolving DNS seeder {}: {}", seeder, e);
            Vec::new()
        }
    }
}

/// Checks if the target port is open for a given address.
fn is_port_open(addr: &SocketAddr, port: u16) -> bool {
    let test_addr = SocketAddr::new(addr.ip(), port);
    match TcpStream::connect_timeout(&test_addr, Duration::from_secs(2)) {
        Ok(_) => {
            println!("Port {} is open on {}", port, test_addr);
            true
        }
        Err(_) => false,
    }
}

/// Retrieves a public node IP address with port 13110 (WALA wRPC borsh) open.
/// Retries for up to 30 seconds.
pub fn get_public_node(num_addresses: usize, target_port: u16) -> Option<IpAddr> {
    let mut rng = rand::thread_rng();
    let start_time = Instant::now();

    loop {
        // Check for timeout
        if start_time.elapsed() > Duration::from_secs(30) {
            println!("Timed out after 30 seconds without finding an open port.");
            return None;
        }

        // Shuffle seeders to pick randomly
        let shuffled_seeders = DNS_SEEDERS.choose_multiple(&mut rng, DNS_SEEDERS.len());

        for &seeder in shuffled_seeders {
            // Fetch addresses from the current seeder
            let addresses = dns_seed(seeder, num_addresses);

            // Check if any address has the target port open
            for addr in &addresses {
                if is_port_open(addr, target_port) {
                    return Some(addr.ip()); // Return the IP address without the port
                }
            }
        }

        println!("No open ports found, retrying after a short delay...");
        std::thread::sleep(Duration::from_secs(5)); // Wait before retrying
    }
}
