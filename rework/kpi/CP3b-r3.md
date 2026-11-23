# KPI CP3b-r3 (2026-11-22T15:00Z, MINGW64_NT-10.0-26200, commit 86dbecf4)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=42.1 samples=[ 43.7 40.6 39.4 42.1 51.4 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 2.6 2.6 2.6 2.6 2.5 ]
- K1 main_to_first_present renderer=cpu median_ms=42.4 samples=[ 42.4 43.7 48.4 41.5 38.7 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.6 2.6 2.8 2.6 2.6 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=174.3 samples=[ 550.6 173.9 165.5 181.5 174.3 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25344 private_kib=5756 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25392 private_kib=5808 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=83456 private_kib=102956 idle_cpu_ms_per_s=1.484 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=634.1 configure_ms=16.0]
- basic-catalog renderer=default ws_kib=51104 private_kib=28412 idle_cpu_ms_per_s=72.482 idle_frames=632 cpu_ticks_100ns=7656250 
- rows3000-still renderer=default ws_kib=102360 private_kib=96488 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51172 private_kib=28588 idle_cpu_ms_per_s=72.659 idle_frames=632 cpu_ticks_100ns=7656250 
- rows3000-still renderer=cpu ws_kib=102576 private_kib=96516 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25876 private_kib=6544 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36292 private_kib=18096 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.47
- item-text-0 renderer=cpu ws_kib=25848 private_kib=6512 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34284 private_kib=15920 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.81
- item-button-0 renderer=cpu ws_kib=25892 private_kib=6548 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=82436 private_kib=68340 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.85
- item-row-0 renderer=cpu ws_kib=25884 private_kib=6552 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=102544 private_kib=96380 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=25.55

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3425
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     304.3 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
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
    [text] H site      65.5 B/item        1 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0<u32,kivact_core::view::descriptor::description::View,alloc::vec::Vec<u32,alloc::alloc::Global>,kv_heap::main::closure$0::closure_env$1>
- heap mode=button live_bytes_per_item=18420
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     912.3 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     672.1 B/item     9002 allocs kivact_style::sheet::props::Style::push
- heap mode=row live_bytes_per_item=27018
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1216.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1120.1 B/item    15002 allocs kivact_style::sheet::props::Style::push
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site     832.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=332 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=7.89 ws_kib=33152 work_ms p50=1.0301 p95=1.4447 p99=1.5816 scene_p95=0.0339 raster_p95=1.3599 interval_p99=17.3545
- rows=0 renderer=cpu frames=334 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.74 ws_kib=33172 work_ms p50=1.0593 p95=1.4897 p99=1.5635 scene_p95=0.0462 raster_p95=1.3963 interval_p99=17.4597
- rows=3000 renderer=default frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.80 ws_kib=107956 work_ms p50=1.2092 p95=1.68 p99=1.8723 scene_p95=0.1089 raster_p95=1.5054 interval_p99=17.6443
- rows=3000 renderer=cpu frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.45 ws_kib=107820 work_ms p50=1.2334 p95=1.6734 p99=1.7442 scene_p95=0.1199 raster_p95=1.5026 interval_p99=17.474

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=4.958 p50=4.958 p95=4.958 p99=4.958
- L counter_click n=200 mean=0.930 p50=1.039 p95=1.079 p99=1.113
- L mount_n100 n=1 mean=21.870 p50=21.870 p95=21.870 p99=21.870
- L unrelated_update_n100 n=50 mean=0.915 p50=0.870 p95=1.174 p99=1.531
- L row_update_n100 n=50 mean=1.159 p50=1.140 p95=1.384 p99=1.604
- L pointer_move_n100 n=120 mean=0.264 p50=0.033 p95=1.083 p99=1.300
- L wheel_n100 n=40 mean=0.014 p50=0.013 p95=0.020 p99=0.032
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 5336500, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=169.725 p50=169.725 p95=169.725 p99=169.725
- L unrelated_update_n1000 n=50 mean=1.025 p50=0.903 p95=1.311 p99=2.365
- L row_update_n1000 n=50 mean=1.658 p50=1.608 p95=1.788 p99=3.454
- L pointer_move_n1000 n=120 mean=0.476 p50=0.098 p95=1.598 p99=2.008
- L wheel_n1000 n=40 mean=0.048 p50=0.047 p95=0.052 p99=0.053
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 30603600, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=498.017 p50=498.017 p95=498.017 p99=498.017
- L unrelated_update_n3000 n=50 mean=0.968 p50=0.854 p95=1.151 p99=4.370
- L row_update_n3000 n=50 mean=2.834 p50=2.665 p95=3.191 p99=7.123
- L pointer_move_n3000 n=120 mean=0.990 p50=0.268 p95=2.924 p99=3.649
- L wheel_n3000 n=40 mean=0.135 p50=0.134 p95=0.143 p99=0.143
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 89157800, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18902528
- K6 kv_startup default 18902016
- K6 kv_startup no-default-features 11215872
- K6 basic-catalog default 35375616
- K6 kv_startup no-gpu 14270464
- K6 basic-catalog no-gpu 30757888
