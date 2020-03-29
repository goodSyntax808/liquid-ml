use crate::dataframe::DataFrame;
use crate::dataframe::Rower;
use crate::error::LiquidError;
use crate::kv::{KVMessage, KVStore, Key, Value};
use crate::network::client::Client;
use bincode::{deserialize, serialize};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::{Notify, RwLock};
use tokio::task::JoinHandle;

pub struct Application {
    pub kv: Arc<KVStore>,
    pub node_id: usize,
    pub blob_receiver: Receiver<Value>,
    // TODO: maybe use a runtime here
    msg_processor: JoinHandle<()>,
    conn_processor: JoinHandle<()>,
    num_nodes: usize,
}

impl Application {
    pub async fn new(
        my_addr: &str,
        server_addr: &str,
        num_nodes: usize,
    ) -> Result<Self, LiquidError> {
        let notifier = Arc::new(Notify::new());
        let c = Client::<KVMessage>::new(
            server_addr.to_string(),
            my_addr.to_string(),
            notifier.clone(),
        )
        .await?;
        let node_id = c.id;
        let arc = Arc::new(RwLock::new(c));
        let (blob_sender, blob_receiver) = channel(2);
        let kv = KVStore::new(arc.clone(), notifier, node_id, blob_sender);
        let fut0 = tokio::spawn(async move {
            Client::accept_new_connections(arc).await.unwrap();
        });
        let kv_arc = Arc::new(kv);
        let arc_new = kv_arc.clone();
        let fut1 = tokio::spawn(async move {
            KVStore::process_messages(arc_new).await.unwrap();
        });
        Ok(Application {
            kv: kv_arc,
            node_id,
            blob_receiver,
            msg_processor: fut1,
            conn_processor: fut0,
            num_nodes,
        })
    }

    pub async fn from_sor(
        file_name: &str,
        my_addr: &str,
        server_addr: &str,
        num_nodes: usize,
    ) -> Result<Self, LiquidError> {
        let app = Application::new(my_addr, server_addr, num_nodes).await?;
        let file = std::fs::metadata(file_name).unwrap();
        // Note: Node ids start at 1
        // TODO: IMPORTANT ROUNDING ERRORS
        let size = file.len() / num_nodes as u64;
        let from = size * (app.node_id - 1) as u64;
        let df = DataFrame::from_sor(
            String::from(file_name),
            from as usize,
            size as usize,
        );
        let key = Key::new("420", app.node_id);
        app.kv.put(&key, df).await?;
        Ok(app)
    }

    ///
    ///
    /// NOTE:
    ///
    /// There is an important design decision that comes with a distinct trade
    /// off here. The trade off is:
    /// 1. Join the last node with the next one until you get to the end. This
    ///    has reduced memory requirements but a performance impact because
    ///    of the synchronous network calls
    /// 2. Join all nodes with one node. This has increased memory requirements
    ///    but greater performance because all nodes can asynchronously send
    ///    to the joiner at one time.
    pub async fn pmap<R>(
        &mut self,
        df_name: &str,
        rower: R,
    ) -> Result<Option<R>, LiquidError>
    where
        R: Rower + Serialize + DeserializeOwned + Send + Clone,
    {
        println!("{}", df_name);
        match self.kv.get(&Key::new("420", self.node_id)).await {
            Ok(df) => {
                let mut res = df.pmap(rower);
                if self.node_id != self.num_nodes {
                    // we are the last node
                    let blob = serialize(&res)?;
                    self.kv.send_blob(self.node_id - 1, blob).await?;
                    Ok(None)
                } else {
                    let mut blob = self.blob_receiver.recv().await.unwrap();
                    let external_rower: R = deserialize(&blob[..])?;
                    res = res.join(&external_rower);
                    if self.node_id != 1 {
                        blob = serialize(&res)?;
                        self.kv.send_blob(self.node_id - 1, blob).await?;
                        Ok(None)
                    } else {
                        Ok(Some(res))
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn run<F, Fut>(self, f: F)
    where
        Fut: Future<Output = ()>,
        F: FnOnce(Arc<KVStore>) -> Fut,
    {
        f(self.kv.clone()).await;
        self.go().await;
    }

    pub async fn go(self) {
        self.msg_processor.await.unwrap();
        self.conn_processor.await.unwrap();
    }
}