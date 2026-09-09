//! Universal Interceptor Pipeline and Chain of Responsibility Pattern.
//!
//! Provides a composable middleware chain for intercepting, authorizing, mutating,
//! and conditionally suppressing engine actions, command dispatches, and hook callbacks.

/// Flow control outcome returned by each link in an [`Interceptor`] pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineFlow {
    /// Continue execution to the next interceptor in the chain.
    Continue,
    /// Execution is complete and considered handled; abort remaining interceptors.
    Handled,
    /// Abort execution and suppress the underlying action or engine event (supercede).
    Block,
}

impl PipelineFlow {
    /// Returns `true` if execution should proceed to the next interceptor.
    #[inline(always)]
    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }

    /// Returns `true` if the pipeline was halted (`Handled` or `Block`).
    #[inline(always)]
    pub fn is_halted(&self) -> bool {
        !self.is_continue()
    }

    /// Returns `true` if the pipeline explicitly blocked/suppressed the action.
    #[inline(always)]
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Block)
    }
}

/// An individual interceptor stage in a Chain of Responsibility [`Pipeline`].
pub trait Interceptor<Ctx> {
    /// Inspects and optionally mutates the given context, returning the next [`PipelineFlow`].
    fn intercept(&self, ctx: &mut Ctx) -> PipelineFlow;
}

impl<Ctx, F> Interceptor<Ctx> for F
where
    F: Fn(&mut Ctx) -> PipelineFlow,
{
    #[inline(always)]
    fn intercept(&self, ctx: &mut Ctx) -> PipelineFlow {
        self(ctx)
    }
}

/// Composable Chain of Responsibility pipeline for a specific context type `Ctx`.
pub struct Pipeline<Ctx> {
    interceptors: Vec<Box<dyn Interceptor<Ctx> + Send + Sync>>,
}

impl<Ctx> Default for Pipeline<Ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Ctx> Pipeline<Ctx> {
    /// Creates a new, empty interceptor pipeline.
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Appends an interceptor to the end of the pipeline.
    pub fn use_interceptor<I>(&mut self, interceptor: I) -> &mut Self
    where
        I: Interceptor<Ctx> + Send + Sync + 'static,
    {
        self.interceptors.push(Box::new(interceptor));
        self
    }

    /// Appends a closure-based interceptor to the end of the pipeline.
    pub fn use_fn<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&mut Ctx) -> PipelineFlow + Send + Sync + 'static,
    {
        self.interceptors.push(Box::new(f));
        self
    }

    /// Executes all interceptors in order against `ctx`.
    ///
    /// Stops and returns early if any interceptor returns [`PipelineFlow::Handled`]
    /// or [`PipelineFlow::Block`].
    pub fn execute(&self, ctx: &mut Ctx) -> PipelineFlow {
        for interceptor in &self.interceptors {
            match interceptor.intercept(ctx) {
                PipelineFlow::Continue => continue,
                halted => return halted,
            }
        }
        PipelineFlow::Continue
    }

    /// Returns the number of registered interceptors in the pipeline.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// Returns `true` if no interceptors are registered.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }

    /// Clears all registered interceptors.
    pub fn clear(&mut self) {
        self.interceptors.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        value: i32,
        log: Vec<&'static str>,
    }

    #[test]
    fn test_pipeline_continue_all() {
        let mut pipeline = Pipeline::<TestContext>::new();
        pipeline
            .use_fn(|ctx| {
                ctx.value += 10;
                ctx.log.push("step1");
                PipelineFlow::Continue
            })
            .use_fn(|ctx| {
                ctx.value *= 2;
                ctx.log.push("step2");
                PipelineFlow::Continue
            });

        let mut ctx = TestContext {
            value: 5,
            log: Vec::new(),
        };
        let flow = pipeline.execute(&mut ctx);

        assert_eq!(flow, PipelineFlow::Continue);
        assert_eq!(ctx.value, 30);
        assert_eq!(ctx.log, vec!["step1", "step2"]);
    }

    #[test]
    fn test_pipeline_handled_short_circuits() {
        let mut pipeline = Pipeline::<TestContext>::new();
        pipeline
            .use_fn(|ctx| {
                ctx.log.push("step1");
                PipelineFlow::Handled
            })
            .use_fn(|ctx| {
                ctx.log.push("step2");
                PipelineFlow::Continue
            });

        let mut ctx = TestContext {
            value: 0,
            log: Vec::new(),
        };
        let flow = pipeline.execute(&mut ctx);

        assert_eq!(flow, PipelineFlow::Handled);
        assert_eq!(ctx.log, vec!["step1"]);
    }

    #[test]
    fn test_pipeline_block_short_circuits() {
        let mut pipeline = Pipeline::<TestContext>::new();
        pipeline
            .use_fn(|ctx| {
                ctx.log.push("step1");
                PipelineFlow::Block
            })
            .use_fn(|ctx| {
                ctx.log.push("step2");
                PipelineFlow::Continue
            });

        let mut ctx = TestContext {
            value: 0,
            log: Vec::new(),
        };
        let flow = pipeline.execute(&mut ctx);

        assert_eq!(flow, PipelineFlow::Block);
        assert!(flow.is_blocked());
        assert_eq!(ctx.log, vec!["step1"]);
    }
}
