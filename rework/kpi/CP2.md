# KPI CP2 (2026-11-21T11:35Z, MINGW64_NT-10.0-26200, commit 01bc2d58)
SKIPPED: basic catalog (--no-catalog)

release build of kivact-example-counter (cold for its own target dir): 295 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=44.6 samples=[ 133.0 44.5 48.8 44.6 44.1 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 2.8 2.6 2.6 2.6 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=45.6 samples=[ 53.9 43.7 52.5 45.6 44.1 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.6 2.6 2.6 2.7 2.6 ]

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26196 private_kib=5868 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26212 private_kib=5876 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84156 private_kib=103460 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=825.1 configure_ms=17.4]
- rows3000-still renderer=default ws_kib=139900 private_kib=132960 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=139564 private_kib=133524 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=26600 private_kib=6596 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=42300 private_kib=23504 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.23
- item-text-0 renderer=cpu ws_kib=26544 private_kib=6516 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=40712 private_kib=21768 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.72
- item-button-0 renderer=cpu ws_kib=26568 private_kib=6560 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=102576 private_kib=88356 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=25.34
- item-row-0 renderer=cpu ws_kib=26540 private_kib=6536 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=135896 private_kib=133044 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=36.45

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
- rows=0 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.48 ws_kib=33904 work_ms p50=1.0309 p95=1.2533 p99=1.5786 scene_p95=0.0382 raster_p95=1.1705 interval_p99=17.2268
- rows=0 renderer=cpu frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.06 ws_kib=33884 work_ms p50=1.0439 p95=1.38 p99=1.5694 scene_p95=0.0383 raster_p95=1.2984 interval_p99=17.3524
- rows=3000 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.27 ws_kib=144808 work_ms p50=1.2932 p95=1.6183 p99=1.937 scene_p95=0.1363 raster_p95=1.4714 interval_p99=17.4989
- rows=3000 renderer=cpu frames=337 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.40 ws_kib=144732 work_ms p50=1.3242 p95=1.7815 p99=2.1644 scene_p95=0.1252 raster_p95=1.5733 interval_p99=17.3545

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.211 p50=6.211 p95=6.211 p99=6.211
- L counter_click n=200 mean=0.920 p50=0.831 p95=1.188 p99=1.331
- L mount_n100 n=1 mean=22.213 p50=22.213 p95=22.213 p99=22.213
- L unrelated_update_n100 n=50 mean=1.238 p50=1.262 p95=1.308 p99=1.626
- L row_update_n100 n=50 mean=1.245 p50=1.212 p95=1.406 p99=1.944
- L pointer_move_n100 n=120 mean=0.276 p50=0.029 p95=1.106 p99=1.250
- L wheel_n100 n=40 mean=0.013 p50=0.012 p95=0.013 p99=0.036
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13519900, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=176.761 p50=176.761 p95=176.761 p99=176.761
- L unrelated_update_n1000 n=50 mean=1.276 p50=1.255 p95=1.431 p99=2.737
- L row_update_n1000 n=50 mean=1.754 p50=1.688 p95=1.912 p99=4.080
- L pointer_move_n1000 n=120 mean=0.497 p50=0.098 p95=1.615 p99=1.673
- L wheel_n1000 n=40 mean=0.047 p50=0.046 p95=0.051 p99=0.053
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 39760700, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=539.295 p50=539.295 p95=539.295 p99=539.295
- L unrelated_update_n3000 n=50 mean=1.334 p50=1.264 p95=1.377 p99=4.587
- L row_update_n3000 n=50 mean=3.329 p50=3.282 p95=4.349 p99=7.658
- L pointer_move_n3000 n=120 mean=1.199 p50=0.288 p95=4.122 p99=4.304
- L wheel_n3000 n=40 mean=0.144 p50=0.140 p95=0.163 p99=0.185
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 119616300, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18856448
- K6 kv_startup default 18855936
- K6 kv_startup no-default-features 11169792
- K6 kv_startup no-gpu 14224384
