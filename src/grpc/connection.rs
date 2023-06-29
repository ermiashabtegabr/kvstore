use async_mutex::Mutex;
use crossbeam::queue::ArrayQueue;
use std::collections::HashMap;
use std::sync::Arc;
use tonic::transport::Channel;

use super::omnipaxos_grpc::omni_paxos_protocol_client::OmniPaxosProtocolClient;
use crate::error;

#[derive(Debug)]
pub struct ConnectionPool {
    connections: ArrayQueue<OmniPaxosProtocolClient<Channel>>,
}

pub struct Connection {
    pub conn: OmniPaxosProtocolClient<Channel>,
    pub pool: Arc<ConnectionPool>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.pool.replenish(self.conn.clone())
    }
}

impl ConnectionPool {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: ArrayQueue::new(16),
        })
    }

    async fn connection(
        &self,
        addr: String,
    ) -> Result<OmniPaxosProtocolClient<Channel>, error::Error> {
        let addr = addr.to_string();
        match self.connections.pop() {
            Some(x) => Ok(x),
            None => OmniPaxosProtocolClient::connect(addr)
                .await
                .map_err(|err| error::Error::Connection(err)),
        }
    }

    fn replenish(&self, conn: OmniPaxosProtocolClient<Channel>) {
        let _ = self.connections.push(conn);
    }
}

#[derive(Debug, Clone)]
pub struct Connections {
    connection_map: Arc<Mutex<HashMap<String, Arc<ConnectionPool>>>>,
}

impl Connections {
    pub fn new() -> Self {
        Self {
            connection_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connection<S: ToString>(&self, addr: &S) -> Result<Connection, error::Error> {
        let mut conns = self.connection_map.lock().await;
        let addr = addr.to_string();
        let pool = conns
            .entry(addr.clone())
            .or_insert_with(ConnectionPool::new);
        let connection = pool.connection(addr).await?;
        let connection = Connection {
            conn: connection,
            pool: pool.clone(),
        };

        Ok(connection)
    }
}
