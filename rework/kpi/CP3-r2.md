# KPI CP3-r2 (2026-11-22T11:03Z, MINGW64_NT-10.0-26200, commit 1ad8547c)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=45.0 samples=[ 141.5 41.5 45.0 42.0 45.3 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 2.7 2.6 2.6 2.6 3.6 ]
- K1 main_to_first_present renderer=cpu median_ms=41.6 samples=[ 43.5 41.6 42.2 41.5 41.6 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.6 2.5 2.6 2.6 2.6 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=176.9 samples=[ 522.7 168.5 176.9 185.1 172.7 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25452 private_kib=5840 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25408 private_kib=5812 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=83404 private_kib=103244 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=718.5 configure_ms=15.9]
- basic-catalog renderer=default ws_kib=52716 private_kib=30012 idle_cpu_ms_per_s=82.546 idle_frames=634 cpu_ticks_100ns=8750000 
- rows3000-still renderer=default ws_kib=165080 private_kib=159944 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=52780 private_kib=30108 idle_cpu_ms_per_s=87.956 idle_frames=639 cpu_ticks_100ns=9375000 
- rows3000-still renderer=cpu ws_kib=164664 private_kib=159768 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25896 private_kib=6572 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=52328 private_kib=34196 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=8.81
- item-text-0 renderer=cpu ws_kib=25876 private_kib=6532 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34428 private_kib=15792 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.85
- item-button-0 renderer=cpu ws_kib=25888 private_kib=6556 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=130148 private_kib=116812 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=34.75
- item-row-0 renderer=cpu ws_kib=25896 private_kib=6552 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=165368 private_kib=160372 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=46.49

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=8633
    [box] H site    5501.5 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     264.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     224.1 B/item     3002 allocs kivact_style::sheet::props::Style::push
    [box] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [box] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [box] H site     109.4 B/item        2 allocs kivact_engine::paint::list::walk::Walk::children
- heap mode=text live_bytes_per_item=2691
    [text] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [text] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [text] H site     273.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [text] H site     160.1 B/item     6002 allocs kivact_layout::tree::pass::impl$0::leaf::closure$1::closure$2
    [text] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.4 B/item        2 allocs kivact_engine::paint::list::walk::Walk::children
    [text] H site      66.9 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
- heap mode=button live_bytes_per_item=33974
    [button] H site   16493.5 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     672.1 B/item     9002 allocs kivact_style::sheet::props::Style::push
- heap mode=row live_bytes_per_item=47780
    [row] H site   21989.5 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1120.1 B/item    15002 allocs kivact_style::sheet::props::Style::push
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site     832.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=336 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=9.19 ws_kib=33164 work_ms p50=1.0038 p95=1.4312 p99=1.6136 scene_p95=0.0469 raster_p95=1.3 interval_p99=17.6484
- rows=0 renderer=cpu frames=334 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=7.28 ws_kib=33164 work_ms p50=1.0696 p95=1.4572 p99=1.6556 scene_p95=0.0469 raster_p95=1.3652 interval_p99=17.5242
- rows=3000 renderer=default frames=334 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.42 ws_kib=170644 work_ms p50=1.2718 p95=1.7685 p99=1.9268 scene_p95=0.13 raster_p95=1.5643 interval_p99=17.4764
- rows=3000 renderer=cpu frames=335 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.35 ws_kib=170476 work_ms p50=1.2596 p95=1.7222 p99=1.9142 scene_p95=0.129 raster_p95=1.5274 interval_p99=17.567

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.346 p50=6.346 p95=6.346 p99=6.346
- L counter_click n=200 mean=0.785 p50=0.750 p95=1.050 p99=1.097
- L mount_n100 n=1 mean=21.892 p50=21.892 p95=21.892 p99=21.892
- L unrelated_update_n100 n=50 mean=0.910 p50=0.880 p95=1.180 p99=1.353
- L row_update_n100 n=50 mean=1.143 p50=1.135 p95=1.274 p99=1.585
- L pointer_move_n100 n=120 mean=0.241 p50=0.031 p95=0.945 p99=1.031
- L wheel_n100 n=40 mean=0.014 p50=0.013 p95=0.014 p99=0.038
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 4675700, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=189.619 p50=189.619 p95=189.619 p99=189.619
- L unrelated_update_n1000 n=50 mean=1.089 p50=1.070 p95=1.253 p99=2.713
- L row_update_n1000 n=50 mean=1.598 p50=1.555 p95=1.615 p99=3.381
- L pointer_move_n1000 n=120 mean=0.447 p50=0.097 p95=1.420 p99=1.482
- L wheel_n1000 n=40 mean=0.048 p50=0.047 p95=0.054 p99=0.072
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 29744900, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=512.134 p50=512.134 p95=512.134 p99=512.134
- L unrelated_update_n3000 n=50 mean=1.016 p50=0.867 p95=1.178 p99=4.225
- L row_update_n3000 n=50 mean=2.937 p50=2.812 p95=3.407 p99=6.973
- L pointer_move_n3000 n=120 mean=0.976 p50=0.274 p95=2.858 p99=2.931
- L wheel_n3000 n=40 mean=0.137 p50=0.135 p95=0.150 p99=0.162
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 89583000, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18902016
- K6 kv_startup default 18901504
- K6 kv_startup no-default-features 11215872
- K6 basic-catalog default 35375616
- K6 kv_startup no-gpu 14269952
- K6 basic-catalog no-gpu 30757376
