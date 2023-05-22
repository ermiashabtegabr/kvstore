pub mod kv;
pub mod rest;
pub mod rpc;
pub mod server;
pub mod util;

use omnipaxos::OmniPaxos;
use omnipaxos_storage::memory_storage::MemoryStorage;

type OmniPaxosKV<E> = OmniPaxos<E, MemoryStorage<E>>;

fn main() {
    unimplemented!();
}
