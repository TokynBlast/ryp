use mlua;
use crate::plugin::{action::PluginAction};

#[inline]
fn add_setting(name: mlua::String, default_value: mlua::Value) -> Result<(), mlua::Error>  {
  todo!("Both add default value, which can be any type,\nand implement actually adding it to the list of settings");
}

#[inline]
fn get_setting_value() {
  todo!("Add getting settings\nWe need a policy so as the user can choose to set the scope");
}

#[inline]
fn set_setting_value() {
  todo!("Add value setting for Lua implemented settings");
}
