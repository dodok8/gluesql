use {
    super::{AlterError, validate_arg_names, validate_default_args},
    crate::{
        ast::{Expr, OperateFunctionArg},
        data::CustomFunction,
        result::Result,
        store::{GStore, GStoreMut, trace},
    },
};

pub fn insert_function<T: GStore + GStoreMut>(
    storage: &mut T,
    func_name: &str,
    args: &[OperateFunctionArg],
    or_replace: bool,
    body: &Expr,
) -> Result<()> {
    validate_arg_names(args)?;
    validate_default_args(args)?;

    if trace::fetch_function(storage, func_name)?.is_none() || or_replace {
        trace::delete_function(storage, func_name)?;
        trace::insert_function(
            storage,
            CustomFunction {
                func_name: func_name.to_owned(),
                args: args.to_owned(),
                body: body.to_owned(),
            },
        )?;
        Ok(())
    } else {
        Err(AlterError::FunctionAlreadyExists(func_name.to_owned()).into())
    }
}

pub fn delete_function<T: GStore + GStoreMut>(
    storage: &mut T,
    func_names: &[String],
    if_exists: bool,
) -> Result<()> {
    for func_name in func_names {
        let function = trace::fetch_function(storage, func_name)?;

        if !if_exists {
            function.ok_or_else(|| AlterError::FunctionNotFound(func_name.to_owned()))?;
        }

        trace::delete_function(storage, func_name)?;
    }
    Ok(())
}
