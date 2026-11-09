# KPI analysis-machine (2026-11-09T10:53Z, MINGW64_NT-10.0-22631, commit 31a7ebe)

## K1 startup (main -> first presented frame, 800x600, median of 5)
- renderer=default startup_ms median=442 samples=[ 454 442 447 437 439 ]
- renderer=cpu startup_ms median=437 samples=[ 469 428 447 427 437 ]

## K2/K3 memory and CPU (idle 3 s, then 10 s sampled)
- counter renderer=default ws_mib=183 private_mib=136 idle_cpu_ms_per_s=0 idle_frames=0
- basic-catalog renderer=default ws_mib=410 private_mib=291 idle_cpu_ms_per_s=1 idle_frames=0
- rows3000-still renderer=default ws_mib=620 private_mib=636 idle_cpu_ms_per_s=0 idle_frames=0
- counter renderer=cpu ws_mib=40 private_mib=22 idle_cpu_ms_per_s=0 idle_frames=0
- basic-catalog renderer=cpu ws_mib=176 private_mib=117 idle_cpu_ms_per_s=0 idle_frames=0
- rows3000-still renderer=cpu ws_mib=473 private_mib=539 idle_cpu_ms_per_s=0 idle_frames=0

## K2 memory per item (kv_heap, 3000 items, live bytes / item of the top sites)
  [box] H total_mib 103.1
  [box] H site    8112.1 B/item     6006 allocs kivact_core::view::mount::resolve::resolve
  [box] H site    6032.0 B/item     3004 allocs kivact_engine::model::aggregate::Model::set_boxed_prop
  [box] H site    4147.2 B/item        2 allocs arvane_compose::cpu::surface::Pixmap::new
  [box] H site    2093.1 B/item      550 allocs arvane_render::cpu::CpuRaster::raster
  [box] H site    1945.3 B/item     3002 allocs kivact_runtime::reactive::props::Prop::into_fn<kivact_style::sheet::Style>
  [box] H site    1714.9 B/item        4 allocs kivact_layout::columns::NodeLayoutColumns::push
  [box] H site    1473.5 B/item     6006 allocs kivact_engine::model::aggregate::Model::set_boxed_prop<kivact_types::layout::LayoutProps,kivact_engine::host::write::typed::impl$3::set_layout::closure$0::closure_env$0>
  [box] H site    1310.7 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<kivact_engine::contract::events::HostEvent>,alloc::alloc::Global>
  [box] H site    1288.9 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<core::option::Option<tuple$<kivact_types::ids::NodeId,kivact_engine::tree::node::HostNode> > >,alloc::alloc::Global>
  [box] H site     874.0 B/item        2 allocs alloc::raw_vec::RawVec::grow_one<kivact_core::view::semantics_data::SemanticsPatch,alloc::alloc::Global>
  [box] H site     720.9 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<kivact_engine::model::drawables::place::Frame>,alloc::alloc::Global>
  [text] H total_mib 52.7
  [text] H site    4147.2 B/item        2 allocs arvane_compose::cpu::surface::Pixmap::new
  [text] H site    2104.9 B/item      550 allocs arvane_render::cpu::CpuRaster::raster
  [text] H site    1714.9 B/item        4 allocs kivact_layout::columns::NodeLayoutColumns::push
  [text] H site    1310.7 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<kivact_engine::contract::events::HostEvent>,alloc::alloc::Global>
  [text] H site    1288.9 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<core::option::Option<tuple$<kivact_types::ids::NodeId,kivact_engine::tree::node::HostNode> > >,alloc::alloc::Global>
  [text] H site     737.5 B/item     3006 allocs kivact_engine::model::aggregate::Model::set_boxed_prop<kivact_types::layout::LayoutProps,kivact_engine::host::write::typed::impl$3::set_layout::closure$0::closure_env$0>
  [text] H site     720.9 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<kivact_engine::model::drawables::place::Frame>,alloc::alloc::Global>
  [text] H site     636.3 B/item        1 allocs hashbrown::raw::RawTable::reserve_rehash<tuple$<kivact_types::ids::NodeId,kivact_core::view::tree::CoreNode>,alloc::alloc::Global,hashbrown::map::make_hasher::closure_env$0<kivact_types::ids::NodeId,kivact_core::view::tree::CoreNode,core::hash::BuildHasher
  [text] H site     520.2 B/item     3001 allocs kivact_engine::measure::leaf::Measure::node_text
  [text] H site     480.5 B/item     3003 allocs kivact_engine::measure::inherited_text::computed_text
  [text] H site     416.3 B/item     3002 allocs kivact_engine::model::aggregate::Model::set_boxed_prop<kivact_types::semantics::node::Semantics<kivact_types::ids::FocusTarget>,kivact_engine::host::write::typed::impl$3::set_semantics::closure$0::closure_env$0>
  [button] H total_mib 347.7
  [button] H site   18080.0 B/item     9004 allocs kivact_engine::model::aggregate::Model::set_boxed_prop
  [button] H site   16216.1 B/item    12006 allocs kivact_core::view::mount::resolve::resolve
  [button] H site   13981.2 B/item        2 allocs alloc::raw_vec::RawVec::grow_one<kivact_core::view::semantics_data::SemanticsPatch,alloc::alloc::Global>
  [button] H site    6859.4 B/item        4 allocs kivact_layout::columns::NodeLayoutColumns::push
  [button] H site    6625.5 B/item    27006 allocs kivact_engine::model::aggregate::Model::set_boxed_prop<kivact_types::layout::LayoutProps,kivact_engine::host::write::typed::impl$3::set_layout::closure$0::closure_env$0>
  [button] H site    5833.3 B/item     9002 allocs kivact_runtime::reactive::props::Prop::into_fn<kivact_style::sheet::Style>
  [button] H site    5242.9 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<kivact_engine::contract::events::HostEvent>,alloc::alloc::Global>
  [button] H site    5155.5 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<core::option::Option<tuple$<kivact_types::ids::NodeId,kivact_engine::tree::node::HostNode> > >,alloc::alloc::Global>
  [button] H site    4147.2 B/item        2 allocs arvane_compose::cpu::surface::Pixmap::new
  [button] H site    2883.6 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<kivact_engine::model::drawables::place::Frame>,alloc::alloc::Global>
  [button] H site    2774.0 B/item   150035 allocs alloc::raw_vec::RawVec::grow_one<kivact_runtime::reactive::data::Signal<enum2$<kivact_core::view::ref_data::RefState> >,alloc::alloc::Global>
  [row] H total_mib 495.6
  [row] H site   24320.1 B/item    18006 allocs kivact_core::view::mount::resolve::resolve
  [row] H site   24104.0 B/item    12004 allocs kivact_engine::model::aggregate::Model::set_boxed_prop
  [row] H site   13981.2 B/item        2 allocs alloc::raw_vec::RawVec::grow_one<kivact_core::view::semantics_data::SemanticsPatch,alloc::alloc::Global>
  [row] H site   13718.9 B/item        4 allocs kivact_layout::columns::NodeLayoutColumns::push
  [row] H site   10485.8 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<kivact_engine::contract::events::HostEvent>,alloc::alloc::Global>
  [row] H site   10311.0 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<core::option::Option<tuple$<kivact_types::ids::NodeId,kivact_engine::tree::node::HostNode> > >,alloc::alloc::Global>
  [row] H site    8097.5 B/item    33006 allocs kivact_engine::model::aggregate::Model::set_boxed_prop<kivact_types::layout::LayoutProps,kivact_engine::host::write::typed::impl$3::set_layout::closure$0::closure_env$0>
  [row] H site    7777.3 B/item    12002 allocs kivact_runtime::reactive::props::Prop::into_fn<kivact_style::sheet::Style>
  [row] H site    5767.2 B/item        1 allocs alloc::raw_vec::RawVec::grow_one<enum2$<kivact_engine::model::drawables::place::Frame>,alloc::alloc::Global>
  [row] H site    5090.0 B/item        1 allocs hashbrown::raw::RawTable::reserve_rehash<tuple$<kivact_types::ids::NodeId,kivact_core::view::tree::CoreNode>,alloc::alloc::Global,hashbrown::map::make_hasher::closure_env$0<kivact_types::ids::NodeId,kivact_core::view::tree::CoreNode,core::hash::BuildHasher
  [row] H site    4147.2 B/item        2 allocs arvane_compose::cpu::surface::Pixmap::new

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default fps=35 cpu_pct_of_core=1 ws_mib=202 work_ms p50=29.9651 p95=30.7359 p99=30.8949 scene_p95=0.0766 raster_p95=2.5053 interval_p99=31.4145
- rows=0 renderer=cpu fps=64 cpu_pct_of_core=6 ws_mib=47 work_ms p50=3.1216 p95=6.1104 p99=7.2441 scene_p95=0.0563 raster_p95=5.9666 interval_p99=20.4447
- rows=3000 renderer=default fps=35 cpu_pct_of_core=5 ws_mib=610 work_ms p50=29.2296 p95=29.9315 p99=30.2591 scene_p95=11.0129 raster_p95=1.3301 interval_p99=31.5453
- rows=3000 renderer=cpu fps=64 cpu_pct_of_core=8 ws_mib=472 work_ms p50=8.5379 p95=13.555 p99=16.1169 scene_p95=10.2767 raster_p95=4.1355 interval_p99=22.7699

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=308.324 p50=308.324 p95=308.324 p99=308.324
- L counter_click n=200 mean=1.496 p50=1.474 p95=1.600 p99=1.981
- L mount_n100 n=1 mean=324.360 p50=324.360 p95=324.360 p99=324.360
- L unrelated_update_n100 n=50 mean=2.711 p50=2.667 p95=2.835 p99=4.821
- L row_update_n100 n=50 mean=11.365 p50=10.002 p95=32.490 p99=32.897
- L pointer_move_n100 n=120 mean=0.881 p50=0.306 p95=2.940 p99=3.198
- L wheel_n100 n=40 mean=0.243 p50=0.239 p95=0.282 p99=0.325
- S stats_n100 Stats { batches: 101, a11y_constructed: 2270, a11y_published: 1573, ops: 2853, last_batch_ops: 1, creates: 1223, set_props: 5609, structural: 1222, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 14627300, live_reactive: 5750, live_owners: 101, live_objects: 409, live_constants: 612, arena_slots: 6872, tree_nodes: 1223, timers: 0, frame_loops: 102, animations: 0 }
- L mount_n1000 n=1 mean=523.851 p50=523.851 p95=523.851 p99=523.851
- L unrelated_update_n1000 n=50 mean=12.708 p50=12.543 p95=13.800 p99=17.868
- L row_update_n1000 n=50 mean=24.896 p50=23.933 p95=47.667 p99=49.793
- L pointer_move_n1000 n=120 mean=6.432 p50=3.482 p95=16.863 p99=18.289
- L wheel_n1000 n=40 mean=2.942 p50=2.942 p95=3.212 p99=3.222
- S stats_n1000 Stats { batches: 101, a11y_constructed: 13070, a11y_published: 12373, ops: 27153, last_batch_ops: 1, creates: 12023, set_props: 54209, structural: 12022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 79065500, live_reactive: 56150, live_owners: 1001, live_objects: 4009, live_constants: 6012, arena_slots: 67172, tree_nodes: 12023, timers: 0, frame_loops: 1002, animations: 0 }
- L mount_n3000 n=1 mean=1007.004 p50=1007.004 p95=1007.004 p99=1007.004
- L unrelated_update_n3000 n=50 mean=44.547 p50=44.169 p95=47.520 p99=53.975
- L row_update_n3000 n=50 mean=65.886 p50=62.688 p95=84.648 p99=85.165
- L pointer_move_n3000 n=120 mean=24.649 p50=13.479 p95=62.771 p99=65.572
- L wheel_n3000 n=40 mean=12.200 p50=12.109 p95=13.594 p99=14.846
- S stats_n3000 Stats { batches: 101, a11y_constructed: 37070, a11y_published: 36373, ops: 81153, last_batch_ops: 1, creates: 36023, set_props: 162209, structural: 36022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 283847900, live_reactive: 168150, live_owners: 3001, live_objects: 12009, live_constants: 18012, arena_slots: 201172, tree_nodes: 36023, timers: 0, frame_loops: 3002, animations: 0 }

## K6 release binary size (bytes)
- kivact-example-counter 24592896
- kivact-ui-basic-catalog 54239232
