# KPI remote (2026-11-23T06:06Z, MINGW64_NT-10.0-22631, commit dfd4c326)

release build of kivact-example-counter (cold for its own target dir): 93 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=40.8 samples=[ 43.9 38.0 37.1 41.0 40.8 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.8 2.7 2.7 2.8 2.9 ]
- K1 main_to_first_present renderer=cpu median_ms=40.1 samples=[ 39.8 42.6 44.2 40.1 39.3 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 2.7 3.1 2.8 2.8 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=203.6 samples=[ 361.6 199.1 201.0 203.6 204.6 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=22264 private_kib=6940 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=22264 private_kib=6940 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=161052 private_kib=123876 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=120.0 configure_ms=127.9]
- basic-catalog renderer=default ws_kib=39856 private_kib=21556 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=default ws_kib=93380 private_kib=81432 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=39652 private_kib=21288 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=93372 private_kib=81376 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=23168 private_kib=8116 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=33516 private_kib=19692 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.45
- item-text-0 renderer=cpu ws_kib=23176 private_kib=8132 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=31552 private_kib=17548 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.79
- item-button-0 renderer=cpu ws_kib=23164 private_kib=8120 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=76312 private_kib=69076 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=17.72
- item-row-0 renderer=cpu ws_kib=23208 private_kib=8148 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=93156 private_kib=81184 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=23.32

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
- heap mode=button live_bytes_per_item=18105
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
- heap mode=row live_bytes_per_item=21496
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1216.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site     832.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=174 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=7.19 ws_kib=33004 work_ms p50=1.8599 p95=3.1859 p99=4.0401 scene_p95=0.0663 raster_p95=3.0121 interval_p99=33.7201
- rows=0 renderer=cpu frames=172 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=32936 work_ms p50=2.899 p95=3.2679 p99=3.7773 scene_p95=0.0653 raster_p95=3.0874 interval_p99=33.6755
- rows=3000 renderer=default frames=173 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=101756 work_ms p50=2.82 p95=3.8721 p99=4.8603 scene_p95=0.1665 raster_p95=3.5699 interval_p99=34.0772
- rows=3000 renderer=cpu frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=101764 work_ms p50=2.4098 p95=3.6681 p99=3.9151 scene_p95=0.1511 raster_p95=3.4094 interval_p99=33.8004

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=3.891 p50=3.891 p95=3.891 p99=3.891
- L counter_click n=200 mean=0.395 p50=0.391 p95=0.415 p99=0.441
- L mount_n100 n=1 mean=12.971 p50=12.971 p95=12.971 p99=12.971
- L unrelated_update_n100 n=50 mean=0.491 p50=0.462 p95=0.577 p99=1.241
- L row_update_n100 n=50 mean=0.600 p50=0.591 p95=0.742 p99=0.899
- L pointer_move_n100 n=120 mean=0.118 p50=0.016 p95=0.454 p99=0.516
- L wheel_n100 n=40 mean=0.007 p50=0.007 p95=0.008 p99=0.015
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 2765, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 2208900, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=89.982 p50=89.982 p95=89.982 p99=89.982
- L unrelated_update_n1000 n=50 mean=0.486 p50=0.462 p95=0.534 p99=1.393
- L row_update_n1000 n=50 mean=0.877 p50=0.850 p95=0.926 p99=1.867
- L pointer_move_n1000 n=120 mean=0.267 p50=0.068 p95=0.826 p99=1.058
- L wheel_n1000 n=40 mean=0.028 p50=0.026 p95=0.042 p99=0.076
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 26165, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 17148600, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=272.932 p50=272.932 p95=272.932 p99=272.932
- L unrelated_update_n3000 n=50 mean=0.531 p50=0.469 p95=0.577 p99=3.001
- L row_update_n3000 n=50 mean=1.500 p50=1.427 p95=1.567 p99=4.384
- L pointer_move_n3000 n=120 mean=0.508 p50=0.170 p95=1.365 p99=1.419
- L wheel_n3000 n=40 mean=0.087 p50=0.086 p95=0.097 p99=0.102
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 78165, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 44778700, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18907136
- K6 kv_startup default 18906624
- K6 kv_startup no-default-features 11220480
- K6 basic-catalog default 35358208
- K6 kv_startup no-gpu 14274560
- K6 basic-catalog no-gpu 30740480
