use log::error;
use crate::core::{Core, CoreStatus};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TryRecvError;
use crate::api::driver::NamespacedDeviceIdentifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRequestMessage {
    #[serde(default)]
    pub id: u64,
    pub content: ClientRequest
}

impl ClientRequestMessage {
    pub fn respond(&self, content: ClientResponse) -> ClientResponseMessage {
        ClientResponseMessage {
            id: self.id,
            content,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientResponseMessage {
    pub id: u64,
    pub content: ClientResponse
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    Ping,
    ListPlugins,
    RefreshDevices,
    ListDiscoveredDevices,
    Die,
    Restart
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientResponse {
    Pong,
    ListPlugins(Vec<String>),
    RefreshedDevices,
    DiscoveredDevices(Vec<NamespacedDeviceIdentifier>),
    Dead,
    Restarting
}

impl<'core> Core<'core> {
    pub fn process_requests(&'core self) {
        let chan = self.client.get().unwrap();

        loop {
            let request = match chan.rx.borrow_mut().try_recv() {
                Ok(v) => v,
                Err(e) => match e {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => {
                        error!("Client thread died! Exiting...");
                        self.status.set(CoreStatus::ClientThreadDied);
                        return;
                    }
                }
            };

            let resp = request.respond(match &request.content {
                ClientRequest::Ping => ClientResponse::Pong,

                ClientRequest::ListPlugins => {
                    let names = self.plugin_index.borrow()
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>();

                    ClientResponse::ListPlugins(names)
                }

                ClientRequest::Die => {
                    self.status.set(CoreStatus::NormalExit);
                    ClientResponse::Dead
                }

                ClientRequest::RefreshDevices => {
                    self.discover_devices();
                    ClientResponse::RefreshedDevices
                }

                ClientRequest::ListDiscoveredDevices => {
                    ClientResponse::DiscoveredDevices(
                        self.discovered_devices.borrow().iter()
                            .cloned()
                            .collect()
                    )
                }

                ClientRequest::Restart => {
                    self.status.set(CoreStatus::RestartNeeded);
                    ClientResponse::Restarting
                }
            });

            if let Err(_) = chan.tx.send(resp) {
                error!("Client thread died! Exiting...");
                self.status.set(CoreStatus::ClientThreadDied);
                return;
            }
        }
    }
}