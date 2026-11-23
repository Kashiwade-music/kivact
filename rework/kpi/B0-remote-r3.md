# KPI remote (2026-11-23T04:07Z, MINGW64_NT-10.0-22631, commit e99e222d)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=42.5 samples=[ 45.0 42.6 41.0 42.5 42.5 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.8 2.7 3.0 2.8 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=39.4 samples=[ 40.0 38.6 40.7 38.8 39.4 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.6 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=208.4 samples=[ 349.5 206.9 208.4 206.9 214.7 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=22560 private_kib=7056 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=22580 private_kib=7064 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=162940 private_kib=125724 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=131.4 configure_ms=122.5]
- basic-catalog renderer=default ws_kib=39808 private_kib=20344 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=default ws_kib=97880 private_kib=86876 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=39940 private_kib=20508 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=97552 private_kib=86416 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=23176 private_kib=8216 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=33816 private_kib=20644 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.55
- item-text-0 renderer=cpu ws_kib=23180 private_kib=8216 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=31448 private_kib=18284 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.76
- item-button-0 renderer=cpu ws_kib=23244 private_kib=8292 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=78616 private_kib=75436 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.46
- item-row-0 renderer=cpu ws_kib=23200 private_kib=8236 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=97496 private_kib=86424 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.77

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3740
    [box] H site     989.9 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [box] H site     589.9 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     280.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     232.2 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     216.0 B/item     3000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [box] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [box] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [box] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [box] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
- heap mode=text live_bytes_per_item=2971
    [text] H site     989.9 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [text] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     360.6 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [text] H site     192.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
    [text] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [text] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
    [text] H site      54.6 B/item        1 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0<u32,kivact_core::view::descriptor::description::View,alloc::vec::Vec<u32,alloc::alloc::Global>,kv_heap::main::closure$0::closure_env$1>
- heap mode=button live_bytes_per_item=20517
    [button] H site    3959.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [button] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1504.0 B/item    72000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [button] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     696.2 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     560.2 B/item    12004 allocs kivact_core::view::element_data::Common::push_style
    [button] H site     416.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
- heap mode=row live_bytes_per_item=23418
    [row] H site    3959.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [row] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site     928.2 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site     928.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [row] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [row] H site     797.4 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [row] H site     772.0 B/item    24000 allocs kivact_core::view::descriptor::description::impl$7::into_view
    [row] H site     732.0 B/item    48000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=173 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=5.77 ws_kib=33152 work_ms p50=2.2472 p95=3.3442 p99=3.6697 scene_p95=0.0603 raster_p95=3.1717 interval_p99=33.494
- rows=0 renderer=cpu frames=173 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=33076 work_ms p50=1.5848 p95=3.1812 p99=3.3263 scene_p95=0.058 raster_p95=3.0031 interval_p99=34.4581
- rows=3000 renderer=default frames=173 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=106184 work_ms p50=3.2936 p95=4.1946 p99=5.0237 scene_p95=0.3141 raster_p95=3.7856 interval_p99=34.0626
- rows=3000 renderer=cpu frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=106568 work_ms p50=2.7369 p95=4.0987 p99=4.2806 scene_p95=0.3068 raster_p95=3.7179 interval_p99=34.4993

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=3.962 p50=3.962 p95=3.962 p99=3.962
- L counter_click n=200 mean=0.473 p50=0.465 p95=0.508 p99=0.684
- L mount_n100 n=1 mean=13.635 p50=13.635 p95=13.635 p99=13.635
- L unrelated_update_n100 n=50 mean=0.708 p50=0.693 p95=0.762 p99=1.248
- L row_update_n100 n=50 mean=1.298 p50=1.290 p95=1.360 p99=1.749
- L pointer_move_n100 n=120 mean=0.161 p50=0.020 p95=0.691 p99=0.832
- L wheel_n100 n=40 mean=0.008 p50=0.008 p95=0.008 p99=0.013
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 8384000, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=102.038 p50=102.038 p95=102.038 p99=102.038
- L unrelated_update_n1000 n=50 mean=0.766 p50=0.725 p95=0.841 p99=1.879
- L row_update_n1000 n=50 mean=2.075 p50=2.044 p95=2.242 p99=3.161
- L pointer_move_n1000 n=120 mean=0.366 p50=0.068 p95=1.330 p99=1.633
- L wheel_n1000 n=40 mean=0.027 p50=0.027 p95=0.031 p99=0.036
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 27048200, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=300.568 p50=300.568 p95=300.568 p99=300.568
- L unrelated_update_n3000 n=50 mean=0.887 p50=0.739 p95=2.080 p99=3.260
- L row_update_n3000 n=50 mean=4.273 p50=4.208 p95=4.550 p99=7.248
- L pointer_move_n3000 n=120 mean=0.838 p50=0.205 p95=2.558 p99=2.913
- L wheel_n3000 n=40 mean=0.101 p50=0.094 p95=0.127 p99=0.130
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 67893400, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18949120
- K6 kv_startup default 18944512
- K6 kv_startup no-default-features 11301888
- K6 basic-catalog default 34037248
- K6 kv_startup no-gpu 14356480
- K6 basic-catalog no-gpu 29464064
