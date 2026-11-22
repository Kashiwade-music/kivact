# KPI CP3-r3 (2026-11-22T11:43Z, MINGW64_NT-10.0-26200, commit 1ad8547c)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=47.8 samples=[ 49.7 47.1 47.8 46.5 60.2 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.7 2.6 2.9 2.9 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=47.4 samples=[ 50.2 47.4 51.1 45.2 46.1 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.7 2.6 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=187.8 samples=[ 594.0 187.8 183.1 180.2 201.8 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25392 private_kib=5796 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25460 private_kib=5888 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=83480 private_kib=101876 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=865.2 configure_ms=20.1]
- basic-catalog renderer=default ws_kib=52692 private_kib=30048 idle_cpu_ms_per_s=91.370 idle_frames=636 cpu_ticks_100ns=9687500 
- rows3000-still renderer=default ws_kib=165344 private_kib=160328 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=52804 private_kib=30144 idle_cpu_ms_per_s=111.441 idle_frames=640 cpu_ticks_100ns=11875000 
- rows3000-still renderer=cpu ws_kib=165012 private_kib=160028 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25952 private_kib=6616 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=52368 private_kib=34080 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=8.81
- item-text-0 renderer=cpu ws_kib=25964 private_kib=6636 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34352 private_kib=15972 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.80
- item-button-0 renderer=cpu ws_kib=25896 private_kib=6552 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=129992 private_kib=116728 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=34.70
- item-row-0 renderer=cpu ws_kib=25984 private_kib=6740 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=165120 private_kib=159980 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=46.38

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=8633
    [box] H site    5501.5 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
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
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
- heap mode=button live_bytes_per_item=33974
    [button] H site   16493.5 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     672.1 B/item     9002 allocs kivact_style::sheet::props::Style::push
- heap mode=row live_bytes_per_item=47780
    [row] H site   21989.5 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1120.1 B/item    15002 allocs kivact_style::sheet::props::Style::push
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site     832.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=335 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.84 ws_kib=33164 work_ms p50=1.002 p95=1.3954 p99=1.5967 scene_p95=0.0441 raster_p95=1.2959 interval_p99=17.4356
- rows=0 renderer=cpu frames=335 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.96 ws_kib=33176 work_ms p50=1.0424 p95=1.4084 p99=1.6005 scene_p95=0.0371 raster_p95=1.3002 interval_p99=17.3284
- rows=3000 renderer=default frames=335 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.27 ws_kib=170080 work_ms p50=1.2363 p95=1.7429 p99=1.8736 scene_p95=0.1346 raster_p95=1.5422 interval_p99=17.4536
- rows=3000 renderer=cpu frames=335 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=9.53 ws_kib=170756 work_ms p50=1.269 p95=1.7265 p99=1.8319 scene_p95=0.1255 raster_p95=1.5318 interval_p99=17.4011

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.168 p50=5.168 p95=5.168 p99=5.168
- L counter_click n=200 mean=0.768 p50=0.741 p95=1.023 p99=1.050
- L mount_n100 n=1 mean=21.925 p50=21.925 p95=21.925 p99=21.925
- L unrelated_update_n100 n=50 mean=0.942 p50=0.868 p95=1.162 p99=1.372
- L row_update_n100 n=50 mean=1.113 p50=1.104 p95=1.167 p99=1.641
- L pointer_move_n100 n=120 mean=0.236 p50=0.032 p95=0.923 p99=0.950
- L wheel_n100 n=40 mean=0.014 p50=0.013 p95=0.014 p99=0.027
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 4623200, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=171.431 p50=171.431 p95=171.431 p99=171.431
- L unrelated_update_n1000 n=50 mean=0.925 p50=0.879 p95=1.179 p99=2.334
- L row_update_n1000 n=50 mean=1.785 p50=1.686 p95=2.186 p99=3.435
- L pointer_move_n1000 n=120 mean=0.546 p50=0.119 p95=1.833 p99=1.992
- L wheel_n1000 n=40 mean=0.051 p50=0.049 p95=0.061 p99=0.078
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 37331900, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=527.280 p50=527.280 p95=527.280 p99=527.280
- L unrelated_update_n3000 n=50 mean=0.967 p50=0.870 p95=1.179 p99=4.249
- L row_update_n3000 n=50 mean=3.147 p50=3.057 p95=3.640 p99=6.914
- L pointer_move_n3000 n=120 mean=1.240 p50=0.329 p95=3.596 p99=3.835
- L wheel_n3000 n=40 mean=0.138 p50=0.137 p95=0.143 p99=0.170
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 116498700, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18902016
- K6 kv_startup default 18901504
- K6 kv_startup no-default-features 11215872
- K6 basic-catalog default 35375616
- K6 kv_startup no-gpu 14269952
- K6 basic-catalog no-gpu 30757376
