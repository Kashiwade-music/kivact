# KPI B0-CP3b (2026-11-22T12:58Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 7 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=53.7 samples=[ 53.7 54.7 52.5 55.1 53.0 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.8 3.0 3.1 2.7 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=53.9 samples=[ 57.3 53.9 52.6 50.6 57.4 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 2.8 2.7 2.7 2.8 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=209.3 samples=[ 582.6 209.3 188.9 212.1 196.1 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26888 private_kib=5980 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26900 private_kib=5996 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=86580 private_kib=103072 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=789.4 configure_ms=16.2]
- basic-catalog renderer=default ws_kib=54116 private_kib=29012 idle_cpu_ms_per_s=102.960 idle_frames=637 cpu_ticks_100ns=10937500 
- rows3000-still renderer=default ws_kib=138516 private_kib=122504 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=54772 private_kib=29080 idle_cpu_ms_per_s=84.060 idle_frames=636 cpu_ticks_100ns=8906250 
- rows3000-still renderer=cpu ws_kib=138372 private_kib=122480 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=27056 private_kib=6680 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=43140 private_kib=24800 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.36
- item-text-0 renderer=cpu ws_kib=27088 private_kib=6720 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=41572 private_kib=22896 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.83
- item-button-0 renderer=cpu ws_kib=27056 private_kib=6672 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=102616 private_kib=93928 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=25.19
- item-row-0 renderer=cpu ws_kib=27056 private_kib=6688 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=139148 private_kib=123124 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=37.36

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=5418
    [box] H site     989.9 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [box] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [box] H site     589.9 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     571.9 B/item     3025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [box] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     280.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     232.2 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     216.0 B/item     3000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [box] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [box] H site     168.3 B/item     3013 allocs arvane_native::a11y::conversion::update
- heap mode=text live_bytes_per_item=4754
    [text] H site     989.9 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [text] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [text] H site     667.2 B/item     9025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [text] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     360.6 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [text] H site     192.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
    [text] H site     183.6 B/item     9013 allocs arvane_native::a11y::conversion::update
    [text] H site     114.0 B/item       22 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
- heap mode=button live_bytes_per_item=26739
    [button] H site    3959.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [button] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    3113.0 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [button] H site    2320.9 B/item    24025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [button] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1504.0 B/item    72000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [button] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     696.2 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
- heap mode=row live_bytes_per_item=35826
    [row] H site    6225.9 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [row] H site    4525.5 B/item    39025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [row] H site    3959.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [row] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site     928.2 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site     928.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     858.6 B/item    39013 allocs arvane_native::a11y::conversion::update
    [row] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [row] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=338 fps=60.2 frame_period_ns=16666666 cpu_pct_of_core=9.19 ws_kib=34696 work_ms p50=1.15 p95=1.6135 p99=1.9092 scene_p95=0.0476 raster_p95=1.4811 interval_p99=17.559
- rows=0 renderer=cpu frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.26 ws_kib=34680 work_ms p50=1.1376 p95=1.59 p99=1.657 scene_p95=0.0514 raster_p95=1.4769 interval_p99=17.6635
- rows=3000 renderer=default frames=339 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=13.55 ws_kib=144492 work_ms p50=1.6154 p95=2.2314 p99=2.4574 scene_p95=0.234 raster_p95=1.9384 interval_p99=17.76
- rows=3000 renderer=cpu frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.49 ws_kib=144040 work_ms p50=1.5808 p95=2.2734 p99=2.4749 scene_p95=0.236 raster_p95=1.9499 interval_p99=17.7088

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.095 p50=5.095 p95=5.095 p99=5.095
- L counter_click n=200 mean=0.873 p50=0.866 p95=0.912 p99=1.014
- L mount_n100 n=1 mean=21.378 p50=21.378 p95=21.378 p99=21.378
- L unrelated_update_n100 n=50 mean=1.216 p50=1.194 p95=1.272 p99=1.899
- L row_update_n100 n=50 mean=2.149 p50=2.137 p95=2.256 p99=2.928
- L pointer_move_n100 n=120 mean=0.314 p50=0.035 p95=1.232 p99=1.408
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.016 p99=0.029
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13834500, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=176.378 p50=176.378 p95=176.378 p99=176.378
- L unrelated_update_n1000 n=50 mean=1.242 p50=1.202 p95=1.234 p99=2.951
- L row_update_n1000 n=50 mean=3.453 p50=3.412 p95=3.711 p99=5.612
- L pointer_move_n1000 n=120 mean=0.656 p50=0.100 p95=2.376 p99=2.596
- L wheel_n1000 n=40 mean=0.048 p50=0.047 p95=0.055 p99=0.060
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 49660200, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=529.112 p50=529.112 p95=529.112 p99=529.112
- L unrelated_update_n3000 n=50 mean=1.308 p50=1.225 p95=1.272 p99=5.121
- L row_update_n3000 n=50 mean=8.047 p50=7.994 p95=8.665 p99=12.145
- L pointer_move_n3000 n=120 mean=1.895 p50=0.294 p95=6.166 p99=6.340
- L wheel_n3000 n=40 mean=0.143 p50=0.138 p95=0.182 p99=0.225
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 179010600, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
