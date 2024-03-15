use crate::{
    database::*, Config, DrawSetting
};
use hecs::World;

pub mod game_content;
pub mod menu_content;

pub use game_content::*;
pub use menu_content::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Menu,
    Game,
}

pub struct Content {
    pub menu_content: MenuContent,
    pub game_content: GameContent,
    pub content_type: ContentType,
}

impl Content {
    pub fn new(world: &mut World, systems: &mut DrawSetting) -> Self {
        let mut content = Content {
            content_type: ContentType::Menu,
            menu_content: MenuContent::new(systems),
            game_content: GameContent::new(systems),
        };

        content.menu_content.show(systems);
        content.game_content.hide(world, systems);

        println!("Gfx Count: {:?}", systems.gfx.count_collection());

        content
    }

    pub fn switch_content(&mut self, world: &mut World, systems: &mut DrawSetting, contenttype: ContentType) {
        if self.content_type == contenttype {
            return;
        }
        
        match self.content_type {
            ContentType::Game => {
                self.game_content.hide(world, systems);
            }
            ContentType::Menu => {
                self.menu_content.hide(systems);
            }
        }
        self.content_type = contenttype;

        match self.content_type {
            ContentType::Game => {
                self.game_content.show(systems);
            }
            ContentType::Menu => {
                self.menu_content.show(systems);
            }
        }

        println!("Gfx Count: {:?}", systems.gfx.count_collection());
    }
}