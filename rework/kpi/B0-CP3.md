# KPI B0-CP3 (2026-11-22T09:33Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 6 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=58.4 samples=[ 59.7 58.4 54.3 56.5 63.9 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 3.5 2.8 3.1 2.7 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=53.0 samples=[ 52.7 54.8 53.0 57.3 51.7 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 3.0 2.8 2.7 2.8 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=188.6 samples=[ 554.4 188.5 188.6 205.7 185.8 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25672 private_kib=5912 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25696 private_kib=5904 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=87096 private_kib=103244 idle_cpu_ms_per_s=5.882 idle_frames=0 cpu_ticks_100ns=625000 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=829.3 configure_ms=27.0]
- basic-catalog renderer=default ws_kib=54204 private_kib=29000 idle_cpu_ms_per_s=104.430 idle_frames=637 cpu_ticks_100ns=11093750 
- rows3000-still renderer=default ws_kib=139540 private_kib=122780 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=54276 private_kib=29124 idle_cpu_ms_per_s=117.019 idle_frames=641 cpu_ticks_100ns=12500000 
- rows3000-still renderer=cpu ws_kib=138752 private_kib=122740 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=27084 private_kib=6700 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=43192 private_kib=24424 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.37
- item-text-0 renderer=cpu ws_kib=27120 private_kib=6732 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=41640 private_kib=22672 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.84
- item-button-0 renderer=cpu ws_kib=27068 private_kib=6676 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=101316 private_kib=93960 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=24.75
- item-row-0 renderer=cpu ws_kib=27088 private_kib=6716 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=138140 private_kib=122200 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=37.02

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
- rows=0 renderer=default frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.38 ws_kib=34704 work_ms p50=1.1113 p95=1.5851 p99=1.7374 scene_p95=0.0481 raster_p95=1.4789 interval_p99=17.6172
- rows=0 renderer=cpu frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.87 ws_kib=34684 work_ms p50=1.5638 p95=2.9251 p99=3.1976 scene_p95=0.0569 raster_p95=1.5385 interval_p99=18.3757
- rows=3000 renderer=default frames=337 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=14.20 ws_kib=143860 work_ms p50=2.0382 p95=3.317 p99=4.178 scene_p95=0.2399 raster_p95=1.9349 interval_p99=18.6033
- rows=3000 renderer=cpu frames=336 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=13.39 ws_kib=141380 work_ms p50=1.6392 p95=2.2316 p99=2.4661 scene_p95=0.244 raster_p95=1.9327 interval_p99=17.7364

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.598 p50=6.598 p95=6.598 p99=6.598
- L counter_click n=200 mean=0.920 p50=0.883 p95=1.108 p99=1.195
- L mount_n100 n=1 mean=22.909 p50=22.909 p95=22.909 p99=22.909
- L unrelated_update_n100 n=50 mean=1.288 p50=1.227 p95=1.501 p99=1.881
- L row_update_n100 n=50 mean=2.285 p50=2.261 p95=2.703 p99=3.127
- L pointer_move_n100 n=120 mean=0.334 p50=0.037 p95=1.423 p99=1.535
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.020 p99=0.036
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 14801400, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=187.604 p50=187.604 p95=187.604 p99=187.604
- L unrelated_update_n1000 n=50 mean=1.381 p50=1.285 p95=1.782 p99=3.380
- L row_update_n1000 n=50 mean=3.765 p50=3.687 p95=4.270 p99=5.866
- L pointer_move_n1000 n=120 mean=0.704 p50=0.102 p95=2.530 p99=2.833
- L wheel_n1000 n=40 mean=0.052 p50=0.047 p95=0.067 p99=0.210
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 54050100, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=572.600 p50=572.600 p95=572.600 p99=572.600
- L unrelated_update_n3000 n=50 mean=1.414 p50=1.296 p95=1.644 p99=5.646
- L row_update_n3000 n=50 mean=8.536 p50=8.411 p95=9.061 p99=12.326
- L pointer_move_n3000 n=120 mean=2.067 p50=0.294 p95=6.653 p99=6.853
- L wheel_n3000 n=40 mean=0.151 p50=0.138 p95=0.268 p99=0.287
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 196579300, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
