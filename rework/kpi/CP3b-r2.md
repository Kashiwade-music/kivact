# KPI CP3b-r2 (2026-11-22T14:21Z, MINGW64_NT-10.0-26200, commit 86dbecf4)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=51.3 samples=[ 50.9 54.2 51.3 49.8 68.7 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 2.9 2.6 2.6 2.6 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=52.9 samples=[ 44.2 61.9 52.9 52.0 55.4 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.6 2.7 2.6 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=189.9 samples=[ 533.6 182.2 183.5 189.9 208.4 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26596 private_kib=5872 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26588 private_kib=5868 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84792 private_kib=103140 idle_cpu_ms_per_s=1.479 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=759.9 configure_ms=16.6]
- basic-catalog renderer=default ws_kib=53724 private_kib=29856 idle_cpu_ms_per_s=72.315 idle_frames=635 cpu_ticks_100ns=7656250 
- rows3000-still renderer=default ws_kib=102052 private_kib=96204 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51032 private_kib=28292 idle_cpu_ms_per_s=84.264 idle_frames=634 cpu_ticks_100ns=8906250 
- rows3000-still renderer=cpu ws_kib=102552 private_kib=96632 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25892 private_kib=6556 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36192 private_kib=18092 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.43
- item-text-0 renderer=cpu ws_kib=25872 private_kib=6536 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34592 private_kib=15904 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.91
- item-button-0 renderer=cpu ws_kib=25872 private_kib=6524 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=82596 private_kib=68796 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.91
- item-row-0 renderer=cpu ws_kib=25888 private_kib=6532 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=102320 private_kib=96208 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=25.48

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
- rows=0 renderer=default frames=334 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=7.28 ws_kib=33164 work_ms p50=1.0883 p95=1.5153 p99=1.6185 scene_p95=0.0506 raster_p95=1.4203 interval_p99=17.548
- rows=0 renderer=cpu frames=335 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.28 ws_kib=33176 work_ms p50=1.1061 p95=1.54 p99=1.6 scene_p95=0.0536 raster_p95=1.4503 interval_p99=17.5788
- rows=3000 renderer=default frames=335 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.47 ws_kib=108092 work_ms p50=1.2843 p95=1.73 p99=1.877 scene_p95=0.1234 raster_p95=1.5437 interval_p99=17.4397
- rows=3000 renderer=cpu frames=335 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=9.79 ws_kib=108124 work_ms p50=1.2468 p95=1.7426 p99=1.9296 scene_p95=0.1443 raster_p95=1.5379 interval_p99=17.5654

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.233 p50=6.233 p95=6.233 p99=6.233
- L counter_click n=200 mean=0.955 p50=1.016 p95=1.045 p99=1.197
- L mount_n100 n=1 mean=20.761 p50=20.761 p95=20.761 p99=20.761
- L unrelated_update_n100 n=50 mean=0.870 p50=0.851 p95=0.987 p99=1.322
- L row_update_n100 n=50 mean=1.100 p50=1.092 p95=1.154 p99=1.549
- L pointer_move_n100 n=120 mean=0.240 p50=0.031 p95=0.981 p99=1.045
- L wheel_n100 n=40 mean=0.013 p50=0.013 p95=0.013 p99=0.042
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 4748600, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=160.096 p50=160.096 p95=160.096 p99=160.096
- L unrelated_update_n1000 n=50 mean=1.083 p50=1.064 p95=1.444 p99=2.386
- L row_update_n1000 n=50 mean=2.111 p50=2.001 p95=2.539 p99=4.298
- L pointer_move_n1000 n=120 mean=0.524 p50=0.104 p95=1.877 p99=2.555
- L wheel_n1000 n=40 mean=0.048 p50=0.047 p95=0.056 p99=0.061
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 34903200, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=489.457 p50=489.457 p95=489.457 p99=489.457
- L unrelated_update_n3000 n=50 mean=0.951 p50=0.851 p95=1.148 p99=4.215
- L row_update_n3000 n=50 mean=3.054 p50=2.802 p95=3.616 p99=6.914
- L pointer_move_n3000 n=120 mean=0.991 p50=0.271 p95=2.772 p99=3.481
- L wheel_n3000 n=40 mean=0.136 p50=0.135 p95=0.142 p99=0.144
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 90669100, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18902528
- K6 kv_startup default 18902016
- K6 kv_startup no-default-features 11215872
- K6 basic-catalog default 35375616
- K6 kv_startup no-gpu 14270464
- K6 basic-catalog no-gpu 30757888
