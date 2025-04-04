use core::mem::take;

use hashbrown::HashMap;

use crate::hook::{hookify, BoxedHook, HookEnv, HookRegistry};

/// Context to run a program inside SP1.
#[derive(Clone, Default)]
pub struct SP1Context<'a> {
    /// The registry of hooks invocable from inside SP1.
    ///
    /// Note: `None` denotes the default list of hooks.
    pub hook_registry: Option<HookRegistry<'a>>,

    /// The maximum number of cpu cycles to use for execution.
    pub max_cycles: Option<u64>,
}

/// A builder for [`SP1Context`].
#[derive(Clone, Default)]
pub struct SP1ContextBuilder<'a> {
    no_default_hooks: bool,
    hook_registry_entries: Vec<(u32, BoxedHook<'a>)>,
    max_cycles: Option<u64>,
}

impl<'a> SP1Context<'a> {
    /// Create a new context builder. See [`SP1ContextBuilder`] for more details.
    #[must_use]
    pub fn builder() -> SP1ContextBuilder<'a> {
        SP1ContextBuilder::new()
    }
}

impl<'a> SP1ContextBuilder<'a> {
    /// Create a new [`SP1ContextBuilder`].
    ///
    /// Prefer using [`SP1Context::builder`].
    #[must_use]
    pub fn new() -> Self {
        SP1ContextBuilder::default()
    }

    /// Build and return the [`SP1Context`].
    ///
    /// Clears and resets the builder, allowing it to be reused.
    pub fn build(&mut self) -> SP1Context<'a> {
        // If hook_registry_entries is nonempty or no_default_hooks true,
        // indicating a non-default value of hook_registry.
        let hook_registry =
            (!self.hook_registry_entries.is_empty() || self.no_default_hooks).then(|| {
                let mut table = if take(&mut self.no_default_hooks) {
                    HashMap::default()
                } else {
                    HookRegistry::default().table
                };
                // Allows overwriting default hooks.
                table.extend(take(&mut self.hook_registry_entries));
                HookRegistry { table }
            });
        let cycle_limit = take(&mut self.max_cycles);
        SP1Context {
            hook_registry,
            max_cycles: cycle_limit,
        }
    }

    /// Add a runtime [Hook](super::Hook) into the context.
    ///
    /// Hooks may be invoked from within SP1 by writing to the specified file descriptor `fd`
    /// with [`sp1_zkvm::io::write`], returning a list of arbitrary data that may be read
    /// with successive calls to [`sp1_zkvm::io::read`].
    pub fn hook(
        &mut self,
        fd: u32,
        f: impl FnMut(HookEnv, &[u8]) -> Vec<Vec<u8>> + Send + Sync + 'a,
    ) -> &mut Self {
        self.hook_registry_entries.push((fd, hookify(f)));
        self
    }

    /// Avoid registering the default hooks in the runtime.
    ///
    /// It is not necessary to call this to override hooks --- instead, simply
    /// register a hook with the same value of `fd` by calling [`Self::hook`].
    pub fn without_default_hooks(&mut self) -> &mut Self {
        self.no_default_hooks = true;
        self
    }

    /// Set the maximum number of cpu cycles to use for execution.
    pub fn max_cycles(&mut self, max_cycles: u64) -> &mut Self {
        self.max_cycles = Some(max_cycles);
        self
    }
}
