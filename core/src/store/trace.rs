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

macro_rules! traced {
    ($name:literal, fields($($fields:tt)*), $item:item) => {
        #[cfg_attr(
            feature = "tracing",
            tracing::instrument(
                name = $name,
                target = "gluesql",
                level = "trace",
                skip_all,
                fields(storage.type = std::any::type_name::<T>(), $($fields)*)
            )
        )]
        $item
    };
    ($name:literal, $item:item) => {
        #[cfg_attr(
            feature = "tracing",
            tracing::instrument(
                name = $name,
                target = "gluesql",
                level = "trace",
                skip_all,
                fields(storage.type = std::any::type_name::<T>())
            )
        )]
        $item
    };
}

traced!(
    "gluesql.storage.plan",
    pub(crate) fn plan<T: Planner + ?Sized>(
        storage: &T,
        statement: StatementPlan,
    ) -> Result<StatementPlan> {
        storage.plan(statement)
    }
);

traced!(
    "gluesql.storage.fetch_schema",
    pub(crate) fn fetch_schema<T: Store + ?Sized>(
        storage: &T,
        table_name: &str,
    ) -> Result<Option<Schema>> {
        storage.fetch_schema(table_name)
    }
);

traced!(
    "gluesql.storage.fetch_all_schemas",
    pub(crate) fn fetch_all_schemas<T: Store + ?Sized>(storage: &T) -> Result<Vec<Schema>> {
        storage.fetch_all_schemas()
    }
);

traced!(
    "gluesql.storage.fetch_data",
    pub(crate) fn fetch_data<T: Store + ?Sized>(
        storage: &T,
        table_name: &str,
        key: &Key,
    ) -> Result<Option<Vec<Value>>> {
        storage.fetch_data(table_name, key)
    }
);

traced!(
    "gluesql.storage.scan_data",
    pub(crate) fn scan_data<'a, T: Store + ?Sized>(
        storage: &'a T,
        table_name: &str,
    ) -> Result<RowIter<'a>> {
        storage.scan_data(table_name)
    }
);

traced!(
    "gluesql.storage.fetch_referencings",
    pub(crate) fn fetch_referencings<T: Store + ?Sized>(
        storage: &T,
        table_name: &str,
    ) -> Result<Vec<Referencing>> {
        storage.fetch_referencings(table_name)
    }
);

traced!(
    "gluesql.storage.insert_schema",
    pub(crate) fn insert_schema<T: StoreMut + ?Sized>(
        storage: &mut T,
        schema: &Schema,
    ) -> Result<()> {
        storage.insert_schema(schema)
    }
);

traced!(
    "gluesql.storage.delete_schema",
    pub(crate) fn delete_schema<T: StoreMut + ?Sized>(
        storage: &mut T,
        table_name: &str,
    ) -> Result<()> {
        storage.delete_schema(table_name)
    }
);

traced!(
    "gluesql.storage.append_data",
    fields(row_count = rows.len()),
    pub(crate) fn append_data<T: StoreMut + ?Sized>(
        storage: &mut T,
        table_name: &str,
        rows: Vec<Vec<Value>>,
    ) -> Result<()> {
        storage.append_data(table_name, rows)
    }
);

traced!(
    "gluesql.storage.insert_data",
    fields(row_count = rows.len()),
    pub(crate) fn insert_data<T: StoreMut + ?Sized>(
        storage: &mut T,
        table_name: &str,
        rows: Vec<(Key, Vec<Value>)>,
    ) -> Result<()> {
        storage.insert_data(table_name, rows)
    }
);

traced!(
    "gluesql.storage.delete_data",
    fields(row_count = keys.len()),
    pub(crate) fn delete_data<T: StoreMut + ?Sized>(
        storage: &mut T,
        table_name: &str,
        keys: Vec<Key>,
    ) -> Result<()> {
        storage.delete_data(table_name, keys)
    }
);

