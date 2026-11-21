# KPI --no-catalog (2026-11-21T12:08Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 300 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=51.7 samples=[ 51.7 58.4 50.6 50.0 54.6 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.9 2.7 2.8 3.0 2.8 ]
- K1 main_to_first_present renderer=cpu median_ms=56.0 samples=[ 54.2 55.6 56.0 68.6 56.8 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.8 2.7 2.7 2.6 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=180.2 samples=[ 561.1 180.2 173.8 175.9 180.3 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26428 private_kib=5984 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26392 private_kib=5936 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=86080 private_kib=103644 idle_cpu_ms_per_s=2.971 idle_frames=0 cpu_ticks_100ns=312500 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=640.5 configure_ms=16.2]
- basic-catalog renderer=default ws_kib=53616 private_kib=28868 idle_cpu_ms_per_s=69.623 idle_frames=633 cpu_ticks_100ns=7343750 
- rows3000-still renderer=default ws_kib=139100 private_kib=122672 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=53592 private_kib=28872 idle_cpu_ms_per_s=71.107 idle_frames=633 cpu_ticks_100ns=7500000 
- rows3000-still renderer=cpu ws_kib=138220 private_kib=122772 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=26616 private_kib=6648 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=42472 private_kib=24340 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.29
- item-text-0 renderer=cpu ws_kib=26640 private_kib=6656 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=40944 private_kib=22704 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.77
- item-button-0 renderer=cpu ws_kib=26656 private_kib=6684 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=101764 private_kib=93512 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=25.04
- item-row-0 renderer=cpu ws_kib=26600 private_kib=6632 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=137920 private_kib=122512 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=37.11

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
    [text] H site     114.1 B/item       23 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
- heap mode=button live_bytes_per_item=26774
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
- heap mode=row live_bytes_per_item=35756
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
- rows=0 renderer=default frames=335 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.23 ws_kib=34208 work_ms p50=1.0582 p95=1.4559 p99=1.6103 scene_p95=0.0384 raster_p95=1.3869 interval_p99=17.417
- rows=0 renderer=cpu frames=334 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.86 ws_kib=34168 work_ms p50=1.0583 p95=1.5063 p99=1.5832 scene_p95=0.0366 raster_p95=1.4423 interval_p99=17.3522
- rows=3000 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.66 ws_kib=143376 work_ms p50=1.4744 p95=1.8827 p99=2.2001 scene_p95=0.2148 raster_p95=1.6857 interval_p99=17.4543
- rows=3000 renderer=cpu frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.54 ws_kib=142548 work_ms p50=1.4815 p95=1.9735 p99=2.235 scene_p95=0.2032 raster_p95=1.748 interval_p99=17.3581

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.321 p50=6.321 p95=6.321 p99=6.321
- L counter_click n=200 mean=0.870 p50=0.862 p95=0.906 p99=0.968
- L mount_n100 n=1 mean=22.324 p50=22.324 p95=22.324 p99=22.324
- L unrelated_update_n100 n=50 mean=1.217 p50=1.199 p95=1.240 p99=1.927
- L row_update_n100 n=50 mean=2.126 p50=2.109 p95=2.279 p99=2.889
- L pointer_move_n100 n=120 mean=0.308 p50=0.033 p95=1.252 p99=1.370
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.015 p99=0.036
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13602500, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=178.179 p50=178.179 p95=178.179 p99=178.179
- L unrelated_update_n1000 n=50 mean=1.263 p50=1.210 p95=1.454 p99=2.918
- L row_update_n1000 n=50 mean=3.393 p50=3.356 p95=3.591 p99=5.514
- L pointer_move_n1000 n=120 mean=0.616 p50=0.097 p95=2.174 p99=2.272
- L wheel_n1000 n=40 mean=0.048 p50=0.047 p95=0.053 p99=0.054
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 44507400, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=534.070 p50=534.070 p95=534.070 p99=534.070
- L unrelated_update_n3000 n=50 mean=1.458 p50=1.348 p95=1.608 p99=5.838
- L row_update_n3000 n=50 mean=7.801 p50=7.617 p95=8.545 p99=12.452
- L pointer_move_n3000 n=120 mean=1.489 p50=0.266 p95=4.844 p99=5.022
- L wheel_n3000 n=40 mean=0.136 p50=0.134 p95=0.141 p99=0.149
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 138531600, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
