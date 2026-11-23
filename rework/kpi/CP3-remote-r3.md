# KPI remote (2026-11-23T04:21Z, MINGW64_NT-10.0-22631, commit dd9c4dad)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=39.2 samples=[ 42.7 39.1 40.3 39.2 38.5 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 3.0 2.7 2.7 2.7 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=39.2 samples=[ 39.1 41.9 39.6 39.2 37.8 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 2.8 2.7 2.8 2.8 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=203.0 samples=[ 350.6 205.3 197.4 203.0 199.2 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=22236 private_kib=6920 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=22308 private_kib=7012 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=161044 private_kib=123716 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=113.5 configure_ms=121.9]
- basic-catalog renderer=default ws_kib=39624 private_kib=21452 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=default ws_kib=97436 private_kib=95756 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=39780 private_kib=21580 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=97932 private_kib=96056 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=23156 private_kib=8112 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=33284 private_kib=19388 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.38
- item-text-0 renderer=cpu ws_kib=23148 private_kib=8124 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=31556 private_kib=17544 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.80
- item-button-0 renderer=cpu ws_kib=23208 private_kib=8184 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=79492 private_kib=70132 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.76
- item-row-0 renderer=cpu ws_kib=23164 private_kib=8124 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=97456 private_kib=96096 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.76

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3320
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     304.3 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     264.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [box] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [box] H site     112.0 B/item     3002 allocs kivact_style::sheet::props::impl$2::clone
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
    [text] H site      65.5 B/item        1 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0<u32,kivact_core::view::descriptor::description::View,alloc::vec::Vec<u32,alloc::alloc::Global>,kv_heap::main::closure$0::closure_env$1>
- heap mode=button live_bytes_per_item=18175
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     912.3 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     528.2 B/item    12004 allocs kivact_core::view::element_data::Common::push_style
- heap mode=row live_bytes_per_item=26389
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1216.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site     832.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=173 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=32932 work_ms p50=2.4185 p95=3.3927 p99=4.4254 scene_p95=0.0702 raster_p95=3.2139 interval_p99=34.6396
- rows=0 renderer=cpu frames=173 fps=32.1 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=32956 work_ms p50=2.71 p95=3.4666 p99=3.8737 scene_p95=0.0641 raster_p95=3.2946 interval_p99=33.7021
- rows=3000 renderer=default frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=106132 work_ms p50=1.7501 p95=3.6516 p99=4.8353 scene_p95=0.1427 raster_p95=3.4206 interval_p99=33.9858
- rows=3000 renderer=cpu frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=106092 work_ms p50=2.7214 p95=3.6982 p99=3.9151 scene_p95=0.1658 raster_p95=3.4505 interval_p99=33.9725

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=4.040 p50=4.040 p95=4.040 p99=4.040
- L counter_click n=200 mean=0.406 p50=0.393 p95=0.446 p99=0.749
- L mount_n100 n=1 mean=13.694 p50=13.694 p95=13.694 p99=13.694
- L unrelated_update_n100 n=50 mean=0.474 p50=0.458 p95=0.535 p99=0.815
- L row_update_n100 n=50 mean=0.603 p50=0.595 p95=0.653 p99=0.919
- L pointer_move_n100 n=120 mean=0.117 p50=0.016 p95=0.447 p99=0.497
- L wheel_n100 n=40 mean=0.007 p50=0.007 p95=0.008 p99=0.016
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 2347400, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=95.416 p50=95.416 p95=95.416 p99=95.416
- L unrelated_update_n1000 n=50 mean=0.496 p50=0.466 p95=0.603 p99=1.362
- L row_update_n1000 n=50 mean=0.922 p50=0.875 p95=1.264 p99=1.838
- L pointer_move_n1000 n=120 mean=0.244 p50=0.060 p95=0.773 p99=0.861
- L wheel_n1000 n=40 mean=0.027 p50=0.026 p95=0.031 p99=0.043
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 15897700, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=281.033 p50=281.033 p95=281.033 p99=281.033
- L unrelated_update_n3000 n=50 mean=0.511 p50=0.451 p95=0.548 p99=2.711
- L row_update_n3000 n=50 mean=1.554 p50=1.482 p95=1.911 p99=4.213
- L pointer_move_n3000 n=120 mean=0.563 p50=0.212 p95=1.532 p99=1.600
- L wheel_n3000 n=40 mean=0.090 p50=0.089 p95=0.096 p99=0.107
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 48069600, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18907136
- K6 kv_startup default 18906624
- K6 kv_startup no-default-features 11220480
- K6 basic-catalog default 35371008
- K6 kv_startup no-gpu 14275584
- K6 basic-catalog no-gpu 30753280
