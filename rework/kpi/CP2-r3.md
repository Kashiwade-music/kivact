# KPI CP2-r3 (2026-11-21T14:09Z, MINGW64_NT-10.0-26200, commit 895802fa)
SKIPPED: basic catalog (--no-catalog)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=43.6 samples=[ 44.4 43.6 43.6 44.8 40.8 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.6 2.8 2.7 2.7 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=42.9 samples=[ 42.2 44.7 41.5 43.3 42.9 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.6 2.9 2.7 2.7 2.6 ]

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25420 private_kib=5884 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25384 private_kib=5844 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=83524 private_kib=103076 idle_cpu_ms_per_s=1.486 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=731.2 configure_ms=16.3]
- rows3000-still renderer=default ws_kib=102012 private_kib=96052 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=101924 private_kib=95740 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25788 private_kib=6520 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36220 private_kib=18040 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.48
- item-text-0 renderer=cpu ws_kib=25808 private_kib=6520 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34224 private_kib=15928 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.81
- item-button-0 renderer=cpu ws_kib=25792 private_kib=6512 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81864 private_kib=68064 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.69
- item-row-0 renderer=cpu ws_kib=25840 private_kib=6584 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=101404 private_kib=95304 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=25.19

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3390
    [box] H site     476.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     436.9 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     415.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     408.5 B/item     3004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [box] H site     280.3 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     264.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     224.1 B/item     3002 allocs kivact_style::sheet::decl::Style::push
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
- heap mode=button live_bytes_per_item=18280
    [button] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1906.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    1747.6 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1224.5 B/item     9004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [button] H site    1172.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [button] H site    1008.0 B/item    42003 allocs kivact_core::view::mount::node::mount_node
    [button] H site     873.8 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [button] H site     840.3 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     672.1 B/item     9002 allocs kivact_style::sheet::decl::Style::push
- heap mode=row live_bytes_per_item=26844
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1172.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1120.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1120.1 B/item    15002 allocs kivact_style::sheet::decl::Style::push
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site     832.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=332 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=8.20 ws_kib=33004 work_ms p50=1.0278 p95=1.1778 p99=1.4418 scene_p95=0.0355 raster_p95=1.097 interval_p99=17.1889
- rows=0 renderer=cpu frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.80 ws_kib=33032 work_ms p50=1.0017 p95=1.1673 p99=1.4489 scene_p95=0.0353 raster_p95=1.0805 interval_p99=17.1653
- rows=3000 renderer=default frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.07 ws_kib=107212 work_ms p50=1.2667 p95=1.5994 p99=1.8364 scene_p95=0.1239 raster_p95=1.4176 interval_p99=17.4761
- rows=3000 renderer=cpu frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.89 ws_kib=107452 work_ms p50=1.1951 p95=1.3299 p99=1.6579 scene_p95=0.1077 raster_p95=1.1686 interval_p99=17.1256

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.038 p50=5.038 p95=5.038 p99=5.038
- L counter_click n=200 mean=0.995 p50=0.913 p95=1.195 p99=1.374
- L mount_n100 n=1 mean=22.737 p50=22.737 p95=22.737 p99=22.737
- L unrelated_update_n100 n=50 mean=1.293 p50=1.266 p95=1.479 p99=1.641
- L row_update_n100 n=50 mean=1.236 p50=1.213 p95=1.428 p99=1.877
- L pointer_move_n100 n=120 mean=0.276 p50=0.032 p95=1.083 p99=1.246
- L wheel_n100 n=40 mean=0.013 p50=0.012 p95=0.018 p99=0.027
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 12839700, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=173.578 p50=173.578 p95=173.578 p99=173.578
- L unrelated_update_n1000 n=50 mean=1.266 p50=1.262 p95=1.466 p99=2.725
- L row_update_n1000 n=50 mean=1.787 p50=1.699 p95=2.071 p99=3.815
- L pointer_move_n1000 n=120 mean=0.498 p50=0.095 p95=1.628 p99=1.810
- L wheel_n1000 n=40 mean=0.046 p50=0.046 p95=0.050 p99=0.052
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 38622200, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=519.499 p50=519.499 p95=519.499 p99=519.499
- L unrelated_update_n3000 n=50 mean=1.253 p50=1.219 p95=1.392 p99=4.700
- L row_update_n3000 n=50 mean=3.010 p50=2.819 p95=3.496 p99=7.484
- L pointer_move_n3000 n=120 mean=1.060 p50=0.267 p95=3.037 p99=3.878
- L wheel_n3000 n=40 mean=0.132 p50=0.131 p95=0.137 p99=0.148
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 103296600, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18856448
- K6 kv_startup default 18855936
- K6 kv_startup no-default-features 11169792
- K6 kv_startup no-gpu 14224384
