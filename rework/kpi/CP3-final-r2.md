# KPI remote (2026-11-23T06:28Z, MINGW64_NT-10.0-22631, commit dfd4c326)

release build of kivact-example-counter (cold for its own target dir): 0 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=43.4 samples=[ 45.2 41.8 45.1 43.4 43.2 ]
- K4 first_frame_raster renderer=default median_ms=2.9 samples=[ 3.0 2.9 2.9 3.0 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=39.7 samples=[ 41.6 39.7 50.0 37.8 37.8 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 2.8 2.7 2.8 2.8 2.9 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=198.7 samples=[ 345.2 196.3 197.3 198.7 202.9 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=22268 private_kib=6932 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=22220 private_kib=6908 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=161172 private_kib=124044 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=109.5 configure_ms=39.6]
- basic-catalog renderer=default ws_kib=39668 private_kib=21420 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=default ws_kib=93312 private_kib=81400 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=39904 private_kib=21624 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=93524 private_kib=81424 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=23168 private_kib=8144 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=33192 private_kib=19448 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.34
- item-text-0 renderer=cpu ws_kib=23176 private_kib=8132 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=31608 private_kib=17576 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.81
- item-button-0 renderer=cpu ws_kib=23160 private_kib=8120 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=76500 private_kib=69600 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=17.78
- item-row-0 renderer=cpu ws_kib=23188 private_kib=8136 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=92796 private_kib=80940 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=23.20

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
- heap mode=button live_bytes_per_item=18070
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
- heap mode=row live_bytes_per_item=21531
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
- rows=0 renderer=default frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=32964 work_ms p50=2.4877 p95=3.1523 p99=3.3254 scene_p95=0.066 raster_p95=2.9878 interval_p99=33.3507
- rows=0 renderer=cpu frames=172 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=11.29 ws_kib=32912 work_ms p50=2.2583 p95=3.2442 p99=3.9236 scene_p95=0.059 raster_p95=3.0748 interval_p99=33.8514
- rows=3000 renderer=default frames=173 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=101496 work_ms p50=2.6352 p95=3.6555 p99=3.9645 scene_p95=0.159 raster_p95=3.4073 interval_p99=34.1852
- rows=3000 renderer=cpu frames=174 fps=32.1 frame_period_ns=31250000 cpu_pct_of_core=2.31 ws_kib=101776 work_ms p50=2.1767 p95=3.8611 p99=4.5726 scene_p95=0.205 raster_p95=3.6361 interval_p99=33.9661

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=3.905 p50=3.905 p95=3.905 p99=3.905
- L counter_click n=200 mean=0.402 p50=0.396 p95=0.429 p99=0.542
- L mount_n100 n=1 mean=13.000 p50=13.000 p95=13.000 p99=13.000
- L unrelated_update_n100 n=50 mean=0.474 p50=0.461 p95=0.513 p99=0.815
- L row_update_n100 n=50 mean=0.610 p50=0.598 p95=0.652 p99=0.928
- L pointer_move_n100 n=120 mean=0.121 p50=0.016 p95=0.462 p99=0.580
- L wheel_n100 n=40 mean=0.007 p50=0.007 p95=0.008 p99=0.018
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 2765, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 2173700, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=91.070 p50=91.070 p95=91.070 p99=91.070
- L unrelated_update_n1000 n=50 mean=0.488 p50=0.461 p95=0.555 p99=1.393
- L row_update_n1000 n=50 mean=0.881 p50=0.858 p95=0.920 p99=1.933
- L pointer_move_n1000 n=120 mean=0.253 p50=0.060 p95=0.896 p99=1.125
- L wheel_n1000 n=40 mean=0.026 p50=0.026 p95=0.027 p99=0.037
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 26165, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 15354700, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=276.723 p50=276.723 p95=276.723 p99=276.723
- L unrelated_update_n3000 n=50 mean=0.518 p50=0.463 p95=0.581 p99=2.577
- L row_update_n3000 n=50 mean=1.539 p50=1.467 p95=1.718 p99=4.274
- L pointer_move_n3000 n=120 mean=0.513 p50=0.171 p95=1.394 p99=1.536
- L wheel_n3000 n=40 mean=0.090 p50=0.087 p95=0.102 p99=0.157
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 78165, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 45338300, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18907136
- K6 kv_startup default 18906624
- K6 kv_startup no-default-features 11220480
- K6 basic-catalog default 35358208
- K6 kv_startup no-gpu 14274560
- K6 basic-catalog no-gpu 30740480
