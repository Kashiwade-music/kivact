# KPI remote (2026-11-23T05:52Z, MINGW64_NT-10.0-22631, commit e99e222d)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=44.1 samples=[ 45.4 45.8 43.4 44.1 42.2 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.8 2.8 2.9 2.8 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=41.0 samples=[ 41.0 41.2 40.2 39.0 41.1 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 2.8 2.8 2.7 2.9 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=200.3 samples=[ 344.3 200.3 202.9 199.6 200.3 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=22588 private_kib=7060 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=22580 private_kib=7056 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=163080 private_kib=125692 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=159.4 configure_ms=157.7]
- basic-catalog renderer=default ws_kib=40008 private_kib=20536 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=default ws_kib=97344 private_kib=86364 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=39936 private_kib=20384 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=97628 private_kib=86432 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=23248 private_kib=8284 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=33788 private_kib=20660 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.51
- item-text-0 renderer=cpu ws_kib=23220 private_kib=8232 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=31648 private_kib=18248 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.81
- item-button-0 renderer=cpu ws_kib=23200 private_kib=8224 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=78548 private_kib=75472 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.45
- item-row-0 renderer=cpu ws_kib=23216 private_kib=8244 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=97368 private_kib=86400 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.72

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3775
    [box] H site     989.9 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [box] H site     589.9 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     280.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     232.2 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     216.0 B/item     3000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [box] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [box] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [box] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [box] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
- heap mode=text live_bytes_per_item=2971
    [text] H site     989.9 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [text] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     360.6 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [text] H site     192.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
    [text] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [text] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
    [text] H site      54.6 B/item        1 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0<u32,kivact_core::view::descriptor::description::View,alloc::vec::Vec<u32,alloc::alloc::Global>,kv_heap::main::closure$0::closure_env$1>
- heap mode=button live_bytes_per_item=20482
    [button] H site    3959.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [button] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1504.0 B/item    72000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [button] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     696.2 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     560.2 B/item    12004 allocs kivact_core::view::element_data::Common::push_style
    [button] H site     416.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
- heap mode=row live_bytes_per_item=23453
    [row] H site    3959.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [row] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site     928.2 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site     928.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [row] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [row] H site     797.4 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [row] H site     772.0 B/item    24000 allocs kivact_core::view::descriptor::description::impl$7::into_view
    [row] H site     732.0 B/item    48000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=174 fps=32.1 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=33092 work_ms p50=2.1914 p95=3.354 p99=3.758 scene_p95=0.0575 raster_p95=3.1859 interval_p99=33.89
- rows=0 renderer=cpu frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=33052 work_ms p50=2.1353 p95=3.3201 p99=4.0394 scene_p95=0.0554 raster_p95=3.1703 interval_p99=33.3811
- rows=3000 renderer=default frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=106284 work_ms p50=2.3753 p95=4.3307 p99=5.526 scene_p95=0.3446 raster_p95=3.952 interval_p99=34.7968
- rows=3000 renderer=cpu frames=174 fps=32.1 frame_period_ns=31250000 cpu_pct_of_core=7.49 ws_kib=105908 work_ms p50=3.1215 p95=4.5062 p99=5.4263 scene_p95=0.4352 raster_p95=4.0361 interval_p99=34.8838

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=3.948 p50=3.948 p95=3.948 p99=3.948
- L counter_click n=200 mean=0.464 p50=0.460 p95=0.489 p99=0.513
- L mount_n100 n=1 mean=13.342 p50=13.342 p95=13.342 p99=13.342
- L unrelated_update_n100 n=50 mean=0.747 p50=0.709 p95=0.915 p99=1.752
- L row_update_n100 n=50 mean=1.331 p50=1.325 p95=1.438 p99=1.852
- L pointer_move_n100 n=120 mean=0.157 p50=0.020 p95=0.621 p99=0.724
- L wheel_n100 n=40 mean=0.008 p50=0.008 p95=0.008 p99=0.015
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 8414800, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=99.064 p50=99.064 p95=99.064 p99=99.064
- L unrelated_update_n1000 n=50 mean=0.773 p50=0.745 p95=0.864 p99=1.762
- L row_update_n1000 n=50 mean=2.067 p50=2.044 p95=2.122 p99=3.081
- L pointer_move_n1000 n=120 mean=0.336 p50=0.055 p95=1.208 p99=1.257
- L wheel_n1000 n=40 mean=0.028 p50=0.027 p95=0.034 p99=0.034
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 25225900, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=292.207 p50=292.207 p95=292.207 p99=292.207
- L unrelated_update_n3000 n=50 mean=0.813 p50=0.749 p95=0.874 p99=3.519
- L row_update_n3000 n=50 mean=4.652 p50=4.544 p95=5.525 p99=7.463
- L pointer_move_n3000 n=120 mean=0.782 p50=0.168 p95=2.418 p99=2.667
- L wheel_n3000 n=40 mean=0.091 p50=0.091 p95=0.094 p99=0.097
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 67599500, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18949120
- K6 kv_startup default 18944512
- K6 kv_startup no-default-features 11301888
- K6 basic-catalog default 34037248
- K6 kv_startup no-gpu 14356480
- K6 basic-catalog no-gpu 29464064
