use mlua::{UserData, UserDataMethods, MetaMethod, FromLua, Value, Lua, Result};

#[derive(Clone)]
struct LuaType {
    name: String,
}

impl FromLua for LuaType {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaType".to_string(),
                message: None,
            }),
        }
    }
}

impl UserData for LuaType {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {

        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(this.name.clone())
        });

        methods.add_meta_method(MetaMethod::NewIndex, |_, this, (_k, _v): (Value, Value)| {
            Err::<(), _>(mlua::Error::RuntimeError(format!(
                "Cannot assign type {} as value",
                this.name
            )))
        });

        methods.add_meta_method(MetaMethod::Eq, |_, this, other: LuaType| {
            Ok(this.name == other.name)
        });
    }
}
