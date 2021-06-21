use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
};
use serde_json::Value as Json;

#[derive(Clone, Copy)]
pub struct DirIfHelper;

impl HelperDef for DirIfHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"dir-if\""))?;

        let value = param.value().is_truthy();

        if value {
            out.write("1")?;
        }

        Ok(())
    }
}

pub static DIR_IF_HELPER: DirIfHelper = DirIfHelper {};

trait JsonTruthy {
    fn is_truthy(&self) -> bool;
}

impl JsonTruthy for Json {
    fn is_truthy(&self) -> bool {
        match *self {
            Json::Bool(ref i) => *i,
            Json::Number(ref n) => {
                // there is no infinity in json/serde_json
                n.as_f64().map(|f| f.is_normal()).unwrap_or(false)
            }
            Json::Null => false,
            Json::String(ref i) => !i.is_empty(),
            Json::Array(ref i) => !i.is_empty(),
            Json::Object(ref i) => !i.is_empty(),
        }
    }
}
