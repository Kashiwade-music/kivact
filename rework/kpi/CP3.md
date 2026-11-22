# KPI CP3 (2026-11-22T09:55Z, MINGW64_NT-10.0-26200, commit 1ad8547c)

release build of kivact-example-counter (cold for its own target dir): 340 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=52.6 samples=[ 133.5 49.0 47.3 52.6 56.9 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.8 2.6 2.6 2.9 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=48.9 samples=[ 48.9 48.1 60.4 50.4 47.0 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.6 2.6 2.7 3.0 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=200.4 samples=[ 843.6 176.7 254.3 191.1 200.4 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=27248 private_kib=6040 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26636 private_kib=5888 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84624 private_kib=104036 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=780.6 configure_ms=23.2]
- basic-catalog renderer=default ws_kib=55440 private_kib=31520 idle_cpu_ms_per_s=95.646 idle_frames=637 cpu_ticks_100ns=10156250 
- rows3000-still renderer=default ws_kib=203064 private_kib=197092 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=55244 private_kib=31348 idle_cpu_ms_per_s=101.037 idle_frames=640 cpu_ticks_100ns=10781250 
- rows3000-still renderer=cpu ws_kib=202692 private_kib=197248 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=27188 private_kib=6632 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=59116 private_kib=39860 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=10.64
- item-text-0 renderer=cpu ws_kib=27104 private_kib=6668 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=41672 private_kib=21936 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.86
- item-button-0 renderer=cpu ws_kib=27084 private_kib=6632 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=150880 private_kib=136700 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=41.27
- item-row-0 renderer=cpu ws_kib=27748 private_kib=6820 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=202480 private_kib=196940 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=58.24

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=10276
    [box] H site    5501.5 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [box] H site     571.9 B/item     3024 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     264.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     224.1 B/item     3002 allocs kivact_style::sheet::props::Style::push
    [box] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
- heap mode=text live_bytes_per_item=4474
    [text] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [text] H site     667.2 B/item     9024 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [text] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [text] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [text] H site     273.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [text] H site     183.5 B/item     9011 allocs arvane_native::a11y::conversion::update
    [text] H site     160.1 B/item     6002 allocs kivact_layout::tree::pass::impl$0::leaf::closure$1::closure$2
    [text] H site     114.1 B/item       23 allocs arvane_native::native_input::windows::subclass
- heap mode=button live_bytes_per_item=40370
    [button] H site   16493.5 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site    3113.0 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    2320.9 B/item    24024 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
- heap mode=row live_bytes_per_item=60118
    [row] H site   21989.5 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    6225.9 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [row] H site    4525.5 B/item    39024 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1120.1 B/item    15002 allocs kivact_style::sheet::props::Style::push

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.60 ws_kib=34452 work_ms p50=1.0472 p95=1.4829 p99=1.6615 scene_p95=0.0507 raster_p95=1.3713 interval_p99=17.5263
- rows=0 renderer=cpu frames=336 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.10 ws_kib=34512 work_ms p50=1.0756 p95=1.4683 p99=1.6171 scene_p95=0.0542 raster_p95=1.3639 interval_p99=17.5647
- rows=3000 renderer=default frames=337 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.69 ws_kib=207388 work_ms p50=1.3587 p95=1.7419 p99=1.9302 scene_p95=0.135 raster_p95=1.4969 interval_p99=17.492
- rows=3000 renderer=cpu frames=337 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.41 ws_kib=208336 work_ms p50=1.3313 p95=1.8634 p99=2.0371 scene_p95=0.1572 raster_p95=1.6328 interval_p99=17.5588

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.615 p50=6.615 p95=6.615 p99=6.615
- L counter_click n=200 mean=0.901 p50=0.820 p95=1.302 p99=1.416
- L mount_n100 n=1 mean=25.727 p50=25.727 p95=25.727 p99=25.727
- L unrelated_update_n100 n=50 mean=0.996 p50=0.937 p95=1.275 p99=1.848
- L row_update_n100 n=50 mean=1.213 p50=1.183 p95=1.391 p99=1.791
- L pointer_move_n100 n=120 mean=0.257 p50=0.035 p95=1.005 p99=1.159
- L wheel_n100 n=40 mean=0.015 p50=0.014 p95=0.019 p99=0.044
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 5460800, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=197.312 p50=197.312 p95=197.312 p99=197.312
- L unrelated_update_n1000 n=50 mean=1.067 p50=0.973 p95=1.508 p99=2.622
- L row_update_n1000 n=50 mean=1.884 p50=1.811 p95=2.267 p99=3.667
- L pointer_move_n1000 n=120 mean=0.619 p50=0.160 p95=2.199 p99=2.716
- L wheel_n1000 n=40 mean=0.056 p50=0.056 p95=0.073 p99=0.089
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 42307300, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=595.258 p50=595.258 p95=595.258 p99=595.258
- L unrelated_update_n3000 n=50 mean=1.036 p50=0.915 p95=1.224 p99=4.654
- L row_update_n3000 n=50 mean=3.850 p50=3.847 p95=4.864 p99=7.798
- L pointer_move_n3000 n=120 mean=1.221 p50=0.325 p95=3.677 p99=4.216
- L wheel_n3000 n=40 mean=0.143 p50=0.138 p95=0.156 p99=0.256
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 117853300, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18902016
- K6 kv_startup default 18901504
- K6 kv_startup no-default-features 11215872
- K6 basic-catalog default 35375616
- K6 kv_startup no-gpu 14269952
- K6 basic-catalog no-gpu 30757376
