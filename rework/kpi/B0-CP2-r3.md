# KPI B0-CP2-r3 (2026-11-21T13:50Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=47.0 samples=[ 49.3 47.1 47.0 43.4 45.9 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.8 2.7 2.7 3.0 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=45.9 samples=[ 43.1 45.7 45.9 45.9 46.8 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.7 2.6 2.6 2.6 2.6 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=176.0 samples=[ 540.1 176.0 173.4 179.6 168.1 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25652 private_kib=5892 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25680 private_kib=5920 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85236 private_kib=103276 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=611.6 configure_ms=15.6]
- basic-catalog renderer=default ws_kib=51484 private_kib=27712 idle_cpu_ms_per_s=86.015 idle_frames=632 cpu_ticks_100ns=9062500 
- rows3000-still renderer=default ws_kib=99916 private_kib=84468 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51560 private_kib=27608 idle_cpu_ms_per_s=65.259 idle_frames=631 cpu_ticks_100ns=6875000 
- rows3000-still renderer=cpu ws_kib=100232 private_kib=84508 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25824 private_kib=6616 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36616 private_kib=19220 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.60
- item-text-0 renderer=cpu ws_kib=25804 private_kib=6608 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34664 private_kib=16664 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.95
- item-button-0 renderer=cpu ws_kib=25828 private_kib=6620 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81584 private_kib=73500 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.59
- item-row-0 renderer=cpu ws_kib=25816 private_kib=6600 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=100416 private_kib=84988 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.87

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3740
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
- heap mode=button live_bytes_per_item=20377
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
- heap mode=row live_bytes_per_item=23418
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
- rows=0 renderer=default frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.62 ws_kib=33340 work_ms p50=1.0214 p95=1.4322 p99=1.5633 scene_p95=0.0415 raster_p95=1.3621 interval_p99=17.271
- rows=0 renderer=cpu frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.35 ws_kib=33236 work_ms p50=1.0607 p95=1.5009 p99=1.5955 scene_p95=0.0346 raster_p95=1.4099 interval_p99=17.271
- rows=3000 renderer=default frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.04 ws_kib=105612 work_ms p50=1.4669 p95=1.8549 p99=2.151 scene_p95=0.2004 raster_p95=1.5908 interval_p99=17.3512
- rows=3000 renderer=cpu frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.07 ws_kib=105788 work_ms p50=1.4802 p95=1.9512 p99=2.259 scene_p95=0.1949 raster_p95=1.7653 interval_p99=17.2943

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.069 p50=5.069 p95=5.069 p99=5.069
- L counter_click n=200 mean=0.887 p50=0.869 p95=1.036 p99=1.272
- L mount_n100 n=1 mean=21.638 p50=21.638 p95=21.638 p99=21.638
- L unrelated_update_n100 n=50 mean=1.284 p50=1.226 p95=1.603 p99=1.903
- L row_update_n100 n=50 mean=2.178 p50=2.143 p95=2.553 p99=2.951
- L pointer_move_n100 n=120 mean=0.318 p50=0.036 p95=1.332 p99=1.390
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.014 p99=0.026
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 14584400, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=176.916 p50=176.916 p95=176.916 p99=176.916
- L unrelated_update_n1000 n=50 mean=1.264 p50=1.210 p95=1.397 p99=3.110
- L row_update_n1000 n=50 mean=3.417 p50=3.397 p95=3.710 p99=5.448
- L pointer_move_n1000 n=120 mean=0.623 p50=0.096 p95=2.199 p99=2.416
- L wheel_n1000 n=40 mean=0.048 p50=0.047 p95=0.053 p99=0.058
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 44962700, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=518.598 p50=518.598 p95=518.598 p99=518.598
- L unrelated_update_n3000 n=50 mean=1.287 p50=1.189 p95=1.399 p99=5.022
- L row_update_n3000 n=50 mean=7.264 p50=7.130 p95=7.800 p99=11.720
- L pointer_move_n3000 n=120 mean=1.544 p50=0.269 p95=5.037 p99=5.392
- L wheel_n3000 n=40 mean=0.136 p50=0.134 p95=0.153 p99=0.177
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 140558100, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
