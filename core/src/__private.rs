use {
    std::fmt::Debug,
    tracing::{Span, trace},
};

pub struct TracedResultIterator<I> {
    inner: I,
    span: Span,
    row_count: usize,
    error_count: usize,
    completed: bool,
}

impl<I> TracedResultIterator<I> {
    pub fn new(inner: I, span: Span) -> Self {
        Self {
            inner,
            span,
            row_count: 0,
            error_count: 0,
            completed: false,
        }
    }
}

impl<I, T, E> Iterator for TracedResultIterator<I>
where
    I: Iterator<Item = Result<T, E>>,
    T: Debug,
    E: Debug,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let span = self.span.clone();
        span.in_scope(|| {
            let item = self.inner.next();

            match &item {
                Some(Ok(row)) => {
                    self.row_count += 1;
                    trace!(target: "gluesql", row = ?row, "storage iterator yielded a row");
                }
                Some(Err(error)) => {
                    self.error_count += 1;
                    trace!(target: "gluesql", error = ?error, "storage iterator yielded an error");
                }
                None => self.completed = true,
            }

            item
        })
    }
}

impl<I> Drop for TracedResultIterator<I> {
    fn drop(&mut self) {
        self.span.record("row_count", self.row_count);
        self.span.record("error_count", self.error_count);
        self.span.record("completed", self.completed);
    }
}
