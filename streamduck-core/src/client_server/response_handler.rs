use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use tokio::sync::Notify;
use rand::random;
use crate::client_server::ClientChannels;
use crate::core::requests::{ClientRequestMessage, ClientResponseMessage};

#[derive(Clone)]
pub struct ResponseHandler {
    chan: &'static ClientChannels,
    waiters: Rc<RefCell<HashMap<u64, Rc<Notify>>>>,
    responses: Rc<RefCell<HashMap<u64, ClientResponseMessage>>>
}

impl ResponseHandler {
    pub fn new(chan: &'static ClientChannels) -> Self {
        Self {
            chan,
            waiters: Rc::new(HashMap::new().into()),
            responses: Rc::new(HashMap::new().into())
        }
    }

    pub async fn wait_for_responses(&self) -> anyhow::Result<()> {
        let Some(res) = self.chan.rx.borrow_mut().recv().await else {
            return Err(anyhow::Error::msg("Request channel died"))
        };

        let id = res.id;

        self.responses.borrow_mut().insert(id, res);
        let waiter = self.waiters.borrow_mut().remove(&id);

        let Some(waiter) = waiter else {
            return Ok(());
        };

        waiter.notify_one();

        Ok(())
    }

    fn unique_id(&self) -> u64 {
        let map = self.waiters.borrow();

        let mut id = random::<u64>();

        while map.contains_key(&id) {
            id = random::<u64>();
        }

        id
    }

    async fn get_response(&self, id: u64) -> ClientResponseMessage {
        let notifier = Rc::new(Notify::new());

        self.waiters.borrow_mut().insert(id, notifier.clone());
        notifier.notified().await;
        self.responses.borrow_mut().remove(&id).expect("Notified without having the response")
    }

    pub async fn request(&self, mut request: ClientRequestMessage) -> ClientResponseMessage {
        let wanted_id = request.id;
        let new_id = self.unique_id();

        request.id = new_id;

        self.chan.tx.send(request).unwrap();
        let mut resp = self.get_response(new_id).await;

        resp.id = wanted_id;

        resp
    }
}