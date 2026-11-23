# KPI CP3b (2026-11-22T13:20Z, MINGW64_NT-10.0-26200, commit 86dbecf4)

release build of kivact-example-counter (cold for its own target dir): 250 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=55.5 samples=[ 151.8 54.0 54.9 69.8 55.5 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 2.7 2.6 2.7 2.6 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=59.1 samples=[ 50.5 87.4 55.9 73.4 59.1 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.6 2.7 2.6 2.6 2.6 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=223.3 samples=[ 920.3 215.2 229.1 196.4 223.3 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26532 private_kib=5812 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26584 private_kib=5864 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84728 private_kib=103036 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=875.7 configure_ms=17.9]
- basic-catalog renderer=default ws_kib=53592 private_kib=29740 idle_cpu_ms_per_s=67.659 idle_frames=638 cpu_ticks_100ns=7187500 
- rows3000-still renderer=default ws_kib=139948 private_kib=133588 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=53608 private_kib=29756 idle_cpu_ms_per_s=77.475 idle_frames=638 cpu_ticks_100ns=8281250 
- rows3000-still renderer=cpu ws_kib=140128 private_kib=133624 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=27132 private_kib=6688 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=43096 private_kib=23684 idle_cpu_ms_per_s=4.411 idle_frames=0 cpu_ticks_100ns=468750 
- K2 per_item mode=box kib_per_item=5.32
- item-text-0 renderer=cpu ws_kib=27096 private_kib=6564 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=41460 private_kib=21704 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.79
- item-button-0 renderer=cpu ws_kib=27168 private_kib=6636 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=110824 private_kib=99028 idle_cpu_ms_per_s=184.700 idle_frames=1 cpu_ticks_100ns=19687500 
- K2 per_item mode=button kib_per_item=27.89
- item-row-0 renderer=cpu ws_kib=27244 private_kib=6652 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=140168 private_kib=133704 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=37.64

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=5103
    [box] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [box] H site     571.9 B/item     3024 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     304.3 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     264.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     224.1 B/item     3002 allocs kivact_style::sheet::props::Style::push
    [box] H site     218.5 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
- heap mode=text live_bytes_per_item=4439
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
- heap mode=button live_bytes_per_item=24781
    [button] H site    3113.0 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    2320.9 B/item    24024 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     912.3 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
- heap mode=row live_bytes_per_item=39357
    [row] H site    6225.9 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [row] H site    4525.5 B/item    39024 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1216.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1120.1 B/item    15002 allocs kivact_style::sheet::props::Style::push

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=339 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.74 ws_kib=34436 work_ms p50=1.0654 p95=1.5058 p99=1.7674 scene_p95=0.0521 raster_p95=1.3893 interval_p99=17.7989
- rows=0 renderer=cpu frames=336 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.33 ws_kib=34612 work_ms p50=1.3906 p95=1.7 p99=1.9707 scene_p95=0.0593 raster_p95=1.5744 interval_p99=17.6172
- rows=3000 renderer=default frames=340 fps=60.2 frame_period_ns=16666666 cpu_pct_of_core=9.96 ws_kib=145896 work_ms p50=1.3456 p95=1.9096 p99=2.1463 scene_p95=0.1583 raster_p95=1.6764 interval_p99=18.0234
- rows=3000 renderer=cpu frames=339 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=13.57 ws_kib=145648 work_ms p50=1.3232 p95=1.9277 p99=2.2791 scene_p95=0.1442 raster_p95=1.7242 interval_p99=17.7774

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.578 p50=5.578 p95=5.578 p99=5.578
- L counter_click n=200 mean=1.005 p50=1.054 p95=1.228 p99=1.426
- L mount_n100 n=1 mean=24.177 p50=24.177 p95=24.177 p99=24.177
- L unrelated_update_n100 n=50 mean=1.033 p50=0.964 p95=1.329 p99=1.894
- L row_update_n100 n=50 mean=1.332 p50=1.275 p95=1.728 p99=1.973
- L pointer_move_n100 n=120 mean=0.283 p50=0.036 p95=1.171 p99=1.360
- L wheel_n100 n=40 mean=0.014 p50=0.013 p95=0.021 p99=0.038
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 5748400, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=188.757 p50=188.757 p95=188.757 p99=188.757
- L unrelated_update_n1000 n=50 mean=0.982 p50=0.904 p95=1.248 p99=2.509
- L row_update_n1000 n=50 mean=1.788 p50=1.744 p95=2.034 p99=3.554
- L pointer_move_n1000 n=120 mean=0.500 p50=0.104 p95=1.793 p99=1.947
- L wheel_n1000 n=40 mean=0.049 p50=0.049 p95=0.056 p99=0.059
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 33767000, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=564.760 p50=564.760 p95=564.760 p99=564.760
- L unrelated_update_n3000 n=50 mean=1.071 p50=0.944 p95=1.333 p99=4.824
- L row_update_n3000 n=50 mean=3.801 p50=3.830 p95=4.531 p99=8.059
- L pointer_move_n3000 n=120 mean=1.362 p50=0.325 p95=4.319 p99=4.518
- L wheel_n3000 n=40 mean=0.152 p50=0.138 p95=0.200 p99=0.441
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 131229300, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18902528
- K6 kv_startup default 18902016
- K6 kv_startup no-default-features 11215872
- K6 basic-catalog default 35375616
- K6 kv_startup no-gpu 14270464
- K6 basic-catalog no-gpu 30757888
