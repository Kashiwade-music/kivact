# KPI CP3b-r4 (2026-11-22T15:40Z, MINGW64_NT-10.0-26200, commit 86dbecf4)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=41.7 samples=[ 44.1 39.8 41.7 52.6 40.9 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 2.6 2.6 2.5 2.5 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=39.5 samples=[ 38.8 39.7 40.4 39.1 39.5 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.5 2.6 2.6 2.6 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=173.5 samples=[ 556.0 179.4 173.5 162.6 164.2 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25376 private_kib=5800 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25444 private_kib=5864 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=83656 private_kib=102660 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=601.9 configure_ms=15.5]
- basic-catalog renderer=default ws_kib=51188 private_kib=28444 idle_cpu_ms_per_s=63.752 idle_frames=632 cpu_ticks_100ns=6718750 
- rows3000-still renderer=default ws_kib=102516 private_kib=96196 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51140 private_kib=28424 idle_cpu_ms_per_s=63.709 idle_frames=633 cpu_ticks_100ns=6718750 
- rows3000-still renderer=cpu ws_kib=102156 private_kib=96360 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25840 private_kib=6516 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36328 private_kib=18044 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.50
- item-text-0 renderer=cpu ws_kib=25864 private_kib=6536 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34316 private_kib=15884 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.82
- item-button-0 renderer=cpu ws_kib=25848 private_kib=6504 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=82568 private_kib=68532 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.91
- item-row-0 renderer=cpu ws_kib=25868 private_kib=6540 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=102428 private_kib=96084 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=25.52

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
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
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
- rows=0 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.20 ws_kib=33172 work_ms p50=1.0545 p95=1.5432 p99=1.6668 scene_p95=0.0354 raster_p95=1.4778 interval_p99=17.5199
- rows=0 renderer=cpu frames=332 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=7.04 ws_kib=33272 work_ms p50=1.0262 p95=1.5033 p99=1.636 scene_p95=0.0399 raster_p95=1.422 interval_p99=17.5565
- rows=3000 renderer=default frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.78 ws_kib=107840 work_ms p50=1.1735 p95=1.6599 p99=1.8649 scene_p95=0.1037 raster_p95=1.5168 interval_p99=17.4408
- rows=3000 renderer=cpu frames=333 fps=60.2 frame_period_ns=16666666 cpu_pct_of_core=7.35 ws_kib=107480 work_ms p50=1.234 p95=1.6738 p99=1.8923 scene_p95=0.1137 raster_p95=1.5023 interval_p99=17.5101

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.149 p50=5.149 p95=5.149 p99=5.149
- L counter_click n=200 mean=0.918 p50=1.034 p95=1.088 p99=1.181
- L mount_n100 n=1 mean=24.089 p50=24.089 p95=24.089 p99=24.089
- L unrelated_update_n100 n=50 mean=0.925 p50=0.873 p95=1.183 p99=1.490
- L row_update_n100 n=50 mean=1.151 p50=1.127 p95=1.389 p99=1.574
- L pointer_move_n100 n=120 mean=0.244 p50=0.029 p95=0.989 p99=1.040
- L wheel_n100 n=40 mean=0.013 p50=0.013 p95=0.013 p99=0.026
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 4840200, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=164.558 p50=164.558 p95=164.558 p99=164.558
- L unrelated_update_n1000 n=50 mean=0.918 p50=0.875 p95=1.045 p99=2.288
- L row_update_n1000 n=50 mean=1.664 p50=1.614 p95=1.868 p99=3.653
- L pointer_move_n1000 n=120 mean=0.471 p50=0.097 p95=1.537 p99=1.704
- L wheel_n1000 n=40 mean=0.048 p50=0.047 p95=0.052 p99=0.105
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 30066000, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=506.931 p50=506.931 p95=506.931 p99=506.931
- L unrelated_update_n3000 n=50 mean=0.981 p50=0.873 p95=1.187 p99=4.293
- L row_update_n3000 n=50 mean=2.902 p50=2.762 p95=3.399 p99=7.053
- L pointer_move_n3000 n=120 mean=1.081 p50=0.287 p95=3.372 p99=3.781
- L wheel_n3000 n=40 mean=0.140 p50=0.139 p95=0.151 p99=0.168
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 97937900, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18902528
- K6 kv_startup default 18902016
- K6 kv_startup no-default-features 11215872
- K6 basic-catalog default 35375616
- K6 kv_startup no-gpu 14270464
- K6 basic-catalog no-gpu 30757888
