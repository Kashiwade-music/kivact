# KPI CP1b (2026-11-20T08:26Z, MINGW64_NT-10.0-26200, commit ea9d0b91)

release build of kivact-example-counter (cold for its own target dir): 327 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=56.2 samples=[ 57.8 46.1 51.4 139.9 56.2 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.9 2.7 2.7 2.8 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=55.9 samples=[ 66.0 48.9 55.9 48.9 58.6 ]
- K4 first_frame_raster renderer=cpu median_ms=3.0 samples=[ 3.5 2.7 2.7 3.0 3.0 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=218.4 samples=[ 657.6 218.4 203.8 251.1 188.7 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26840 private_kib=5924 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26824 private_kib=5876 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=88620 private_kib=106332 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=665.6 configure_ms=17.3]
- basic-catalog renderer=default ws_kib=54256 private_kib=28976 idle_cpu_ms_per_s=123.257 idle_frames=639 cpu_ticks_100ns=13125000 
- rows3000-still renderer=default ws_kib=137808 private_kib=120128 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=54204 private_kib=28880 idle_cpu_ms_per_s=126.596 idle_frames=637 cpu_ticks_100ns=13437500 
- rows3000-still renderer=cpu ws_kib=136592 private_kib=120280 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=27060 private_kib=6616 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=42796 private_kib=23856 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.25
- item-text-0 renderer=cpu ws_kib=27092 private_kib=6604 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=41292 private_kib=22376 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.73
- item-button-0 renderer=cpu ws_kib=27064 private_kib=6620 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=101748 private_kib=92036 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=24.89
- item-row-0 renderer=cpu ws_kib=27100 private_kib=6648 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=136976 private_kib=120120 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=36.63

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=5278
    [box] H site    1022.7 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [box] H site     572.0 B/item     3026 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [box] H site     480.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     312.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     256.3 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [box] H site     216.0 B/item     3000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [box] H site     168.3 B/item     3013 allocs arvane_native::a11y::conversion::update
- heap mode=text live_bytes_per_item=4649
    [text] H site    1022.7 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [text] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [text] H site     667.2 B/item     9026 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [text] H site     480.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     273.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [text] H site     183.6 B/item     9013 allocs arvane_native::a11y::conversion::update
    [text] H site     160.1 B/item     6002 allocs kivact_layout::tree::pass::impl$0::leaf::closure$1::closure$2
    [text] H site     114.1 B/item       23 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
- heap mode=button live_bytes_per_item=25935
    [button] H site    4090.6 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    3113.0 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [button] H site    2523.1 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    2320.9 B/item    24026 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [button] H site    1922.4 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1660.0 B/item    75000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [button] H site     768.3 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
- heap mode=row live_bytes_per_item=22579
    [row] H site    4090.6 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    2523.1 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    1922.4 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site    1024.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1024.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     888.0 B/item    51000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [row] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site     772.0 B/item    24000 allocs kivact_core::view::descriptor::description::impl$7::into_view
    [row] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=339 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.40 ws_kib=33104 work_ms p50=1.4018 p95=1.6994 p99=1.8345 scene_p95=0.0463 raster_p95=1.5702 interval_p99=17.5212
- rows=0 renderer=cpu frames=337 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.69 ws_kib=33232 work_ms p50=1.4047 p95=1.8199 p99=1.9883 scene_p95=0.0477 raster_p95=1.7067 interval_p99=17.5437
- rows=3000 renderer=default frames=336 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=11.71 ws_kib=103804 work_ms p50=1.7075 p95=1.9089 p99=2.137 scene_p95=0.1322 raster_p95=1.7312 interval_p99=17.3699
- rows=3000 renderer=cpu frames=339 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.71 ws_kib=103852 work_ms p50=1.6957 p95=2.018 p99=2.149 scene_p95=0.1465 raster_p95=1.8325 interval_p99=17.5557

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.370 p50=6.370 p95=6.370 p99=6.370
- L counter_click n=200 mean=0.831 p50=0.811 p95=0.994 p99=1.021
- L mount_n100 n=1 mean=20.293 p50=20.293 p95=20.293 p99=20.293
- L unrelated_update_n100 n=50 mean=0.957 p50=0.944 p95=1.020 p99=1.429
- L row_update_n100 n=50 mean=1.269 p50=1.249 p95=1.483 p99=1.778
- L pointer_move_n100 n=120 mean=0.299 p50=0.036 p95=1.208 p99=1.236
- L wheel_n100 n=40 mean=0.014 p50=0.013 p95=0.019 p99=0.036
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 11176200, live_reactive: 2062, live_owners: 101, live_objects: 203, live_constants: 409, arena_slots: 2775, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=156.776 p50=156.776 p95=156.776 p99=156.776
- L unrelated_update_n1000 n=50 mean=0.972 p50=0.928 p95=1.073 p99=2.475
- L row_update_n1000 n=50 mean=2.025 p50=1.951 p95=2.221 p99=3.961
- L pointer_move_n1000 n=120 mean=0.568 p50=0.100 p95=1.970 p99=2.283
- L wheel_n1000 n=40 mean=0.048 p50=0.047 p95=0.057 p99=0.074
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 36973800, live_reactive: 20062, live_owners: 1001, live_objects: 2003, live_constants: 4009, arena_slots: 27075, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=478.926 p50=478.926 p95=478.926 p99=478.926
- L unrelated_update_n3000 n=50 mean=1.019 p50=0.933 p95=1.119 p99=4.511
- L row_update_n3000 n=50 mean=3.869 p50=3.615 p95=4.706 p99=7.970
- L pointer_move_n3000 n=120 mean=1.862 p50=0.312 p95=6.112 p99=6.310
- L wheel_n3000 n=40 mean=0.145 p50=0.139 p95=0.173 p99=0.286
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 157470800, live_reactive: 60062, live_owners: 3001, live_objects: 6003, live_constants: 12009, arena_slots: 81075, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 19180032
- K6 kv_startup default 19177472
- K6 kv_startup no-default-features 11525632
- K6 basic-catalog default 36572672
- K6 kv_startup no-gpu 14580736
- K6 basic-catalog no-gpu 31990784
