# KPI CP1 (2026-11-19T11:24Z, MINGW64_NT-10.0-26200, commit d615568c)

release build of kivact-example-counter (cold for its own target dir): 332 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=52.4 samples=[ 60.3 52.5 48.7 52.4 50.8 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.9 2.7 2.7 2.7 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=51.1 samples=[ 51.9 49.7 54.2 48.1 51.1 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.8 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=181.8 samples=[ 517.0 181.8 179.7 182.7 181.1 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26480 private_kib=5912 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26460 private_kib=5908 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84388 private_kib=103876 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=753.3 configure_ms=18.1]
- basic-catalog renderer=default ws_kib=56524 private_kib=31696 idle_cpu_ms_per_s=121.748 idle_frames=639 cpu_ticks_100ns=12968750 
- rows3000-still renderer=default ws_kib=160772 private_kib=153068 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=56560 private_kib=31668 idle_cpu_ms_per_s=95.200 idle_frames=640 cpu_ticks_100ns=10156250 
- rows3000-still renderer=cpu ws_kib=159980 private_kib=153020 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=26664 private_kib=6608 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=44096 private_kib=26624 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.81
- item-text-0 renderer=cpu ws_kib=26676 private_kib=6616 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=41112 private_kib=22420 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.81
- item-button-0 renderer=cpu ws_kib=26740 private_kib=6672 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=122140 private_kib=111996 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=31.80
- item-row-0 renderer=cpu ws_kib=26672 private_kib=6612 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=159948 private_kib=153128 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=44.43

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=6222
    [box] H site    1048.7 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site    1022.7 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [box] H site     571.9 B/item     3025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [box] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     336.3 B/item     9009 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     280.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     216.1 B/item     3001 allocs kivact_core::view::mount::style::merge_layers::closure$0
    [box] H site     216.0 B/item     3000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [box] H site     207.5 B/item        3 allocs kivact_engine::model::aggregate::Slots::push
- heap mode=text live_bytes_per_item=4684
    [text] H site    1022.7 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [text] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [text] H site     667.2 B/item     9025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [text] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     360.6 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     207.5 B/item        3 allocs kivact_engine::model::aggregate::Slots::push
    [text] H site     183.6 B/item     9013 allocs arvane_native::a11y::conversion::update
    [text] H site     160.1 B/item     6002 allocs kivact_layout::tree::pass::impl$0::leaf::closure$1::closure$2
    [text] H site     114.2 B/item       25 allocs arvane_native::native_input::windows::subclass
    [text] H site     112.3 B/item        3 allocs kivact_engine::paint::list::walk::Walk::node
- heap mode=button live_bytes_per_item=32646
    [button] H site    4090.6 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    3113.0 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [button] H site    2320.9 B/item    24025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [button] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1656.1 B/item    21001 allocs kivact_core::view::mount::style::merge_layers::closure$0
    [button] H site    1600.1 B/item     9001 allocs kivact_style::paint::background::impl$5::clone
    [button] H site    1536.0 B/item    75000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site    1280.0 B/item     6000 allocs kivact_style::sheet::fields::impl$72::clone
    [button] H site    1008.3 B/item    27009 allocs kivact_core::view::mount::style::bind_node_style
- heap mode=row live_bytes_per_item=45718
    [row] H site    7591.3 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    6225.9 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [row] H site    4525.5 B/item    39025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [row] H site    4090.6 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    1992.1 B/item    24001 allocs kivact_core::view::mount::style::merge_layers::closure$0
    [row] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    1600.1 B/item     9001 allocs kivact_style::paint::background::impl$5::clone
    [row] H site    1344.3 B/item    36009 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1280.0 B/item     6000 allocs kivact_style::sheet::fields::impl$72::clone
    [row] H site     936.2 B/item    18004 allocs kivact_style::sheet::fields::Style::merge

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=339 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.78 ws_kib=34256 work_ms p50=1.3745 p95=1.5235 p99=1.6637 scene_p95=0.046 raster_p95=1.418 interval_p99=17.2134
- rows=0 renderer=cpu frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.16 ws_kib=34172 work_ms p50=1.3843 p95=1.5108 p99=1.7152 scene_p95=0.0386 raster_p95=1.4042 interval_p99=17.3468
- rows=3000 renderer=default frames=339 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=13.82 ws_kib=165520 work_ms p50=1.8081 p95=2.1715 p99=2.3056 scene_p95=0.2396 raster_p95=1.8828 interval_p99=17.3594
- rows=3000 renderer=cpu frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.37 ws_kib=165992 work_ms p50=1.7884 p95=1.9097 p99=2.0841 scene_p95=0.2195 raster_p95=1.6425 interval_p99=17.3821

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.453 p50=6.453 p95=6.453 p99=6.453
- L counter_click n=200 mean=1.016 p50=1.100 p95=1.157 p99=1.215
- L mount_n100 n=1 mean=20.492 p50=20.492 p95=20.492 p99=20.492
- L unrelated_update_n100 n=50 mean=1.050 p50=1.014 p95=1.322 p99=1.586
- L row_update_n100 n=50 mean=1.253 p50=1.246 p95=1.287 p99=1.995
- L pointer_move_n100 n=120 mean=0.297 p50=0.033 p95=1.208 p99=1.241
- L wheel_n100 n=40 mean=0.014 p50=0.013 p95=0.023 p99=0.025
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 10152400, live_reactive: 2474, live_owners: 101, live_objects: 205, live_constants: 409, arena_slots: 3189, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=160.296 p50=160.296 p95=160.296 p99=160.296
- L unrelated_update_n1000 n=50 mean=1.060 p50=1.025 p95=1.059 p99=2.563
- L row_update_n1000 n=50 mean=1.910 p50=1.866 p95=1.940 p99=3.868
- L pointer_move_n1000 n=120 mean=0.533 p50=0.093 p95=1.821 p99=1.875
- L wheel_n1000 n=40 mean=0.046 p50=0.046 p95=0.051 p99=0.053
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 36808400, live_reactive: 24074, live_owners: 1001, live_objects: 2005, live_constants: 4009, arena_slots: 31089, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=478.973 p50=478.973 p95=478.973 p99=478.973
- L unrelated_update_n3000 n=50 mean=1.138 p50=1.041 p95=1.347 p99=4.543
- L row_update_n3000 n=50 mean=3.667 p50=3.559 p95=3.867 p99=8.043
- L pointer_move_n3000 n=120 mean=1.452 p50=0.273 p95=4.852 p99=5.070
- L wheel_n3000 n=40 mean=0.136 p50=0.135 p95=0.149 p99=0.163
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 131587200, live_reactive: 72074, live_owners: 3001, live_objects: 6005, live_constants: 12009, arena_slots: 93089, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 19049984
- K6 kv_startup default 19045888
- K6 kv_startup no-default-features 11397120
- K6 basic-catalog default 34378240
- K6 kv_startup no-gpu 14451712
- K6 basic-catalog no-gpu 29795840
