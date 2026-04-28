use mlua;
use crate::plugin::{action::PluginAction};
use mlua::LuaSerdeExt;
use triomphe::Arc;
use parking_lot::{Mutex, Condvar};

#[inline]
fn add_setting(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error>  {
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
fn get_setting_value(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error> {
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
fn set_setting_value(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error> {
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
pub fn integrate_settings(lua: &mlua::Lua, tx: crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let internal_table = lua.create_table()?;
    add_setting(lua, &tx, &internal_table)?;
    get_setting_value(lua, &tx, &internal_table)?;
    set_setting_value(lua, &tx, &internal_table)?;

    let proxy = lua.create_table()?;
    let metatable = lua.create_table()?;

    // Handle Access: When user calls settings.add, look it up in internal_table
    let internal_clone = internal_table.clone();
    metatable.set("__index", lua.create_function(move |_, key: String| {
        internal_clone.get::<mlua::Value>(key)
    })?)?;

    // TODO: Implement this. This is a safety feature.
    //       When a dev tries to modify the table, they become suspicous, and we shouldn't let them continue.

    // let tx_shutdown = tx.clone();
    // metatable.set("__newindex", lua.create_function(move |_, (_t, _k, _v): (mlua::Value, mlua::Value, mlua::Value)| {
    //     let _ = tx_shutdown.send(PluginAction::Shutdown);
    //     Err::<(), mlua::Error>(mlua::Error::RuntimeError(
    //         "Attempt to modify settings".into()
    //     ))
    // })?)?;

    proxy.set_metatable(Some(metatable))?;
    lua.globals().set("settings", proxy)?;

    Ok(())
}
