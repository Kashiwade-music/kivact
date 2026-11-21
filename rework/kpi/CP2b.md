# KPI CP2b (2026-11-21T12:48Z, MINGW64_NT-10.0-26200, commit 895802fa)
SKIPPED: basic catalog (--no-catalog)

release build of kivact-example-counter (cold for its own target dir): 300 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=46.4 samples=[ 88.4 45.5 46.5 46.4 43.9 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 2.9 2.6 2.6 2.6 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=44.6 samples=[ 43.8 51.5 44.6 43.7 50.3 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 2.8 2.9 2.6 2.9 2.7 ]

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26144 private_kib=5832 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26196 private_kib=5888 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84224 private_kib=102684 idle_cpu_ms_per_s=1.486 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=764.9 configure_ms=15.8]
- rows3000-still renderer=default ws_kib=140052 private_kib=133244 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=138648 private_kib=132504 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=26544 private_kib=6548 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=42272 private_kib=23496 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.24
- item-text-0 renderer=cpu ws_kib=26568 private_kib=6600 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=40756 private_kib=21664 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.73
- item-button-0 renderer=cpu ws_kib=26576 private_kib=6580 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=103288 private_kib=88740 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=25.57
- item-row-0 renderer=cpu ws_kib=26548 private_kib=6560 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=138888 private_kib=133048 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=37.45

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=5068
    [box] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [box] H site     571.9 B/item     3023 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     280.3 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     264.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     224.1 B/item     3002 allocs kivact_style::sheet::decl::Style::push
    [box] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
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
- heap mode=button live_bytes_per_item=24676
    [button] H site    3113.0 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    2320.9 B/item    24023 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1172.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [button] H site     840.3 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
- heap mode=row live_bytes_per_item=39217
    [row] H site    6225.9 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [row] H site    4525.4 B/item    39023 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1172.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1120.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1120.1 B/item    15002 allocs kivact_style::sheet::decl::Style::push

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.28 ws_kib=33892 work_ms p50=1.0577 p95=1.3772 p99=1.5883 scene_p95=0.0422 raster_p95=1.2516 interval_p99=17.3243
- rows=0 renderer=cpu frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.32 ws_kib=33880 work_ms p50=1.0415 p95=1.3927 p99=1.5695 scene_p95=0.0536 raster_p95=1.2804 interval_p99=17.3519
- rows=3000 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.44 ws_kib=144688 work_ms p50=1.2748 p95=1.5806 p99=1.8529 scene_p95=0.1313 raster_p95=1.3983 interval_p99=17.442
- rows=3000 renderer=cpu frames=333 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=10.71 ws_kib=144532 work_ms p50=1.2798 p95=1.5963 p99=1.8713 scene_p95=0.1286 raster_p95=1.4121 interval_p99=17.3962

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.227 p50=6.227 p95=6.227 p99=6.227
- L counter_click n=200 mean=0.943 p50=0.860 p95=1.186 p99=1.309
- L mount_n100 n=1 mean=22.626 p50=22.626 p95=22.626 p99=22.626
- L unrelated_update_n100 n=50 mean=1.287 p50=1.288 p95=1.448 p99=1.598
- L row_update_n100 n=50 mean=1.245 p50=1.222 p95=1.390 p99=1.903
- L pointer_move_n100 n=120 mean=0.275 p50=0.029 p95=1.106 p99=1.197
- L wheel_n100 n=40 mean=0.013 p50=0.012 p95=0.018 p99=0.037
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13145700, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=178.102 p50=178.102 p95=178.102 p99=178.102
- L unrelated_update_n1000 n=50 mean=1.239 p50=1.269 p95=1.431 p99=2.674
- L row_update_n1000 n=50 mean=1.804 p50=1.719 p95=1.935 p99=3.925
- L pointer_move_n1000 n=120 mean=0.503 p50=0.101 p95=1.670 p99=1.735
- L wheel_n1000 n=40 mean=0.045 p50=0.045 p95=0.050 p99=0.052
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 39363100, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=534.049 p50=534.049 p95=534.049 p99=534.049
- L unrelated_update_n3000 n=50 mean=1.313 p50=1.260 p95=1.295 p99=4.651
- L row_update_n3000 n=50 mean=3.426 p50=3.447 p95=3.882 p99=7.990
- L pointer_move_n3000 n=120 mean=1.291 p50=0.337 p95=3.632 p99=4.411
- L wheel_n3000 n=40 mean=0.144 p50=0.135 p95=0.161 p99=0.333
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 124606900, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18856448
- K6 kv_startup default 18855936
- K6 kv_startup no-default-features 11169792
- K6 kv_startup no-gpu 14224384
