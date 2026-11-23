# KPI remote (2026-11-23T03:39Z, MINGW64_NT-10.0-22631, commit dd9c4dad)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=38.9 samples=[ 39.8 38.3 38.9 37.9 41.0 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.9 2.9 2.7 2.8 2.8 ]
- K1 main_to_first_present renderer=cpu median_ms=40.2 samples=[ 41.2 38.8 38.4 40.2 40.3 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 2.9 2.7 2.8 2.8 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=199.2 samples=[ 324.7 193.5 191.3 199.6 199.2 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=22236 private_kib=6932 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=22260 private_kib=6940 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=160980 private_kib=123704 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=107.2 configure_ms=40.6]
- basic-catalog renderer=default ws_kib=39756 private_kib=21508 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=default ws_kib=97752 private_kib=96440 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=39684 private_kib=21456 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=97232 private_kib=95844 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=23156 private_kib=8092 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=33104 private_kib=19336 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.32
- item-text-0 renderer=cpu ws_kib=23196 private_kib=8160 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=31632 private_kib=17580 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.81
- item-button-0 renderer=cpu ws_kib=23164 private_kib=8112 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=79096 private_kib=69896 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.64
- item-row-0 renderer=cpu ws_kib=23156 private_kib=8112 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=97896 private_kib=96052 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.91

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
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
- heap mode=button live_bytes_per_item=18175
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
- heap mode=row live_bytes_per_item=26389
    [row] H site    3812.0 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    3495.3 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    2654.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    2040.5 B/item    15004 allocs kivact_layout::tree::partial::impl$0::compute_child_layout::closure$0
    [row] H site    1747.6 B/item        4 allocs kivact_engine::model::aggregate::Slots::push
    [row] H site    1284.0 B/item    66000 allocs kivact_core::view::boundary::mount_in
    [row] H site    1216.3 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1048.0 B/item    45003 allocs kivact_core::view::mount::node::mount_node
    [row] H site     832.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=172 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=33004 work_ms p50=2.834 p95=3.1947 p99=4.0832 scene_p95=0.0592 raster_p95=3.0304 interval_p99=33.4331
- rows=0 renderer=cpu frames=172 fps=31.9 frame_period_ns=31250000 cpu_pct_of_core=16.52 ws_kib=32932 work_ms p50=2.8429 p95=3.2518 p99=3.8421 scene_p95=0.0665 raster_p95=3.0583 interval_p99=34.1783
- rows=3000 renderer=default frames=173 fps=32.1 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=106488 work_ms p50=3.2268 p95=3.7398 p99=4.5709 scene_p95=0.1587 raster_p95=3.5154 interval_p99=34.0995
- rows=3000 renderer=cpu frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=106624 work_ms p50=3.2235 p95=3.9052 p99=4.3952 scene_p95=0.1661 raster_p95=3.5899 interval_p99=34.251

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=3.878 p50=3.878 p95=3.878 p99=3.878
- L counter_click n=200 mean=0.395 p50=0.391 p95=0.411 p99=0.450
- L mount_n100 n=1 mean=13.226 p50=13.226 p95=13.226 p99=13.226
- L unrelated_update_n100 n=50 mean=0.473 p50=0.460 p95=0.560 p99=0.777
- L row_update_n100 n=50 mean=0.637 p50=0.615 p95=0.741 p99=1.113
- L pointer_move_n100 n=120 mean=0.121 p50=0.016 p95=0.471 p99=0.516
- L wheel_n100 n=40 mean=0.008 p50=0.007 p95=0.012 p99=0.017
- S stats_n100 Stats { batches: 101, a11y_constructed: 943, a11y_published: 761, a11y_enumerated: 511, ops: 3275, last_batch_ops: 1, creates: 614, set_props: 2047, structural: 613, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 2402900, live_reactive: 1961, live_owners: 203, live_objects: 203, live_constants: 409, arena_slots: 2776, tree_nodes: 614, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=95.440 p50=95.440 p95=95.440 p99=95.440
- L unrelated_update_n1000 n=50 mean=0.489 p50=0.456 p95=0.620 p99=1.481
- L row_update_n1000 n=50 mean=0.990 p50=0.916 p95=1.491 p99=1.860
- L pointer_move_n1000 n=120 mean=0.245 p50=0.059 p95=0.763 p99=0.843
- L wheel_n1000 n=40 mean=0.026 p50=0.026 p95=0.031 p99=0.038
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5443, a11y_published: 5261, a11y_enumerated: 5011, ops: 31175, last_batch_ops: 1, creates: 6014, set_props: 19147, structural: 6013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 16539800, live_reactive: 19061, live_owners: 2003, live_objects: 2003, live_constants: 4009, arena_slots: 27076, tree_nodes: 6014, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=281.607 p50=281.607 p95=281.607 p99=281.607
- L unrelated_update_n3000 n=50 mean=0.511 p50=0.458 p95=0.566 p99=2.674
- L row_update_n3000 n=50 mean=1.514 p50=1.447 p95=2.089 p99=4.042
- L pointer_move_n3000 n=120 mean=0.511 p50=0.169 p95=1.369 p99=1.438
- L wheel_n3000 n=40 mean=0.086 p50=0.085 p95=0.093 p99=0.093
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15443, a11y_published: 15261, a11y_enumerated: 15011, ops: 93175, last_batch_ops: 1, creates: 18014, set_props: 57147, structural: 18013, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 44576900, live_reactive: 57061, live_owners: 6003, live_objects: 6003, live_constants: 12009, arena_slots: 81076, tree_nodes: 18014, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18907136
- K6 kv_startup default 18906624
- K6 kv_startup no-default-features 11220480
- K6 basic-catalog default 35371008
- K6 kv_startup no-gpu 14275584
- K6 basic-catalog no-gpu 30753280
