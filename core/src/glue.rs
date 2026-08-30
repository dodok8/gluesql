use crate::{
    executor::{Payload, execute},
    parse_sql::parse,
    plan::StatementPlan,
    result::Result,
    store::{GStore, GStoreMut, Planner, trace},
    translate::{IntoParamLiteral, ParamLiteral, translate_with_params},
};

#[derive(Debug)]
pub struct Glue<T: GStore + GStoreMut + Planner> {
    pub storage: T,
}

impl<T: GStore + GStoreMut + Planner> Glue<T> {
    pub fn new(storage: T) -> Self {
        #[cfg(feature = "tracing")]
        crate::__private::ensure_default_subscriber();

        Self { storage }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "gluesql.plan", target = "gluesql", level = "debug", skip_all)
    )]
    fn plan_statement(&self, statement: StatementPlan) -> Result<StatementPlan> {
        trace::plan(&self.storage, statement)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "gluesql.plan_sql",
            target = "gluesql",
            level = "debug",
            skip_all,
            fields(
                sql = %sql,
                params = ?params
            )
        )
    )]
    fn plan_param_literals(
        &self,
        sql: &str,
        params: &[ParamLiteral],
    ) -> Result<Vec<StatementPlan>> {
        parse(sql)?
            .into_iter()
            .map(|p| {
                translate_with_params(&p, params)
                    .and_then(|statement| self.plan_statement(statement.into()))
            })
            .collect()
    }

    /// Plans all statements in the SQL string using the supplied parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing the SQL text fails or when building an execution plan for
    /// a statement fails.
    pub fn plan_with_params<Sql, I, P>(&mut self, sql: Sql, params: I) -> Result<Vec<StatementPlan>>
    where
        Sql: AsRef<str>,
        I: IntoIterator<Item = P>,
        P: IntoParamLiteral,
    {
        let params: Vec<ParamLiteral> = params
            .into_iter()
            .map(IntoParamLiteral::into_param_literal)
            .collect();

        self.plan_param_literals(sql.as_ref(), &params)
    }

    /// Plans all statements in the SQL string without parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing the SQL text fails or when planning one of the
    /// statements fails.
    pub fn plan<Sql: AsRef<str>>(&mut self, sql: Sql) -> Result<Vec<StatementPlan>> {
        self.plan_with_params(sql, std::iter::empty::<ParamLiteral>())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "gluesql.execute_statement",
            target = "gluesql",
            level = "debug",
            skip_all
        )
    )]
    pub fn execute_stmt(&mut self, statement: &StatementPlan) -> Result<Payload> {
        execute(&mut self.storage, statement)
    }

    /// Executes all statements in the SQL string using the supplied parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing fails, planning fails, or executing a statement
    /// against the storage fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "gluesql.execute",
            target = "gluesql",
            level = "info",
            skip_all,
            fields(
                sql = %sql.as_ref(),
                params = tracing::field::Empty
            )
        )
    )]
    pub fn execute_with_params<Sql, I, P>(&mut self, sql: Sql, params: I) -> Result<Vec<Payload>>
    where
        Sql: AsRef<str>,
        I: IntoIterator<Item = P>,
        P: IntoParamLiteral,
    {
        let params: Vec<ParamLiteral> = params
            .into_iter()
            .map(IntoParamLiteral::into_param_literal)
            .collect();
        #[cfg(feature = "tracing")]
        tracing::Span::current().record("params", tracing::field::debug(&params));
        let statements = self.plan_param_literals(sql.as_ref(), &params)?;
        let mut payloads = Vec::<Payload>::new();
        for statement in &statements {
            let payload = self.execute_stmt(statement)?;
            payloads.push(payload);
        }

        Ok(payloads)
    }

    /// Executes all statements in the SQL string without parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing fails, planning fails, or executing a statement fails.
    pub fn execute<Sql: AsRef<str>>(&mut self, sql: Sql) -> Result<Vec<Payload>> {
        self.execute_with_params(sql, std::iter::empty::<ParamLiteral>())
    }
}
