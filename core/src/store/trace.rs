use {
    super::{
        AlterTable, CustomFunction, CustomFunctionMut, Index, IndexMut, MetaIter, Metadata,
        Planner, RowIter, Store, StoreMut, Transaction,
    },
    crate::{
        ast::{ColumnDef, IndexOperator, OrderByExpr},
        data::{CustomFunction as Function, Key, Schema, Value},
        executor::Referencing,
        plan::StatementPlan,
        result::Result,
    },
};

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.plan",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn plan<T: Planner + ?Sized>(
    storage: &T,
    statement: StatementPlan,
) -> Result<StatementPlan> {
    storage.plan(statement)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.fetch_schema",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn fetch_schema<T: Store + ?Sized>(
    storage: &T,
    table_name: &str,
) -> Result<Option<Schema>> {
    storage.fetch_schema(table_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.fetch_all_schemas",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn fetch_all_schemas<T: Store + ?Sized>(storage: &T) -> Result<Vec<Schema>> {
    storage.fetch_all_schemas()
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.fetch_data",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn fetch_data<T: Store + ?Sized>(
    storage: &T,
    table_name: &str,
    key: &Key,
) -> Result<Option<Vec<Value>>> {
    storage.fetch_data(table_name, key)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.scan_data",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn scan_data<'a, T: Store + ?Sized>(
    storage: &'a T,
    table_name: &str,
) -> Result<RowIter<'a>> {
    storage.scan_data(table_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.fetch_referencings",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn fetch_referencings<T: Store + ?Sized>(
    storage: &T,
    table_name: &str,
) -> Result<Vec<Referencing>> {
    storage.fetch_referencings(table_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.insert_schema",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn insert_schema<T: StoreMut + ?Sized>(storage: &mut T, schema: &Schema) -> Result<()> {
    storage.insert_schema(schema)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.delete_schema",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn delete_schema<T: StoreMut + ?Sized>(storage: &mut T, table_name: &str) -> Result<()> {
    storage.delete_schema(table_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.append_data",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(
            storage.type = std::any::type_name::<T>(),
            row_count = rows.len()
        )
    )
)]
pub(crate) fn append_data<T: StoreMut + ?Sized>(
    storage: &mut T,
    table_name: &str,
    rows: Vec<Vec<Value>>,
) -> Result<()> {
    storage.append_data(table_name, rows)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.insert_data",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(
            storage.type = std::any::type_name::<T>(),
            row_count = rows.len()
        )
    )
)]
pub(crate) fn insert_data<T: StoreMut + ?Sized>(
    storage: &mut T,
    table_name: &str,
    rows: Vec<(Key, Vec<Value>)>,
) -> Result<()> {
    storage.insert_data(table_name, rows)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.delete_data",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(
            storage.type = std::any::type_name::<T>(),
            row_count = keys.len()
        )
    )
)]
pub(crate) fn delete_data<T: StoreMut + ?Sized>(
    storage: &mut T,
    table_name: &str,
    keys: Vec<Key>,
) -> Result<()> {
    storage.delete_data(table_name, keys)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.scan_indexed_data",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn scan_indexed_data<'a, T: Index + ?Sized>(
    storage: &'a T,
    table_name: &str,
    index_name: &str,
    asc: Option<bool>,
    cmp_value: Option<(&IndexOperator, Value)>,
) -> Result<RowIter<'a>> {
    storage.scan_indexed_data(table_name, index_name, asc, cmp_value)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.create_index",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn create_index<T: IndexMut + ?Sized>(
    storage: &mut T,
    table_name: &str,
    index_name: &str,
    column: &OrderByExpr,
) -> Result<()> {
    storage.create_index(table_name, index_name, column)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.drop_index",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn drop_index<T: IndexMut + ?Sized>(
    storage: &mut T,
    table_name: &str,
    index_name: &str,
) -> Result<()> {
    storage.drop_index(table_name, index_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.rename_schema",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn rename_schema<T: AlterTable + ?Sized>(
    storage: &mut T,
    table_name: &str,
    new_table_name: &str,
) -> Result<()> {
    storage.rename_schema(table_name, new_table_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.rename_column",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn rename_column<T: AlterTable + ?Sized>(
    storage: &mut T,
    table_name: &str,
    old_column_name: &str,
    new_column_name: &str,
) -> Result<()> {
    storage.rename_column(table_name, old_column_name, new_column_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.add_column",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn add_column<T: AlterTable + ?Sized>(
    storage: &mut T,
    table_name: &str,
    column_def: &ColumnDef,
) -> Result<()> {
    storage.add_column(table_name, column_def)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.drop_column",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn drop_column<T: AlterTable + ?Sized>(
    storage: &mut T,
    table_name: &str,
    column_name: &str,
    if_exists: bool,
) -> Result<()> {
    storage.drop_column(table_name, column_name, if_exists)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.scan_table_meta",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn scan_table_meta<T: Metadata + ?Sized>(storage: &T) -> Result<MetaIter> {
    storage.scan_table_meta()
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.fetch_function",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn fetch_function<'a, T: CustomFunction + ?Sized>(
    storage: &'a T,
    func_name: &str,
) -> Result<Option<&'a Function>> {
    storage.fetch_function(func_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.fetch_all_functions",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn fetch_all_functions<T: CustomFunction + ?Sized>(
    storage: &T,
) -> Result<Vec<&Function>> {
    storage.fetch_all_functions()
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.insert_function",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn insert_function<T: CustomFunctionMut + ?Sized>(
    storage: &mut T,
    function: Function,
) -> Result<()> {
    storage.insert_function(function)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.delete_function",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn delete_function<T: CustomFunctionMut + ?Sized>(
    storage: &mut T,
    func_name: &str,
) -> Result<()> {
    storage.delete_function(func_name)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.begin",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(
            storage.type = std::any::type_name::<T>(),
            autocommit = autocommit
        )
    )
)]
pub(crate) fn begin<T: Transaction + ?Sized>(storage: &mut T, autocommit: bool) -> Result<bool> {
    storage.begin(autocommit)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.rollback",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn rollback<T: Transaction + ?Sized>(storage: &mut T) -> Result<()> {
    storage.rollback()
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "gluesql.storage.commit",
        target = "gluesql",
        level = "trace",
        skip_all,
        fields(storage.type = std::any::type_name::<T>())
    )
)]
pub(crate) fn commit<T: Transaction + ?Sized>(storage: &mut T) -> Result<()> {
    storage.commit()
}
