use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::sync::{Mutex, RwLock};

#[cfg(feature = "render-debug")]
use std::collections::btree_map;
#[cfg(feature = "render-debug")]
use std::sync::Arc;

/// Generates legion system setup procedure for.
///
/// Consider this example:
///
/// ```
/// struct FooEvent(f32);
/// struct BarEvent(f32);
///
/// struct QuxComp(u32);
/// struct CorgeComp(u32);
///
/// #[derive(Default)]
/// struct GraultRes(u64);
/// #[derive(Default)]
/// struct WaldoRes(u64);
///
/// #[codegen::system]
/// #[read_component(QuxComp)]
/// #[write_component(CorgeComp)]
/// fn example(
///     world: &mut legion::world::SubWorld,
///     #[subscriber] foo_sub: impl Iterator<Item = FooEvent>,
///     #[publisher] bar_pub: impl FnMut(BarEvent),
///     #[resource] grault_res: &mut GraultRes,
///     #[resource] waldo_res: &WaldoRes,
///     #[state(0)] local_counter: &mut i32,
/// ) {
///     use legion::IntoQuery;
///
///     for (qux, corge) in <(&QuxComp, &mut CorgeComp)>::query().iter_mut(world) {
///         corge.0 = qux.0;
///     }
///
///     for &FooEvent(float) in foo_sub {
///         bar_pub(BarEvent(float));
///     }
///
///     grault_res.0 = waldo_res.0;
///
///     *local_counter += 1;
/// }
///
/// fn setup_ecs(setup: codegen::SetupEcs) -> codegen::SetupEcs {
///     setup.uses(example_setup)
/// }
/// ```
///
/// If some of the parameters need to be thread-unsafe,
/// apply the `#[thread_local]` attribute on the function.
pub use traffloat_codegen_raw::system;

/// Implements network layer serialization.
pub use traffloat_codegen_raw::Gen;

/// Whether debug info should be rendered.
pub const RENDER_DEBUG: bool = cfg!(feature = "render-debug");

/// The standard setup parameters
#[derive(Default)]
pub struct SetupEcs {
    /// Whether to enable server-only systems
    pub server: bool,
    /// The legion::Scheduler builder
    pub builder: legion::systems::Builder,
    /// The legion world storing entities and components
    pub world: legion::World,
    /// The resource set storing legion resources
    pub resources: legion::Resources,
}

impl SetupEcs {
    /// Register a bundle
    pub fn uses(self, setup_ecs: impl FnOnce(Self) -> Self) -> Self {
        setup_ecs(self)
    }

    /// Add a system
    pub fn system(mut self, sys: impl legion::systems::ParallelRunnable + 'static) -> Self {
        self.builder.add_system(sys);
        self
    }
    /// Add a thread-local system
    pub fn system_local(mut self, sys: impl legion::systems::Runnable + 'static) -> Self {
        self.builder.add_thread_local(sys);
        self
    }

    /// Add an entity
    pub fn entity<T>(mut self, components: T) -> Self
    where
        Option<T>: legion::storage::IntoComponentSource,
    {
        self.world.push(components);
        self
    }
    /// Add entities
    pub fn entities<T>(mut self, components: impl legion::storage::IntoComponentSource) -> Self {
        self.world.extend(components);
        self
    }

    /// Add a resource
    pub fn resource(mut self, res: impl legion::systems::Resource) -> Self {
        self.resources.get_or_insert(res);
        self
    }
    /// Add a default resource
    pub fn resource_default<T: legion::systems::Resource + Default>(mut self) -> Self {
        self.resources.get_or_default::<T>();
        self
    }
    /// Declare a published event
    pub fn publish<T: shrev::Event>(mut self) -> Self {
        let _ = self
            .resources
            .get_or_insert_with(shrev::EventChannel::<T>::new);
        self
    }
    /// Declare a subscribed event
    pub fn subscribe<T: shrev::Event>(&mut self) -> shrev::ReaderId<T> {
        let mut channel = self
            .resources
            .get_mut_or_insert_with(shrev::EventChannel::<T>::new);
        channel.register_reader()
    }

    /// Build the setup into a legion
    pub fn build(mut self) -> Legion {
        Legion {
            world: self.world,
            resources: self.resources,
            schedule: self.builder.build(),
        }
    }
}

/// The set of values required to run legion
pub struct Legion {
    /// The legion world storing entities and components
    pub world: legion::World,
    /// The resource set storing legion resources
    pub resources: legion::Resources,
    /// The legion scheduler running systems
    pub schedule: legion::Schedule,
}

impl Legion {
    /// Spins all systems once.
    pub fn run(&mut self) {
        self.schedule.execute(&mut self.world, &mut self.resources)
    }

