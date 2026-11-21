# KPI CP2-r2 (2026-11-21T14:39Z, MINGW64_NT-10.0-26200, commit 895802fa)
SKIPPED: basic catalog (--no-catalog)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=42.5 samples=[ 43.9 42.2 41.0 42.9 42.5 ]
- K4 first_frame_raster renderer=default median_ms=2.5 samples=[ 2.5 2.5 2.6 2.8 2.5 ]
- K1 main_to_first_present renderer=cpu median_ms=41.9 samples=[ 42.1 41.5 41.9 39.9 43.4 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.6 2.6 2.6 2.6 2.6 ]

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25428 private_kib=5852 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25360 private_kib=5788 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=83148 private_kib=99664 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=608.1 configure_ms=15.7]
- rows3000-still renderer=default ws_kib=101696 private_kib=95564 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=101604 private_kib=95664 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25812 private_kib=6548 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36456 private_kib=18324 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.55
- item-text-0 renderer=cpu ws_kib=25816 private_kib=6532 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34108 private_kib=15856 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.76
- item-button-0 renderer=cpu ws_kib=25812 private_kib=6544 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=82208 private_kib=68360 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.80
- item-row-0 renderer=cpu ws_kib=25856 private_kib=6612 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=101328 private_kib=95376 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=25.16

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3425
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     280.3 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     264.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     224.1 B/item     3002 allocs kivact_style::sheet::decl::Style::push
    [box] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [box] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [box] H site     109.4 B/item        2 allocs kivact_engine::paint::list::walk::Walk::children
- heap mode=text live_bytes_per_item=2656
    [text] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [text] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [text] H site     273.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [text] H site     160.1 B/item     6002 allocs kivact_layout::tree::pass::impl$0::leaf::closure$1::closure$2
    [text] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.4 B/item        2 allocs kivact_engine::paint::list::walk::Walk::children
    [text] H site      66.9 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [text] H site      65.5 B/item        1 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0<u32,kivact_core::view::descriptor::description::View,alloc::vec::Vec<u32,alloc::alloc::Global>,kv_heap::main::closure$0::closure_env$1>
- heap mode=button live_bytes_per_item=18280
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1172.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [button] H site     840.3 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     672.1 B/item     9002 allocs kivact_style::sheet::decl::Style::push
- heap mode=row live_bytes_per_item=26844
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1172.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1120.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1120.1 B/item    15002 allocs kivact_style::sheet::decl::Style::push
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site     832.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.34 ws_kib=33104 work_ms p50=1.0335 p95=1.2002 p99=1.4875 scene_p95=0.0345 raster_p95=1.1065 interval_p99=17.2127
- rows=0 renderer=cpu frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.36 ws_kib=33284 work_ms p50=0.9911 p95=1.2412 p99=1.4592 scene_p95=0.0412 raster_p95=1.137 interval_p99=17.3309
- rows=3000 renderer=default frames=331 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=8.49 ws_kib=107156 work_ms p50=1.2284 p95=1.4227 p99=1.82 scene_p95=0.1123 raster_p95=1.2438 interval_p99=17.2035
- rows=3000 renderer=cpu frames=331 fps=59.8 frame_period_ns=16666666 cpu_pct_of_core=6.78 ws_kib=107184 work_ms p50=1.2183 p95=1.3995 p99=1.7565 scene_p95=0.1093 raster_p95=1.2533 interval_p99=17.2541

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.010 p50=5.010 p95=5.010 p99=5.010
- L counter_click n=200 mean=0.918 p50=0.846 p95=1.163 p99=1.382
- L mount_n100 n=1 mean=22.809 p50=22.809 p95=22.809 p99=22.809
- L unrelated_update_n100 n=50 mean=1.283 p50=1.266 p95=1.441 p99=1.569
- L row_update_n100 n=50 mean=1.234 p50=1.210 p95=1.379 p99=1.868
- L pointer_move_n100 n=120 mean=0.272 p50=0.028 p95=1.093 p99=1.282
- L wheel_n100 n=40 mean=0.013 p50=0.012 p95=0.022 p99=0.027
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 12968500, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=171.621 p50=171.621 p95=171.621 p99=171.621
- L unrelated_update_n1000 n=50 mean=1.244 p50=1.215 p95=1.402 p99=2.580
- L row_update_n1000 n=50 mean=1.709 p50=1.651 p95=1.962 p99=3.538
- L pointer_move_n1000 n=120 mean=0.488 p50=0.097 p95=1.573 p99=2.189
- L wheel_n1000 n=40 mean=0.045 p50=0.044 p95=0.050 p99=0.053
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 38352100, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=519.173 p50=519.173 p95=519.173 p99=519.173
- L unrelated_update_n3000 n=50 mean=1.297 p50=1.222 p95=1.409 p99=4.439
- L row_update_n3000 n=50 mean=3.121 p50=2.973 p95=3.694 p99=7.177
- L pointer_move_n3000 n=120 mean=1.097 p50=0.285 p95=3.303 p99=3.791
- L wheel_n3000 n=40 mean=0.143 p50=0.136 p95=0.165 p99=0.166
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 107138300, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18856448
- K6 kv_startup default 18855936
- K6 kv_startup no-default-features 11169792
- K6 kv_startup no-gpu 14224384
