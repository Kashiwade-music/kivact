# KPI CP2 (2026-11-09T23:36Z, MINGW64_NT-10.0-26200, commit 4f7351e1)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=40.9 samples=[ 43.6 39.4 40.9 41.5 40.3 ]
- K4 first_frame_raster renderer=default median_ms=3.2 samples=[ 3.3 3.0 3.0 4.0 3.2 ]
- K1 main_to_first_present renderer=cpu median_ms=41.3 samples=[ 42.1 40.3 44.1 39.6 41.3 ]
- K4 first_frame_raster renderer=cpu median_ms=3.0 samples=[ 3.0 3.1 3.0 3.1 2.9 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=196.8 samples=[ 574.8 201.8 188.2 196.8 184.1 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=35068 private_kib=13548 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=35076 private_kib=13556 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=94704 private_kib=111044 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=368.8 configure_ms=14.4]
- basic-catalog renderer=default ws_kib=152096 private_kib=89516 idle_cpu_ms_per_s=122.666 idle_frames=627 cpu_ticks_100ns=12812500 
- rows3000-still renderer=default ws_kib=275332 private_kib=328012 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=151708 private_kib=88964 idle_cpu_ms_per_s=116.823 idle_frames=626 cpu_ticks_100ns=12187500 
- rows3000-still renderer=cpu ws_kib=275880 private_kib=328292 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=35956 private_kib=15816 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=69936 private_kib=54004 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=11.33
- item-text-0 renderer=cpu ws_kib=35896 private_kib=15752 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=64580 private_kib=48884 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=9.56
- item-button-0 renderer=cpu ws_kib=35884 private_kib=15736 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=223544 private_kib=216096 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=62.55
- item-row-0 renderer=cpu ws_kib=35924 private_kib=15764 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=275328 private_kib=328208 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=79.80

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=11604
    [box] H site   11448.6 B/item   102603 allocs 
    [box] H site    3474.8 B/item      293 allocs CallWindowProcW
    [box] H site       0.2 B/item        1 allocs IsWindowEnabled
    [box] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
- heap mode=text live_bytes_per_item=9961
    [text] H site    9856.6 B/item    49453 allocs 
    [text] H site    3420.3 B/item      668 allocs CallWindowProcW
    [text] H site       0.2 B/item        1 allocs IsWindowEnabled
    [text] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
- heap mode=button live_bytes_per_item=61901
    [button] H site   61745.4 B/item   786192 allocs 
    [button] H site    3491.2 B/item      149 allocs CallWindowProcW
    [button] H site       0.2 B/item        1 allocs IsWindowEnabled
    [button] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
- heap mode=row live_bytes_per_item=98496
    [row] H site   97947.8 B/item   914456 allocs 
    [row] H site    3853.5 B/item      566 allocs CallWindowProcW
    [row] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [row] H site       0.2 B/item        1 allocs IsWindowEnabled

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=325 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=10.37 ws_kib=38444 work_ms p50=1.4879 p95=1.8643 p99=2.4056 scene_p95=0.0357 raster_p95=1.758 interval_p99=17.6138
- rows=0 renderer=cpu frames=326 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.21 ws_kib=38496 work_ms p50=1.4468 p95=1.8235 p99=2.12 scene_p95=0.0374 raster_p95=1.7148 interval_p99=17.4295
- rows=3000 renderer=default frames=302 fps=54.9 frame_period_ns=16666666 cpu_pct_of_core=99.95 ws_kib=274348 work_ms p50=17.1512 p95=18.823 p99=21.3854 scene_p95=16.7678 raster_p95=1.9254 interval_p99=22.1379
- rows=3000 renderer=cpu frames=296 fps=53.9 frame_period_ns=16666666 cpu_pct_of_core=99.59 ws_kib=274688 work_ms p50=17.5497 p95=19.0386 p99=20.5416 scene_p95=17.1074 raster_p95=2.0769 interval_p99=21.4011

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.624 p50=6.624 p95=6.624 p99=6.624
- L counter_click n=200 mean=1.784 p50=1.759 p95=1.877 p99=2.188
- L mount_n100 n=1 mean=36.633 p50=36.633 p95=36.633 p99=36.633
- L unrelated_update_n100 n=50 mean=3.326 p50=3.266 p95=3.564 p99=5.311
- L row_update_n100 n=50 mean=7.828 p50=7.942 p95=8.326 p99=8.616
- L pointer_move_n100 n=120 mean=1.297 p50=0.581 p95=3.851 p99=4.245
- L wheel_n100 n=40 mean=0.503 p50=0.502 p95=0.529 p99=0.539
- S stats_n100 Stats { batches: 101, a11y_constructed: 2270, a11y_published: 1573, ops: 2853, last_batch_ops: 1, creates: 1223, set_props: 4993, structural: 1222, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 18324400, live_reactive: 4730, live_owners: 101, live_objects: 205, live_constants: 714, arena_slots: 5750, tree_nodes: 1223, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=362.218 p50=362.218 p95=362.218 p99=362.218
- L unrelated_update_n1000 n=50 mean=16.652 p50=16.421 p95=18.355 p99=20.925
- L row_update_n1000 n=50 mean=25.227 p50=25.282 p95=26.572 p99=27.122
- L pointer_move_n1000 n=120 mean=11.016 p50=7.102 p95=25.212 p99=25.782
- L wheel_n1000 n=40 mean=6.249 p50=6.238 p95=7.034 p99=7.077
- S stats_n1000 Stats { batches: 101, a11y_constructed: 13070, a11y_published: 12373, ops: 27153, last_batch_ops: 1, creates: 12023, set_props: 48193, structural: 12022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 109213900, live_reactive: 46130, live_owners: 1001, live_objects: 2005, live_constants: 7014, arena_slots: 56150, tree_nodes: 12023, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=1459.317 p50=1459.317 p95=1459.317 p99=1459.317
- L unrelated_update_n3000 n=50 mean=58.705 p50=58.437 p95=60.918 p99=66.226
- L row_update_n3000 n=50 mean=79.276 p50=79.039 p95=81.729 p99=83.647
- L pointer_move_n3000 n=120 mean=38.720 p50=23.156 p95=87.917 p99=88.398
- L wheel_n3000 n=40 mean=22.931 p50=22.868 p95=23.562 p99=24.428
- S stats_n3000 Stats { batches: 101, a11y_constructed: 37070, a11y_published: 36373, ops: 81153, last_batch_ops: 1, creates: 36023, set_props: 144193, structural: 36022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 300760600, live_reactive: 138130, live_owners: 3001, live_objects: 6005, live_constants: 21014, arena_slots: 168150, tree_nodes: 36023, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18978816
- K6 kv_startup default 18976768
- K6 kv_startup no-default-features 15922176
- K6 basic-catalog default 43090944