traced!(
    "gluesql.storage.scan_indexed_data",
    pub(crate) fn scan_indexed_data<'a, T: Index + ?Sized>(
        storage: &'a T,
        table_name: &str,
        index_name: &str,
        asc: Option<bool>,
        cmp_value: Option<(&IndexOperator, Value)>,
    ) -> Result<RowIter<'a>> {
        storage.scan_indexed_data(table_name, index_name, asc, cmp_value)
    }
);

traced!(
    "gluesql.storage.create_index",
    pub(crate) fn create_index<T: IndexMut + ?Sized>(
        storage: &mut T,
        table_name: &str,
        index_name: &str,
        column: &OrderByExpr,
    ) -> Result<()> {
        storage.create_index(table_name, index_name, column)
    }
);

traced!(
    "gluesql.storage.drop_index",
    pub(crate) fn drop_index<T: IndexMut + ?Sized>(
        storage: &mut T,
        table_name: &str,
        index_name: &str,
    ) -> Result<()> {
        storage.drop_index(table_name, index_name)
    }
);

traced!(
    "gluesql.storage.rename_schema",
    pub(crate) fn rename_schema<T: AlterTable + ?Sized>(
        storage: &mut T,
        table_name: &str,
        new_table_name: &str,
    ) -> Result<()> {
        storage.rename_schema(table_name, new_table_name)
    }
);

traced!(
    "gluesql.storage.rename_column",
    pub(crate) fn rename_column<T: AlterTable + ?Sized>(
        storage: &mut T,
        table_name: &str,
        old_column_name: &str,
        new_column_name: &str,
    ) -> Result<()> {
        storage.rename_column(table_name, old_column_name, new_column_name)
    }
);

traced!(
    "gluesql.storage.add_column",
    pub(crate) fn add_column<T: AlterTable + ?Sized>(
        storage: &mut T,
        table_name: &str,
        column_def: &ColumnDef,
    ) -> Result<()> {
        storage.add_column(table_name, column_def)
    }
);

traced!(
    "gluesql.storage.drop_column",
    pub(crate) fn drop_column<T: AlterTable + ?Sized>(
        storage: &mut T,
        table_name: &str,
        column_name: &str,
        if_exists: bool,
    ) -> Result<()> {
        storage.drop_column(table_name, column_name, if_exists)
    }
);

traced!(
    "gluesql.storage.scan_table_meta",
    pub(crate) fn scan_table_meta<T: Metadata + ?Sized>(storage: &T) -> Result<MetaIter> {
        storage.scan_table_meta()
    }
);

traced!(
    "gluesql.storage.fetch_function",
    pub(crate) fn fetch_function<'a, T: CustomFunction + ?Sized>(
        storage: &'a T,
        func_name: &str,
    ) -> Result<Option<&'a Function>> {
        storage.fetch_function(func_name)
    }
);

traced!(
    "gluesql.storage.fetch_all_functions",
    pub(crate) fn fetch_all_functions<T: CustomFunction + ?Sized>(
        storage: &T,
    ) -> Result<Vec<&Function>> {
        storage.fetch_all_functions()
    }
);

traced!(
    "gluesql.storage.insert_function",
    pub(crate) fn insert_function<T: CustomFunctionMut + ?Sized>(
        storage: &mut T,
        function: Function,
    ) -> Result<()> {
        storage.insert_function(function)
    }
);

traced!(
    "gluesql.storage.delete_function",
    pub(crate) fn delete_function<T: CustomFunctionMut + ?Sized>(
        storage: &mut T,
        func_name: &str,
    ) -> Result<()> {
        storage.delete_function(func_name)
    }
);

traced!(
    "gluesql.storage.begin",
    fields(autocommit = autocommit),
    pub(crate) fn begin<T: Transaction + ?Sized>(
        storage: &mut T,
        autocommit: bool,
    ) -> Result<bool> {
        storage.begin(autocommit)
    }
);

traced!(
    "gluesql.storage.rollback",
    pub(crate) fn rollback<T: Transaction + ?Sized>(storage: &mut T) -> Result<()> {
        storage.rollback()
    }
);

traced!(
    "gluesql.storage.commit",
    pub(crate) fn commit<T: Transaction + ?Sized>(storage: &mut T) -> Result<()> {
        storage.commit()
    }
);
