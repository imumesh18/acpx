use std::{fmt, future::Future, pin::Pin, rc::Rc};

/// A small boxed future alias for runtime-neutral public APIs.
pub type Task<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// A boxed local task intended for background ACP work.
pub type LocalTask<T = ()> = Task<'static, T>;

/// Runtime hooks required by `acpx`.
///
/// The ACP SDK spawns local `!Send` futures, so `acpx` makes that contract
/// explicit instead of hiding it behind runtime-specific types.
#[derive(Clone)]
pub struct RuntimeContext {
    spawn_local: Rc<dyn Fn(LocalTask)>,
}

impl RuntimeContext {
    /// Creates a runtime context from a local task spawner.
    pub fn new(spawn_local: impl Fn(LocalTask) + 'static) -> Self {
        Self {
            spawn_local: Rc::new(spawn_local),
        }
    }

    /// Spawns a local task onto the caller-provided runtime.
    pub fn spawn_local(&self, task: LocalTask) {
        (self.spawn_local)(task);
    }

    /// Boxes and spawns a local future.
    pub fn spawn(&self, task: impl Future<Output = ()> + 'static) {
        self.spawn_local(Box::pin(task));
    }
}

impl fmt::Debug for RuntimeContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeContext")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use futures::executor::block_on;

    use super::RuntimeContext;

    #[test]
    fn runtime_context_delegates_spawned_tasks() {
        let ran = Rc::new(Cell::new(false));
        let marker = Rc::clone(&ran);
        let runtime = RuntimeContext::new(move |task| {
            block_on(task);
            marker.set(true);
        });

        runtime.spawn(async {});

        assert!(ran.get());
    }

    #[test]
    fn runtime_context_debug_name_is_stable() {
        let runtime = RuntimeContext::new(|_| {});
        assert_eq!(format!("{runtime:?}"), "RuntimeContext { .. }",);
    }
}
