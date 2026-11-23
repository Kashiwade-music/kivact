# KPI remote (2026-11-23T07:10Z, MINGW64_NT-10.0-22631, commit dfd4c326)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=40.7 samples=[ 44.6 40.7 40.6 41.8 37.7 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.8 2.8 2.8 2.7 2.8 ]
- K1 main_to_first_present renderer=cpu median_ms=40.4 samples=[ 39.5 40.9 49.1 38.3 40.4 ]
- K4 first_frame_raster renderer=cpu median_ms=2.9 samples=[ 2.9 2.8 2.9 2.7 3.0 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=204.7 samples=[ 349.9 198.7 203.1 206.1 204.7 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=22288 private_kib=6980 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=22252 private_kib=6924 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=160940 private_kib=123564 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=245.8 configure_ms=213.8]
- basic-catalog renderer=default ws_kib=39740 private_kib=21524 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=default ws_kib=93804 private_kib=81688 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=39692 private_kib=21444 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=93628 private_kib=81616 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=23216 private_kib=8168 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=33316 private_kib=19404 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.37
- item-text-0 renderer=cpu ws_kib=23160 private_kib=8100 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=31412 private_kib=17572 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.75
- item-button-0 renderer=cpu ws_kib=23188 private_kib=8144 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=76556 private_kib=69288 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=17.79
- item-row-0 renderer=cpu ws_kib=23180 private_kib=8128 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=93492 private_kib=81396 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=23.44

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
- heap mode=row live_bytes_per_item=21566
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
- rows=0 renderer=default frames=173 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=9.80 ws_kib=32976 work_ms p50=0.8306 p95=1.1502 p99=1.5354 scene_p95=0.0322 raster_p95=1.1071 interval_p99=32.0827
- rows=0 renderer=cpu frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=32956 work_ms p50=0.8403 p95=1.4162 p99=1.9723 scene_p95=0.0345 raster_p95=1.341 interval_p99=32.3331
- rows=3000 renderer=default frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=102012 work_ms p50=0.9973 p95=1.7995 p99=2.449 scene_p95=0.09 raster_p95=1.7102 interval_p99=32.5837
- rows=3000 renderer=cpu frames=173 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=101576 work_ms p50=1.0005 p95=1.7937 p99=2.6837 scene_p95=0.1138 raster_p95=1.6043 interval_p99=33.0847

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=4.409 p50=4.409 p95=4.409 p99=4.409
- L counter_click n=200 mean=0.408 p50=0.404 p95=0.433 p99=0.509
- L mount_n100 n=1 mean=14.060 p50=14.060 p95=14.060 p99=14.060
- L unrelated_update_n100 n=50 mean=0.479 p50=0.461 p95=0.556 p99=0.797
- L row_update_n100 n=50 mean=0.613 p50=0.604 p95=0.672 p99=0.929
- L pointer_move_n100 n=120 mean=0.122 p50=0.016 p95=0.465 p99=0.586
- L wheel_n100 n=40 mean=0.007 p50=0.007 p95=0.008 p99=0.014
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 2765, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 2238500, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=103.660 p50=103.660 p95=103.660 p99=103.660
- L unrelated_update_n1000 n=50 mean=0.513 p50=0.485 p95=0.575 p99=1.456
- L row_update_n1000 n=50 mean=0.910 p50=0.886 p95=0.945 p99=1.980
- L pointer_move_n1000 n=120 mean=0.250 p50=0.060 p95=0.787 p99=0.980
- L wheel_n1000 n=40 mean=0.027 p50=0.026 p95=0.028 p99=0.036
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 26165, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 15942200, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=280.012 p50=280.012 p95=280.012 p99=280.012
- L unrelated_update_n3000 n=50 mean=0.543 p50=0.470 p95=0.680 p99=2.593
- L row_update_n3000 n=50 mean=1.555 p50=1.454 p95=1.801 p99=4.371
- L pointer_move_n3000 n=120 mean=0.555 p50=0.193 p95=1.490 p99=1.642
- L wheel_n3000 n=40 mean=0.092 p50=0.091 p95=0.093 p99=0.114
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 78165, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 47367700, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18907136
- K6 kv_startup default 18906624
- K6 kv_startup no-default-features 11220480
- K6 basic-catalog default 35358208
- K6 kv_startup no-gpu 14274560
- K6 basic-catalog no-gpu 30740480
