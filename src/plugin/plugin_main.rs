use mlua::{Lua, Result, StdLib};

pub fn load_plugins() -> Result<()> {
    let lua = Lua::new();

    crate::plugin::policy::apply_restrictions(&lua).expect("Something went wrong with applying resrtictions to plugin Lua.");

    lua.load_std_libs(StdLib::ALL_SAFE)?;

    let _ = crate::plugin::global_set::apply_globals(&lua);

    // TODO: Make 3 worker threads, then make Lua give tasks
    lua.load(r#"
        --print("This still needs to be done")

    "#).exec()?;
    Ok(())
}
