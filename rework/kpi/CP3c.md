# KPI CP3c (2026-11-22T23:57Z, MINGW64_NT-10.0-26200, commit 44fff3e8)

release build of kivact-example-counter (cold for its own target dir): 215 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=48.9 samples=[ 56.6 47.9 48.6 48.9 49.2 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 3.3 3.1 2.6 2.6 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=44.5 samples=[ 44.4 44.2 48.9 52.3 44.5 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 3.7 2.6 2.6 2.6 2.6 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=166.3 samples=[ 516.5 172.1 163.7 166.3 162.4 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26236 private_kib=5848 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26176 private_kib=5808 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84460 private_kib=103092 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=614.5 configure_ms=15.9]
- basic-catalog renderer=default ws_kib=53304 private_kib=29816 idle_cpu_ms_per_s=78.470 idle_frames=632 cpu_ticks_100ns=8281250 
- rows3000-still renderer=default ws_kib=138900 private_kib=132248 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=53124 private_kib=29700 idle_cpu_ms_per_s=67.994 idle_frames=634 cpu_ticks_100ns=7187500 
- rows3000-still renderer=cpu ws_kib=138328 private_kib=132244 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=26572 private_kib=6532 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=41948 private_kib=23072 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.13
- item-text-0 renderer=cpu ws_kib=26600 private_kib=6552 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=40732 private_kib=21816 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.71
- item-button-0 renderer=cpu ws_kib=26612 private_kib=6564 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=101944 private_kib=87852 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=25.11
- item-row-0 renderer=cpu ws_kib=26620 private_kib=6564 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=137928 private_kib=131828 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=37.10

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=4998
    [box] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [box] H site     571.9 B/item     3023 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     304.3 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     264.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [box] H site     168.3 B/item     3011 allocs arvane_native::a11y::conversion::update
- heap mode=text live_bytes_per_item=4439
    [text] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [text] H site     667.1 B/item     9023 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [text] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [text] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [text] H site     273.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [text] H site     183.5 B/item     9011 allocs arvane_native::a11y::conversion::update
    [text] H site     160.1 B/item     6002 allocs kivact_layout::tree::pass::impl$0::leaf::closure$1::closure$2
    [text] H site     114.2 B/item       25 allocs arvane_native::native_input::windows::subclass
- heap mode=button live_bytes_per_item=24432
    [button] H site    3113.0 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    2320.9 B/item    24023 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     912.3 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
- heap mode=row live_bytes_per_item=38727
    [row] H site    6225.9 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [row] H site    4525.4 B/item    39023 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1216.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.60 ws_kib=34064 work_ms p50=1.0304 p95=1.4092 p99=1.5254 scene_p95=0.0364 raster_p95=1.3106 interval_p99=17.4182
- rows=0 renderer=cpu frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.01 ws_kib=34004 work_ms p50=1.0049 p95=1.3775 p99=1.5372 scene_p95=0.0355 raster_p95=1.2066 interval_p99=17.3051
- rows=3000 renderer=default frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.21 ws_kib=144028 work_ms p50=1.2151 p95=1.5793 p99=1.8489 scene_p95=0.1052 raster_p95=1.3779 interval_p99=17.4538
- rows=3000 renderer=cpu frames=331 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=9.07 ws_kib=143368 work_ms p50=1.2389 p95=1.6657 p99=1.9137 scene_p95=0.114 raster_p95=1.5062 interval_p99=17.4879

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.185 p50=6.185 p95=6.185 p99=6.185
- L counter_click n=200 mean=0.903 p50=0.894 p95=1.066 p99=1.094
- L mount_n100 n=1 mean=21.160 p50=21.160 p95=21.160 p99=21.160
- L unrelated_update_n100 n=50 mean=0.905 p50=0.871 p95=1.182 p99=1.291
- L row_update_n100 n=50 mean=1.156 p50=1.145 p95=1.263 p99=1.510
- L pointer_move_n100 n=120 mean=0.241 p50=0.029 p95=0.970 p99=1.067
- L wheel_n100 n=40 mean=0.013 p50=0.012 p95=0.013 p99=0.038
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 4672500, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=166.601 p50=166.601 p95=166.601 p99=166.601
- L unrelated_update_n1000 n=50 mean=0.906 p50=0.868 p95=1.003 p99=2.329
- L row_update_n1000 n=50 mean=1.665 p50=1.621 p95=1.855 p99=3.382
- L pointer_move_n1000 n=120 mean=0.450 p50=0.096 p95=1.431 p99=1.490
- L wheel_n1000 n=40 mean=0.047 p50=0.046 p95=0.053 p99=0.055
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 29435600, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=501.190 p50=501.190 p95=501.190 p99=501.190
- L unrelated_update_n3000 n=50 mean=1.040 p50=0.883 p95=1.186 p99=4.309
- L row_update_n3000 n=50 mean=2.945 p50=2.810 p95=3.243 p99=7.106
- L pointer_move_n3000 n=120 mean=1.025 p50=0.280 p95=2.885 p99=3.364
- L wheel_n3000 n=40 mean=0.141 p50=0.140 p95=0.149 p99=0.164
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 92577600, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18903040
- K6 kv_startup default 18903040
- K6 kv_startup no-default-features 11216896
- K6 basic-catalog default 35376640
- K6 kv_startup no-gpu 14271488
- K6 basic-catalog no-gpu 30758912
