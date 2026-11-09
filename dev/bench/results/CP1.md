# KPI CP1 (2026-11-09T17:16Z, MINGW64_NT-10.0-26200, commit c33eb86a)

release build of kivact-example-counter (cold for its own target dir): 279 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=41.7 samples=[ 43.5 39.6 41.7 40.0 43.0 ]
- K4 first_frame_raster renderer=default median_ms=3.1 samples=[ 3.3 3.1 3.0 3.0 3.6 ]
- K1 main_to_first_present renderer=cpu median_ms=40.6 samples=[ 40.6 43.0 44.7 39.8 39.5 ]
- K4 first_frame_raster renderer=cpu median_ms=3.2 samples=[ 3.2 3.1 3.0 3.2 3.9 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=194.2 samples=[ 575.3 194.2 199.9 193.5 191.0 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=35076 private_kib=13616 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=35092 private_kib=13628 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=132252 private_kib=132696 idle_cpu_ms_per_s=1.501 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=779.8 configure_ms=14.9]
- basic-catalog renderer=default ws_kib=154460 private_kib=92076 idle_cpu_ms_per_s=124.182 idle_frames=627 cpu_ticks_100ns=12968750 
- rows3000-still renderer=default ws_kib=378700 private_kib=441324 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=154296 private_kib=91800 idle_cpu_ms_per_s=110.806 idle_frames=626 cpu_ticks_100ns=11562500 
- rows3000-still renderer=cpu ws_kib=376524 private_kib=439920 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=35892 private_kib=15856 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=93560 private_kib=79016 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=19.22
- item-text-0 renderer=cpu ws_kib=35868 private_kib=15812 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=67252 private_kib=52264 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=10.46
- item-button-0 renderer=cpu ws_kib=35908 private_kib=15872 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=300576 private_kib=312856 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=88.22
- item-row-0 renderer=cpu ws_kib=35868 private_kib=15832 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=376968 private_kib=440332 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=113.70

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=19608
    [box] H site   19556.7 B/item   108616 allocs 
    [box] H site    3392.8 B/item      288 allocs CallWindowProcW
    [box] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [box] H site       0.2 B/item        1 allocs IsWindowEnabled
- heap mode=text live_bytes_per_item=11185
    [text] H site   11123.5 B/item    46467 allocs 
    [text] H site    3420.4 B/item      672 allocs CallWindowProcW
    [text] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [text] H site       0.2 B/item        1 allocs IsWindowEnabled
- heap mode=button live_bytes_per_item=93498
    [button] H site   93455.5 B/item   891197 allocs 
    [button] H site    3409.6 B/item      150 allocs CallWindowProcW
    [button] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [button] H site       0.2 B/item        1 allocs IsWindowEnabled
- heap mode=row live_bytes_per_item=135231
    [row] H site  134814.9 B/item  1022481 allocs 
    [row] H site    3771.6 B/item      565 allocs CallWindowProcW
    [row] H site       0.2 B/item        1 allocs IsWindowEnabled
    [row] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=326 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=10.07 ws_kib=38432 work_ms p50=1.8143 p95=2.3224 p99=2.5063 scene_p95=0.0335 raster_p95=2.2261 interval_p99=17.5754
- rows=0 renderer=cpu frames=325 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=12.68 ws_kib=38448 work_ms p50=1.8023 p95=2.2244 p99=2.4658 scene_p95=0.0352 raster_p95=2.156 interval_p99=17.5754
- rows=3000 renderer=default frames=272 fps=50.0 frame_period_ns=16666666 cpu_pct_of_core=98.29 ws_kib=376024 work_ms p50=18.2075 p95=19.6638 p99=22.2916 scene_p95=17.7416 raster_p95=2.1615 interval_p99=24.106
- rows=3000 renderer=cpu frames=273 fps=49.9 frame_period_ns=16666666 cpu_pct_of_core=98.87 ws_kib=376828 work_ms p50=18.2536 p95=19.4029 p99=20.756 scene_p95=17.4729 raster_p95=2.1994 interval_p99=22.2525

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.531 p50=6.531 p95=6.531 p99=6.531
- L counter_click n=200 mean=1.957 p50=1.938 p95=2.157 p99=2.268
- L mount_n100 n=1 mean=40.498 p50=40.498 p95=40.498 p99=40.498
- L unrelated_update_n100 n=50 mean=3.493 p50=3.406 p95=3.832 p99=5.047
- L row_update_n100 n=50 mean=8.439 p50=8.540 p95=9.265 p99=9.472
- L pointer_move_n100 n=120 mean=1.336 p50=0.620 p95=3.723 p99=4.481
- L wheel_n100 n=40 mean=0.544 p50=0.537 p95=0.562 p99=0.712
- S stats_n100 Stats { batches: 101, a11y_constructed: 2270, a11y_published: 1573, ops: 2853, last_batch_ops: 1, creates: 1223, set_props: 5609, structural: 1222, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 18940800, live_reactive: 5852, live_owners: 101, live_objects: 409, live_constants: 612, arena_slots: 6974, tree_nodes: 1223, timers: 0, frame_loops: 102, animations: 0 }
- L mount_n1000 n=1 mean=386.359 p50=386.359 p95=386.359 p99=386.359
- L unrelated_update_n1000 n=50 mean=16.833 p50=16.510 p95=19.559 p99=21.527
- L row_update_n1000 n=50 mean=25.507 p50=25.432 p95=27.106 p99=28.085
- L pointer_move_n1000 n=120 mean=12.213 p50=8.076 p95=26.074 p99=27.821
- L wheel_n1000 n=40 mean=7.246 p50=7.208 p95=8.022 p99=8.742
- S stats_n1000 Stats { batches: 101, a11y_constructed: 13070, a11y_published: 12373, ops: 27153, last_batch_ops: 1, creates: 12023, set_props: 54209, structural: 12022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 157453800, live_reactive: 57152, live_owners: 1001, live_objects: 4009, live_constants: 6012, arena_slots: 68174, tree_nodes: 12023, timers: 0, frame_loops: 1002, animations: 0 }
- L mount_n3000 n=1 mean=1578.842 p50=1578.842 p95=1578.842 p99=1578.842
- L unrelated_update_n3000 n=50 mean=63.018 p50=63.087 p95=65.197 p99=69.799
- L row_update_n3000 n=50 mean=82.412 p50=81.888 p95=85.932 p99=88.948
- L pointer_move_n3000 n=120 mean=44.156 p50=28.398 p95=96.943 p99=98.078
- L wheel_n3000 n=40 mean=27.555 p50=27.389 p95=29.168 p99=29.280
- S stats_n3000 Stats { batches: 101, a11y_constructed: 37070, a11y_published: 36373, ops: 81153, last_batch_ops: 1, creates: 36023, set_props: 162209, structural: 36022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 464482700, live_reactive: 171152, live_owners: 3001, live_objects: 12009, live_constants: 18012, arena_slots: 204174, tree_nodes: 36023, timers: 0, frame_loops: 3002, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18949632
- K6 kv_startup default 18947584
- K6 kv_startup no-default-features 15892992
- K6 basic-catalog default 43045888
