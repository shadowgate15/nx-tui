use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{
    action::{Action, ActionSender},
    config::Config,
};

#[derive(Default)]
pub struct Projects {
    command_tx: Option<ActionSender>,
    config: Config,
}

impl Projects {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Projects {
    fn register_action_handler(&mut self, tx: ActionSender) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn init(&mut self, _area: Size) -> Result<()> {
        if let Some(tx) = &self.command_tx {
            tx.send(Action::GetProjects)?;
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let border = Block::bordered().title(" Projects ");
        let inner_area = border.inner(area);

        frame.render_widget(border, area);
        frame.render_widget(Paragraph::new("hello world"), inner_area);
        Ok(())
    }
}
