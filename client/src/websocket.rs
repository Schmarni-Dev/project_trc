use ::futures::stream::{SplitSink, SplitStream};
use ::futures::{SinkExt, StreamExt as _};
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, IoTaskPool, Task};
use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::handshake::client::Response;
use tungstenite::Message;
use url::Url;

pub struct WebSocketPlugin<S, R>
where
    S: Serialize + Send + Sync + 'static,
    R: DeserializeOwned + Send + Sync + 'static,
{
    url: Url,
    _send: PhantomData<S>,
    _recv: PhantomData<R>,
}

impl<S, R> WebSocketPlugin<S, R>
where
    S: Serialize + Send + Sync + 'static + Clone,
    R: DeserializeOwned + Send + Sync + 'static,
{
    fn write_ws_msgs(writer: Res<WsSendSender<S, R>>, mut reader: EventReader<WsMsgRecived<S>>) {
        for msg in reader.read() {
            writer.sender.send(msg.0.clone()).unwrap();
        }
    }
    fn read_ws_msgs(reader: Res<WsRecvReceiver<S, R>>, mut writer: EventWriter<WsMsgRecived<R>>) {
        for msg in reader.receiver.try_iter() {
            match msg {
                Ok(msg) => {
                    writer.send(WsMsgRecived(msg));
                }
                Err(err) => error!("{err}"),
            }
        }
    }

    fn poll_ws_creation(mut cmds: Commands, mut task: ResMut<WsCreationTask<S, R>>) {
        if let Some((ws, _response)) = block_on(future::poll_once(&mut task.task)) {
            let (recv_tx, recv_rx) = crossbeam_channel::unbounded();
            let (ws_tx, ws_rx) = ws.split();
            let recv_task = IoTaskPool::get().spawn(Self::ws_poll(ws_rx, recv_tx));
            cmds.insert_resource(WsRecvReceiver::<S, R> {
                receiver: recv_rx,
                _send: default(),
                _recv: default(),
            });
            let (send_tx, send_rx) = crossbeam_channel::unbounded();
            cmds.insert_resource(WsSendSender::<S, R> {
                sender: send_tx,
                _send: default(),
                _recv: default(),
            });
            let send_task = IoTaskPool::get().spawn(Self::send_ws_msgs(send_rx, ws_tx));
            cmds.insert_resource(WsRuntimeTasks::<S, R> {
                recv: recv_task,
                send: send_task,
                _send: default(),
                _recv: default(),
            });
        }
    }
    async fn create_websocket_connection(
        url: Url,
    ) -> (WebSocketStream<MaybeTlsStream<TcpStream>>, Response) {
        connect_async(url).await.unwrap()
    }

    async fn ws_poll(
        mut ws_stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        sender: crossbeam_channel::Sender<Result<R, WsRecvError>>,
    ) {
        while let Ok(msg) = ws_stream.try_next().await {
            let Some(msg) = msg else {
                warn!("no message from server");
                sender.send(Err(WsRecvError::MissingMessage)).unwrap();
                continue;
            };
            match msg {
                tungstenite::Message::Text(text) => {
                    let Ok(packet) = serde_json::from_str::<R>(&text) else {
                        error!("unable to parse packet from server: {}", text);
                        sender.send(Err(WsRecvError::UnableToParseData)).unwrap();
                        continue;
                    };
                    sender.send(Ok(packet)).unwrap();
                }
                tungstenite::Message::Binary(_bin) => todo!(),
                tungstenite::Message::Close(_) => todo!(),
                tungstenite::Message::Frame(_) => {}
                tungstenite::Message::Ping(_) => {}
                tungstenite::Message::Pong(_) => {}
            }
        }
    }

    async fn send_ws_msgs(
        recv: crossbeam_channel::Receiver<S>,
        mut ws_send: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ) {
        loop {
            for msg in recv.try_iter() {
                let msg = serde_json::to_string(&msg).unwrap();
                ws_send.send(Message::Text(msg)).await.unwrap();
            }
            bevy::tasks::futures_lite::future::yield_now().await;
        }
    }
}

#[derive(Event)]
pub struct WsMsgRecived<T>(pub T);
#[derive(Event)]
pub struct SendWsMsg<T>(pub T);

impl<T: Clone> Clone for SendWsMsg<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Copy> Copy for SendWsMsg<T> {}

impl<T: Clone> Clone for WsMsgRecived<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Copy> Copy for WsMsgRecived<T> {}

impl<S, R> Plugin for WebSocketPlugin<S, R>
where
    S: Serialize + Send + Sync + 'static + Clone,
    R: DeserializeOwned + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        let url = self.url.clone();
        app.add_event::<WsMsgRecived<R>>();
        app.add_event::<SendWsMsg<S>>();
        app.add_systems(Last, move |mut cmds: Commands| {
            let task = IoTaskPool::get().spawn(Self::create_websocket_connection(url.clone()));
            cmds.insert_resource(WsCreationTask::<S, R> {
                task,
                _send: default(),
                _recv: default(),
            });
        });
        app.add_systems(Last, Self::poll_ws_creation);
        app.add_systems(
            First,
            Self::read_ws_msgs.run_if(resource_exists::<WsRecvReceiver<S, R>>),
        );
        app.add_systems(
            Last,
            Self::write_ws_msgs.run_if(resource_exists::<WsSendSender<S, R>>),
        );
    }
}

#[derive(Resource)]
struct WsRecvReceiver<S, R: Send + Sync + 'static> {
    receiver: crossbeam_channel::Receiver<Result<R, WsRecvError>>,
    _send: PhantomData<S>,
    _recv: PhantomData<R>,
}
#[derive(Resource)]
struct WsSendSender<S: Send + Sync + 'static, R: Send + Sync + 'static> {
    sender: crossbeam_channel::Sender<S>,
    _send: PhantomData<S>,
    _recv: PhantomData<R>,
}

#[derive(Resource)]
struct WsRuntimeTasks<S, R> {
    recv: Task<()>,
    send: Task<()>,
    _send: PhantomData<S>,
    _recv: PhantomData<R>,
}

#[derive(Resource)]
struct WsCreationTask<S, R> {
    task: Task<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response)>,
    _send: PhantomData<S>,
    _recv: PhantomData<R>,
}

#[derive(thiserror::Error, Debug)]
pub enum WsRecvError {
    #[error("Websocket Closed")]
    Closed,
    #[error("Unable to Parse Data from Websocket to Struct")]
    UnableToParseData,
    #[error("Websocket stream.next was None")]
    MissingMessage,
}
