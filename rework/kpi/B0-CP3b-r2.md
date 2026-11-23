# KPI B0-CP3b-r2 (2026-11-22T13:58Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 5 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=74.5 samples=[ 58.0 76.3 72.9 81.0 74.5 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.8 3.5 2.7 2.7 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=64.1 samples=[ 61.9 68.9 97.1 64.1 63.6 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.7 2.6 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=222.9 samples=[ 639.8 232.5 222.9 213.6 218.6 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26944 private_kib=6048 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26892 private_kib=5988 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=86568 private_kib=103500 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=657.6 configure_ms=19.0]
- basic-catalog renderer=default ws_kib=54276 private_kib=29016 idle_cpu_ms_per_s=102.724 idle_frames=639 cpu_ticks_100ns=10937500 
- rows3000-still renderer=default ws_kib=138468 private_kib=122568 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=54152 private_kib=28916 idle_cpu_ms_per_s=108.929 idle_frames=637 cpu_ticks_100ns=11562500 
- rows3000-still renderer=cpu ws_kib=138280 private_kib=122344 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=27156 private_kib=6792 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=43140 private_kib=24368 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.33
- item-text-0 renderer=cpu ws_kib=27092 private_kib=6720 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=41728 private_kib=22836 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.88
- item-button-0 renderer=cpu ws_kib=27104 private_kib=6736 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=102348 private_kib=93640 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=25.08
- item-row-0 renderer=cpu ws_kib=27084 private_kib=6732 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=138224 private_kib=122312 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=37.05

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
- heap mode=row live_bytes_per_item=35791
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
- rows=0 renderer=default frames=336 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=10.87 ws_kib=34748 work_ms p50=1.1843 p95=1.6588 p99=1.8888 scene_p95=0.0462 raster_p95=1.5167 interval_p99=17.5974
- rows=0 renderer=cpu frames=339 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=10.24 ws_kib=34808 work_ms p50=1.1496 p95=1.729 p99=2.1415 scene_p95=0.0642 raster_p95=1.4639 interval_p99=19.7407
- rows=3000 renderer=default frames=339 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=13.28 ws_kib=144876 work_ms p50=1.578 p95=2.2663 p99=2.381 scene_p95=0.2489 raster_p95=1.9576 interval_p99=17.7669
- rows=3000 renderer=cpu frames=338 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=13.03 ws_kib=141520 work_ms p50=1.5411 p95=2.2394 p99=2.5087 scene_p95=0.2373 raster_p95=1.9407 interval_p99=17.8048

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.771 p50=5.771 p95=5.771 p99=5.771
- L counter_click n=200 mean=0.947 p50=0.898 p95=1.210 p99=1.667
- L mount_n100 n=1 mean=22.332 p50=22.332 p95=22.332 p99=22.332
- L unrelated_update_n100 n=50 mean=1.235 p50=1.217 p95=1.295 p99=2.017
- L row_update_n100 n=50 mean=2.254 p50=2.263 p95=2.569 p99=3.236
- L pointer_move_n100 n=120 mean=0.320 p50=0.037 p95=1.321 p99=1.413
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.014 p99=0.022
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 15353400, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=188.706 p50=188.706 p95=188.706 p99=188.706
- L unrelated_update_n1000 n=50 mean=1.269 p50=1.226 p95=1.276 p99=3.146
- L row_update_n1000 n=50 mean=3.639 p50=3.632 p95=3.942 p99=5.668
- L pointer_move_n1000 n=120 mean=0.649 p50=0.101 p95=2.408 p99=2.600
- L wheel_n1000 n=40 mean=0.048 p50=0.048 p95=0.053 p99=0.055
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 50801800, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=532.251 p50=532.251 p95=532.251 p99=532.251
- L unrelated_update_n3000 n=50 mean=1.311 p50=1.224 p95=1.304 p99=5.187
- L row_update_n3000 n=50 mean=8.490 p50=8.486 p95=9.740 p99=12.147
- L pointer_move_n3000 n=120 mean=1.991 p50=0.300 p95=6.380 p99=6.561
- L wheel_n3000 n=40 mean=0.145 p50=0.140 p95=0.176 p99=0.282
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 188020900, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
