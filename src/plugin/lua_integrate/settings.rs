use mlua;
use crate::plugin::{action::PluginAction};
use mlua::LuaSerdeExt;
use triomphe::Arc;
use parking_lot::{Mutex, Condvar};

#[inline]
fn add_setting(lua: &mlua::Lua, tx: &crossbeam::channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error>  {
    // We clone the sender for each function
    let tx_add = tx.clone();

    settings_table.set("add",
        lua.create_function(move |lua, (name, value): (String, mlua::Value)| {
            let json_value: serde_json::Value = lua.from_value(value)?;
            let _ = tx_add.send(PluginAction::MakeSetting { name, value: json_value }).ok();
            Ok(())
        })?
    )?;
    Ok(())
}

#[inline]
fn get_setting_value(lua: &mlua::Lua, tx: &crossbeam::channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx_get = tx.clone();

    settings_table.set("get",
        lua.create_function(move |lua_ctx, name: String| {
            // 1. Create the high-speed responder
            let responder = Arc::new(crate::plugin::action::Responder {
                value: Mutex::new(None),
                signal: Condvar::new(),
            });

            // 2. Send the Arc to the App
            tx_get.send(PluginAction::GetSettingValue {
                name: name.clone(),
                responder: responder.clone()
            }).ok();

            // 3. LOCK and WAIT
            let mut lock = responder.value.lock();
            if lock.is_none() {
                // This parks the thread until App calls .notify_one()
                responder.signal.wait(&mut lock);
            }

            // 4. Take the value and convert to Lua
            let info = lock.take().unwrap_or(serde_json::Value::Null);
            lua_ctx.to_value(&info)
        })?
    )?;
    Ok(())
}

#[inline]
fn set_setting_value(lua: &mlua::Lua, tx: &crossbeam::channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx_set = tx.clone();

    settings_table.set("set",
        lua.create_function(move |lua, (name, value): (String, mlua::Value)| {
            let json_value: serde_json::Value = lua.from_value(value)?;
            let _ = tx_set.send(PluginAction::SetSetting { name, value: json_value });
            Ok(())
        })?
    )?;
    Ok(())
}

#[inline]
pub fn integrate_settings(lua: &mlua::Lua, tx: crossbeam::channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let settings_table = lua.create_table()?;
    let globals = lua.globals();
    add_setting(lua, &tx, &settings_table)?;
    get_setting_value(lua, &tx, &settings_table)?;
    set_setting_value(lua, &tx, &settings_table)?;
    globals.set("settings", settings_table)?;
    Ok(())
}
