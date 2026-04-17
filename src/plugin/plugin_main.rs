use mlua::{Lua, Result, StdLib};
use crate::plugin::action::PluginAction;

pub fn load_plugins(tx: crossbeam::channel::Sender<PluginAction>) -> Result<()> {
    let lua = Lua::new();

    crate::plugin::policy::apply_restrictions(&lua, &tx).expect("Something went wrong with applying resrtictions to plugin Lua.");

    lua.load_std_libs(StdLib::ALL_SAFE)?;

    let _ = crate::plugin::global_set::apply_globals(&lua, tx);

    // TODO: Make 3 worker threads, then make Lua give tasks
    lua.load(r#"
          print("haiiii")
          settings.add("Ryp Is Awesome", true)
          if true then
              print("hoi")
          end
    "#).exec()?;
    Ok(())
}
