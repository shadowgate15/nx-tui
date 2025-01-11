use derive_deref::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::broadcast;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,

    // Projects
    GetProjects,
    Projects(String),
}

pub type ActionReceiver = broadcast::Receiver<Action>;

#[derive(Deref, DerefMut)]
pub struct ActionSender(pub broadcast::Sender<Action>);

impl ActionSender {
    pub fn send(&self, action: Action) -> Result<(), broadcast::error::SendError<Action>> {
        self.0.send(action)?;

        Ok(())
    }

    pub fn clone(&self) -> Self {
        self.0.clone().into()
    }
}

impl From<broadcast::Sender<Action>> for ActionSender {
    fn from(sender: broadcast::Sender<Action>) -> Self {
        Self(sender)
    }
}