    /// Publishes an event.
    pub fn publish<T: shrev::Event>(&mut self, event: T) {
        let mut channel = match self.resources.get_mut::<shrev::EventChannel<T>>() {
            Some(channel) => channel,
            None => panic!(
                "EventChannel<{}> has not been initialized",
                std::any::type_name::<T>()
            ),
        };
        channel.single_write(event);
    }
}

/// Performance tracking
#[derive(Default)]
pub struct Perf {
    pub map: PerfMap,
}

pub type PerfMap = RwLock<BTreeMap<&'static str, Mutex<VecDeque<i64>>>>;

const MAX_FRAMES: usize = 100;

impl Perf {
    /// Update a timer
    pub fn push(&self, name: &'static str, value: i64) {
        loop {
            {
                let map = self.map.read().expect("Perf poisoned");
                if let Some(deque) = map.get(name) {
                    let mut deque = deque.lock().expect("Perf poisoned");
                    while deque.len() >= MAX_FRAMES {
                        deque.pop_front();
                    }
                    deque.push_back(value);
                    return;
                }
            }

            {
                let mut map = self.map.write().expect("Perf poisoned");
                let _ = map.entry(name).or_default();
            }
        }
    }
}

/// The resource storing debug entries to render.
#[cfg(feature = "render-debug")]
#[derive(Default, getset::Getters)]
pub struct DebugEntries {
    #[getset(get = "pub")]
    /// Entries in the format `entries[category][name]`.
    entries: BTreeMap<&'static str, BTreeMap<&'static str, DebugEntry>>,
}

#[cfg(feature = "render-debug")]
impl DebugEntries {
    /// Creates a new entry.
    pub fn entry(&mut self, category: &'static str, name: &'static str) -> DebugEntry {
        let entries = self.entries.entry(category).or_default();
        match entries.entry(name) {
            btree_map::Entry::Occupied(_) => panic!("Duplicate debug entry {}/{}", category, name),
            btree_map::Entry::Vacant(entry) => entry.insert(DebugEntry::default()).clone(),
        }
    }
}

/// The value of a debug entry.
#[cfg(feature = "render-debug")]
#[derive(Debug, Clone, Default)]
pub struct DebugEntry {
    value: Arc<Mutex<String>>,
}

/// Updates a debug entry.
///
/// Example:
/// ```n_run
/// use codegen::{DebugEntry, update_debug};
/// # fn get_entry() -> &'static mut DebugEntry{
///     unimplemented!()
/// # }
/// let pi_entry: &mut DebugEntry = get_entry();
/// update_debug!(pi_entry, "{:.1}", std::f32::consts::PI);
/// ```
#[macro_export]
macro_rules! update_debug {
    ($entry:expr, $lit:literal $($tt:tt)*) => {
        if cfg!(feature = "render-debug") {
            $entry._update(std::format_args!($lit $($tt)*));
        }
    }
}

#[cfg(feature = "render-debug")]
impl DebugEntry {
    /// Updates the debug entry.
    pub fn _update(&self, new: impl fmt::Display) {
        use fmt::Write;

        let mut value = self.value.lock().expect("Poisoned debug entry");
        value.clear();
        write!(value, "{}", new).expect("String::write_fmt never fails");
    }

    /// Returns the value as a str
    pub fn value(&self) -> impl AsRef<str> + '_ {
        use std::sync::MutexGuard;

        struct MutexStr<'t>(MutexGuard<'t, String>);
        impl<'t> AsRef<str> for MutexStr<'t> {
            fn as_ref(&self) -> &str {
                self.0.as_str()
            }
        }
        let value = self.value.lock().expect("Poisoned debug entry");
        MutexStr(value)
    }
}

/// Dummy struct for debug entry in non-render-debug builds.
#[cfg(not(feature = "render-debug"))]
// #[derive(Debug, Clone, Default)]
pub struct DebugEntry(pub ());

#[cfg(not(feature = "render-debug"))]
impl DebugEntry {
    /// Dummy method for debug entry in non-render-debug builds.
    pub fn _update(&self, _new: impl fmt::Display) {
        unimplemented!()
    }
}

/// The high-resolution clock in microseconds
#[cfg(target_arch = "wasm32")]
pub fn hrtime() -> i64 {
    (web_sys::window()
        .expect("Window uninitialized")
        .performance()
        .expect("window.performance uninitialized")
        .now()
        * 1000.) as i64
}

/// The high-resolution clock in microseconds
#[cfg(not(target_arch = "wasm32"))]
pub fn hrtime() -> i64 {
    use std::time::Instant;

    lazy_static::lazy_static! {
        static ref EPOCH: Instant = Instant::now();
    }

    EPOCH.elapsed().as_micros() as i64
}
