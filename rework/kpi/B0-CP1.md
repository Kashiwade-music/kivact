# KPI B0-CP1 (2026-11-20T09:16Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 354 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=66.1 samples=[ 68.8 69.7 62.5 66.1 54.2 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.9 2.7 2.7 2.7 3.3 ]
- K1 main_to_first_present renderer=cpu median_ms=64.8 samples=[ 51.7 80.0 64.8 57.4 73.7 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.6 2.8 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=215.1 samples=[ 537.9 201.5 189.8 219.5 215.1 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26984 private_kib=6012 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26976 private_kib=5988 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=86808 private_kib=104280 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=782.9 configure_ms=20.5]
- basic-catalog renderer=default ws_kib=54152 private_kib=28824 idle_cpu_ms_per_s=138.861 idle_frames=642 cpu_ticks_100ns=14843750 
- rows3000-still renderer=default ws_kib=139612 private_kib=122676 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=54080 private_kib=28848 idle_cpu_ms_per_s=124.019 idle_frames=642 cpu_ticks_100ns=13281250 
- rows3000-still renderer=cpu ws_kib=138468 private_kib=122648 idle_cpu_ms_per_s=17.644 idle_frames=0 cpu_ticks_100ns=1875000 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=27144 private_kib=6708 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=43188 private_kib=24412 idle_cpu_ms_per_s=8.812 idle_frames=0 cpu_ticks_100ns=937500 
- K2 per_item mode=box kib_per_item=5.35
- item-text-0 renderer=cpu ws_kib=27668 private_kib=6860 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=41716 private_kib=22744 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.68
- item-button-0 renderer=cpu ws_kib=27072 private_kib=6748 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=102736 private_kib=94128 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=25.22
- item-row-0 renderer=cpu ws_kib=27080 private_kib=6732 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=138200 private_kib=122332 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=37.04

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
- rows=0 renderer=default frames=345 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.59 ws_kib=34664 work_ms p50=1.5087 p95=1.821 p99=1.9313 scene_p95=0.0555 raster_p95=1.6526 interval_p99=17.6699
- rows=0 renderer=cpu frames=340 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.97 ws_kib=34692 work_ms p50=1.5111 p95=1.8732 p99=1.9772 scene_p95=0.0544 raster_p95=1.7383 interval_p99=17.7992
- rows=3000 renderer=default frames=341 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=15.66 ws_kib=144032 work_ms p50=2.1949 p95=2.5509 p99=2.693 scene_p95=0.2643 raster_p95=2.1736 interval_p99=17.7608
- rows=3000 renderer=cpu frames=340 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=15.46 ws_kib=144176 work_ms p50=2.2216 p95=2.5844 p99=2.6624 scene_p95=0.3077 raster_p95=2.2296 interval_p99=18.1025

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=7.302 p50=7.302 p95=7.302 p99=7.302
- L counter_click n=200 mean=0.928 p50=0.883 p95=1.092 p99=1.302
- L mount_n100 n=1 mean=22.302 p50=22.302 p95=22.302 p99=22.302
- L unrelated_update_n100 n=50 mean=1.232 p50=1.206 p95=1.278 p99=2.050
- L row_update_n100 n=50 mean=2.162 p50=2.148 p95=2.363 p99=2.943
- L pointer_move_n100 n=120 mean=0.315 p50=0.036 p95=1.319 p99=1.429
- L wheel_n100 n=40 mean=0.015 p50=0.014 p95=0.022 p99=0.037
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 14102800, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=180.455 p50=180.455 p95=180.455 p99=180.455
- L unrelated_update_n1000 n=50 mean=1.264 p50=1.198 p95=1.471 p99=3.013
- L row_update_n1000 n=50 mean=4.209 p50=4.017 p95=5.124 p99=6.244
- L pointer_move_n1000 n=120 mean=0.787 p50=0.111 p95=2.974 p99=3.585
- L wheel_n1000 n=40 mean=0.057 p50=0.049 p95=0.183 p99=0.220
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 61377100, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=582.588 p50=582.588 p95=582.588 p99=582.588
- L unrelated_update_n3000 n=50 mean=1.375 p50=1.255 p95=1.601 p99=5.321
- L row_update_n3000 n=50 mean=8.244 p50=8.100 p95=9.151 p99=11.805
- L pointer_move_n3000 n=120 mean=1.944 p50=0.291 p95=6.413 p99=7.089
- L wheel_n3000 n=40 mean=0.150 p50=0.141 p95=0.257 p99=0.313
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 183592200, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
