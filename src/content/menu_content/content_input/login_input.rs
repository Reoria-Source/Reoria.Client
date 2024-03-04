use graphics::*;
use winit::{
    event::*,
    keyboard::*,
};
use hecs::World;

use crate::{button, content::*, ContentType, DrawSetting, MouseInputType, fade::*, logic::FloatFix};

pub fn login_mouse_input(
    menu_content: &mut MenuContent,
    _world: &mut World,
    systems: &mut DrawSetting,
    input_type: MouseInputType,
    screen_pos: Vec2,
) {
    match input_type {
        MouseInputType::MouseMove => {
            hover_buttons(menu_content, systems, screen_pos);
            hover_checkbox(menu_content, systems, screen_pos);
        }
        MouseInputType::MouseLeftDown => {
            let button_index = click_buttons(menu_content, systems, screen_pos);
            if let Some(index) = button_index {
                menu_content.did_button_click = true;
                trigger_button(menu_content, systems, index);
            }

            let checkbox_index = click_checkbox(menu_content, systems, screen_pos);
            if let Some(index) = checkbox_index {
                menu_content.did_checkbox_click = true;
                trigger_checkbox(menu_content, systems, index);
            }

            click_textbox(menu_content, systems, screen_pos);
        }
        MouseInputType::MouseRelease => {
            reset_buttons(menu_content, systems);
            reset_checkbox(menu_content, systems);
        }
        _ => {}
    }
}

pub fn login_key_input(
    menu_content: &mut MenuContent,
    _world: &mut World,
    systems: &mut DrawSetting,
    event: &KeyEvent,
) {
    if let Some(textbox_index) = menu_content.selected_textbox {
        menu_content.textbox[textbox_index].enter_text(systems, event);
    }
}

fn trigger_button(
    menu_content: &mut MenuContent,
    systems: &mut DrawSetting,
    index: usize,
) {
    match index {
        0 => { // Login
            println!("Login");
            println!("Email: {:?}", menu_content.textbox[0].text);
            println!("Password: {:?}", menu_content.textbox[1].text);
            systems.fade.init_fade(&mut systems.gfx, FadeType::In, FADE_LOGIN);
        }
        1 => { // Register
            let fvalue = 0.09999;
            let result = fvalue.add_f32(0.00001, 5);
            println!("result: {result}");
            let resultb = result.add_f32(0.00001, 5);
            println!("result b: {resultb}");
            /*println!("Register");
            println!("Old Collection {:?}", systems.gfx.count_collection());
            create_window(systems, menu_content, WindowType::Register);
            println!("New Collection {:?}", systems.gfx.count_collection());*/
        }
        _ => {}
    }
}

fn trigger_checkbox(
    _menu_content: &mut MenuContent,
    _systems: &mut DrawSetting,
    index: usize,
) {
    match index {
        0 => { // Remember Account
            println!("Remember Account");
        }
        _ => {}
    }
}